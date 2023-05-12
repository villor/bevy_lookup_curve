use bevy_reflect::{Reflect, FromReflect};
use bevy_math::Vec2;
use egui::{Pos2, Ui, emath, Frame, Shape, Color32, Rect, Painter, Stroke, Sense , epaint::CubicBezierShape};

use crate::{LookupCurve, Knot, KnotInterpolation};

#[derive(Reflect, FromReflect)]
pub struct LookupCurveEditor {
  //pub curve: Option<Handle<LookupCurve>>,
  pub offset: Vec2,
  pub scale: Vec2,

  pub grid_step_x: f32,
  pub grid_step_y: f32,

  pub editor_size: Vec2,
  pub hover_point: Vec2,
}

impl Default for LookupCurveEditor {
  fn default() -> Self {
    Self {
      //curve: None,
      offset: Vec2::ZERO,
      scale: Vec2::new(1.0, 1.0),

      grid_step_x: 0.1,
      grid_step_y: 0.1,

      editor_size: Vec2::ZERO,
      hover_point: Vec2::ZERO,
    }
  }
}

impl LookupCurveEditor {


  // TODO : Rename these functions and make them clearer
  // Move to a paintcontext? with access to to_screeen / to_canvas
  fn curve_to_canvas(&self, curve: Vec2) -> Pos2 {
    let canvas = (curve - self.offset) * self.editor_size / self.scale;
    Pos2::new(canvas.x, self.editor_size.y - canvas.y)
  }

  fn canvas_to_curve(&self, canvas: Pos2) -> Vec2 {
    let canvas = Vec2::new(canvas.x, self.editor_size.y - canvas.y);
    self.offset + canvas / self.editor_size * self.scale
  }

  fn canvas_to_curve_vec(&self, canvas: emath::Vec2) -> Vec2 {
    let canvas = Vec2::new(canvas.x, -canvas.y);
    canvas / self.editor_size * self.scale
  }

  pub fn ui<'a>(&mut self, ui: &mut Ui, curves: impl Iterator<Item = &'a mut LookupCurve>, sample: Option<f32>) {
    ui.label(format!("x = {}, y = {}", self.hover_point.x, self.hover_point.y));

    Frame::canvas(ui.style()).show(ui, |ui| {
      let (response, painter) =
        ui.allocate_painter(emath::Vec2::new(ui.available_width(), ui.available_height()), Sense::hover());

      let to_screen = emath::RectTransform::from_to(
        Rect::from_min_size(Pos2::ZERO, response.rect.size()),
        response.rect,
      );
      let to_canvas = emath::RectTransform::from_to(
        response.rect,
        Rect::from_min_size(Pos2::ZERO, response.rect.size()),
      );
      
      let width = response.rect.width();
      let height = response.rect.height();
      self.editor_size = Vec2::new(width, height);

      if let Some(hover_pos) = response.hover_pos() {
        self.hover_point = self.canvas_to_curve(to_canvas.transform_pos(hover_pos));
      } else {
        self.hover_point = Vec2::ZERO;
      }

      self.paint_grid(&painter, &to_screen);

      // Draw the curve
      for curve in curves {
        let curve_stroke = Stroke {
          color: Color32::GREEN,
          width: 2.0,
        };

        // TODO: Only knots inside viewport
        let mut prev_knot: Option<&Knot> = None;
        for knot in curve.knots().iter() {
          if let Some(prev_knot) = prev_knot {
            match prev_knot.interpolation {
              KnotInterpolation::Constant => {
                painter.add(Shape::line(vec![
                  to_screen.transform_pos(self.curve_to_canvas(prev_knot.position)),
                  to_screen.transform_pos(self.curve_to_canvas(Vec2::new(knot.position.x, prev_knot.position.y))),
                  to_screen.transform_pos(self.curve_to_canvas(knot.position)),
                ], curve_stroke));
              },
              KnotInterpolation::Linear => {
                painter.add(Shape::line(vec![
                  to_screen.transform_pos(self.curve_to_canvas(prev_knot.position)),
                  to_screen.transform_pos(self.curve_to_canvas(knot.position)),
                ], curve_stroke));
              },
              KnotInterpolation::Bezier => {
                painter.add(CubicBezierShape::from_points_stroke([
                  to_screen.transform_pos(self.curve_to_canvas(prev_knot.position)),
                  to_screen.transform_pos(self.curve_to_canvas(prev_knot.position + prev_knot.right_tangent)),
                  to_screen.transform_pos(self.curve_to_canvas(knot.position + knot.left_tangent)),
                  to_screen.transform_pos(self.curve_to_canvas(knot.position)),
                ], false, Color32::TRANSPARENT, curve_stroke));
              }
            }
          }

          prev_knot = Some(knot);
        }

        // Handles
        let knot_radius = 8.0;
        let mut modified_knot = None;
        for (i, knot) in curve.knots().iter().enumerate() {
          let point_in_screen = to_screen.transform_pos(self.curve_to_canvas(knot.position));
          let interact_rect = Rect::from_center_size(point_in_screen, emath::Vec2::splat(2.0 * knot_radius));
          let interact_id = response.id.with(knot.id);
          let interact_response = ui.interact(interact_rect, interact_id, Sense::drag());

          if interact_response.dragged() {
            modified_knot = Some((i, Knot {
              position: knot.position + self.canvas_to_curve_vec(interact_response.drag_delta()),
              ..*knot
            }));
          }

          painter.add(Shape::circle_filled(
            to_screen.transform_pos(self.curve_to_canvas(knot.position)),
            3.0,
            Color32::LIGHT_GREEN
          ));

          // right tangent
          {
            let point_in_screen = to_screen.transform_pos(self.curve_to_canvas(knot.position + knot.right_tangent));
            let interact_rect = Rect::from_center_size(point_in_screen, emath::Vec2::splat(2.0 * knot_radius));
            let interact_id = interact_id.with(0);
            let interact_response = ui.interact(interact_rect, interact_id, Sense::drag());

            if interact_response.dragged() {
              modified_knot = Some((i, Knot {
                right_tangent: knot.right_tangent + self.canvas_to_curve_vec(interact_response.drag_delta()),
                ..*knot
              }));
            }

            painter.add(Shape::circle_filled(
              point_in_screen,
              3.0,
              Color32::LIGHT_GRAY
            ));
          }

          // left tangent
          {
            let point_in_screen = to_screen.transform_pos(self.curve_to_canvas(knot.position + knot.left_tangent));
            let interact_rect = Rect::from_center_size(point_in_screen, emath::Vec2::splat(2.0 * knot_radius));
            let interact_id = interact_id.with(1);
            let interact_response = ui.interact(interact_rect, interact_id, Sense::drag());

            if interact_response.dragged() {
              modified_knot = Some((i, Knot {
                left_tangent: knot.left_tangent + self.canvas_to_curve_vec(interact_response.drag_delta()),
                ..*knot
              }));
            }

            painter.add(Shape::circle_filled(
              point_in_screen,
              3.0,
              Color32::LIGHT_GRAY
            ));
          }
        }

        // Apply modifications
        if let Some((i, knot)) = modified_knot {
          curve.modify_knot(i, &knot);
        }

        // Sample to visualize and test find_y_given_x
        if let Some(sample) = sample {
          painter.add(Shape::circle_filled(
            to_screen.transform_pos(self.curve_to_canvas(Vec2::new(sample, curve.find_y_given_x(sample)))),
            3.0,
            Color32::RED
          ));
        }
      }

    });
  }

  fn paint_grid(&mut self, painter: &Painter, to_screen: &emath::RectTransform) {
    // vertical lines
    if self.grid_step_x > 0.0 {
      let grid_offset_x = self.offset.x % self.grid_step_x;
      let grid_x_count = (self.scale.x / self.grid_step_x).ceil() as i32 + 1;
      for i in 0..grid_x_count {
        let grid_local_x = (i as f32) * self.grid_step_x - grid_offset_x;

        let line_from = self.offset + Vec2::new(grid_local_x, 0.0);
        let line_to = self.offset + Vec2::new(grid_local_x, self.scale.y);

        painter.add(Shape::LineSegment {
          points: [
            to_screen.transform_pos(self.curve_to_canvas(line_from)),
            to_screen.transform_pos(self.curve_to_canvas(line_to))
          ],
          stroke: Stroke {
            width: 1.0,
            color: Color32::DARK_GRAY,
          },
        });
      }
    }

    // horizontal lines
    if self.grid_step_y > 0.0 {
      let grid_offset_y = self.offset.y % self.grid_step_y;
      let grid_y_count = (self.scale.y / self.grid_step_y).ceil() as i32 + 1;
      for i in 0..grid_y_count {
        let grid_local_y = (i as f32) * self.grid_step_y - grid_offset_y;

        let line_from = self.offset + Vec2::new(0.0, grid_local_y);
        let line_to = self.offset + Vec2::new(self.scale.x, grid_local_y);

        painter.add(Shape::LineSegment {
          points: [
            to_screen.transform_pos(self.curve_to_canvas(line_from)),
            to_screen.transform_pos(self.curve_to_canvas(line_to))
          ],
          stroke: Stroke {
            width: 1.0,
            color: Color32::DARK_GRAY,
          },
        });
      }
    }

  }
}
