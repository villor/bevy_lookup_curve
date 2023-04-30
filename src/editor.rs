use bevy_reflect::{Reflect, FromReflect};
use bevy_math::Vec2;
use egui::{Pos2, Ui, emath, Frame, Shape, Color32, Rect, Painter, Stroke, Sense, epaint::CubicBezierShape};

use crate::{LookupCurve, Key, KeyInterpolation};

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

  pub fn ui<'a>(&mut self, ui: &mut Ui, curves: impl Iterator<Item = &'a mut LookupCurve>) {
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

      for curve in curves {
        let curve_stroke = Stroke {
          color: Color32::GREEN,
          width: 2.0,
        };

        // TODO: Only keys inside viewport
        let mut prev_key: Option<&Key> = None;
        for key in curve.keys.iter() {
          if let Some(prev_key) = prev_key {
            match prev_key.interpolation {
              KeyInterpolation::Constant => {
                painter.add(Shape::line(vec![
                  to_screen.transform_pos(self.curve_to_canvas(prev_key.position)),
                  to_screen.transform_pos(self.curve_to_canvas(Vec2::new(key.position.x, prev_key.position.y))),
                  to_screen.transform_pos(self.curve_to_canvas(key.position)),
                ], curve_stroke));
              },
              KeyInterpolation::Linear => {
                painter.add(Shape::line(vec![
                  to_screen.transform_pos(self.curve_to_canvas(prev_key.position)),
                  to_screen.transform_pos(self.curve_to_canvas(key.position)),
                ], curve_stroke));
              },
              // KeyInterpolation::Bezier => {
              //   painter.add(CubicBezierShape::from_points_stroke([
              //     to_screen.transform_pos(self.curve_to_canvas(prev_key.position)),
              //     to_screen.transform_pos(self.curve_to_canvas(prev_key.handle_right.position)),
              //     to_screen.transform_pos(self.curve_to_canvas(key.handle_left.position)),
              //     to_screen.transform_pos(self.curve_to_canvas(key.position)),
              //   ], false, Color32::TRANSPARENT, curve_stroke));
              // }
            }
          }

          prev_key = Some(key);
        }

        // Handles
        let key_radius = 8.0;
        for (i, key) in curve.keys.iter_mut().enumerate() {
          let point_in_screen = to_screen.transform_pos(self.curve_to_canvas(key.position));
          let interact_rect = Rect::from_center_size(point_in_screen, emath::Vec2::splat(2.0 * key_radius));
          let interact_id = response.id.with(i);
          let interact_response = ui.interact(interact_rect, interact_id, Sense::drag());

          if interact_response.dragged() {
            // TODO: fix ordering
            key.position += self.canvas_to_curve_vec(interact_response.drag_delta());
          }

          painter.add(Shape::circle_filled(
            to_screen.transform_pos(self.curve_to_canvas(key.position)),
            3.0,
            Color32::LIGHT_GREEN
          ));
        }
      }

    });
  }

  fn paint_grid(&mut self, painter: &Painter, to_screen: &emath::RectTransform) {
    // vertical lines
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

    // horizontal lines
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


// fn curve_editor_render(
//   mut painter: ShapePainter,
//   editors: Query<&LookupCurveEditor>,
//   lookup_curves: Res<Assets<LookupCurve>>,
// ) {
//   for editor in editors.iter() {
//     if let Some(curve) = lookup_curves.get(&editor.curve) {
//       // Axes
//       painter.color = Color::BLACK;
//       painter.thickness = 1.0;
//       painter.cap = Cap::None;
//       painter.line(Vec3::new(0.0, 0.0, 0.0), Vec3::new(editor.width, 0.0, 0.0));
//       painter.line(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, editor.height, 0.0));

//       painter.color = Color::LIME_GREEN;
//       for key in curve.keys.iter() {
//         painter.transform.translation = (key.position * editor.scale).extend(0.0);
//         painter.circle(3.0);
//       }

//       painter.transform.translation = Vec3::ZERO;
//       painter.thickness = 2.0;
//       painter.cap = Cap::Round;

//       let mut prev_x = 0.0;
//       let mut prev_y = 0.0;
//       for a in 0..=editor.num_samples {
//         let editor_x = editor.offset.x + ((a as f32) / (editor.num_samples as f32)) * editor.width;
//         let curve_x = editor_x / editor.scale.x;
//         let curve_y = curve.find_y_given_x(curve_x);
//         let editor_y = curve_y * editor.scale.y;

//         if a > 0 {
//           painter.line(Vec3::new(prev_x, prev_y, 0.0), Vec3::new(editor_x, editor_y, 0.0));
//         }

//         prev_x = editor_x;
//         prev_y = editor_y;
//       }

//       // for a in 0..num_segments {
//       //   let b = a + 1;

//       //   let editor_ax = ((a as f32) / (num_segments as f32)) * editor_width;
//       //   let editor_bx = ((b as f32) / (num_segments as f32)) * editor_width;

//       //   let ax = (a as f32) / (num_segments as f32) * curve_width;
//       //   let bx = (b as f32) / (num_segments as f32) * curve_width;
//       //   let ay = curve.find_y_given_x(ax);
//       //   let by = curve.find_y_given_x(bx);

//       //   let norm_ax = 

//       //   let line_ax = ax / curve_width * editor_width;
//       //   let line_ay = ay / curve_width * editor_height;
//       //   let line_bx = bx / curve_width * editor_width;
//       //   let line_by = by / curve_width * editor_height;

//       //   painter.cap = Cap::Round;
//       //   painter.line(Vec3::new(line_ax, line_ay, 0.0), Vec3::new(line_bx, line_by, 0.0));
//       // }
//     }
//   }
// }