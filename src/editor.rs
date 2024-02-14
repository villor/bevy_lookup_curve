use bevy_app::{App, Plugin, Update};
use bevy_asset::{Assets, Handle};
use bevy_ecs::prelude::{Component, Query, ResMut};
use bevy_egui::{EguiContexts, EguiPlugin};
use bevy_reflect::Reflect;
use bevy_math::Vec2;
use egui::{Pos2, Ui, emath, Frame, Shape, Color32, Rect, Painter, Stroke, Sense , epaint::CubicBezierShape};

use crate::{LookupCurve, Knot, KnotInterpolation, asset::save_lookup_curve};

pub(crate) struct EditorPlugin;

impl Plugin for EditorPlugin {
  fn build(&self, app: &mut App) {
    if !app.is_plugin_added::<EguiPlugin>() {
      app.add_plugins(EguiPlugin);
    }
    app.add_systems(Update, lookup_curve_editor_ui);
  }
}

#[derive(Component, Reflect)]
pub struct LookupCurveEditor {
  pub title: String,
  pub curve_handle: Handle<LookupCurve>,
  pub egui_editor: LookupCurveEguiEditor,
  pub sample: Option<f32>,
}

impl LookupCurveEditor {
  pub fn new(curve_handle: Handle<LookupCurve>) -> Self {
    Self {
      title: "Lookup curve".to_string(),
      curve_handle,
      egui_editor: LookupCurveEguiEditor::default(),
      sample: None,
    }
  }

  pub fn with_save_path(curve_handle: Handle<LookupCurve>, path: String) -> Self {
    Self {
      egui_editor: LookupCurveEguiEditor {
        ron_path: Some(path),
        ..Default::default()
      },
      ..LookupCurveEditor::new(curve_handle)
    }
  }
}

fn lookup_curve_editor_ui(
  mut editors: Query<&mut LookupCurveEditor>,
  mut contexts: EguiContexts,
  mut curves: ResMut<Assets<LookupCurve>>,
) {
  for mut editor in &mut editors {
    if let Some(curve) = curves.get_mut(&editor.curve_handle) {
      egui::Window::new(editor.title.clone())
        .show(contexts.ctx_mut(), |ui| {
          let sample = editor.sample;
          editor.egui_editor.ui(ui, curve, sample);
        });
    }
  }
}

#[derive(Reflect)]
pub struct LookupCurveEguiEditor {
  //pub curve: Option<Handle<LookupCurve>>,
  pub offset: Vec2,
  pub scale: Vec2,

  pub grid_step_x: f32,
  pub grid_step_y: f32,

  pub editor_size: Vec2,
  pub hover_point: Vec2,

  pub ron_path: Option<String>,
}

impl Default for LookupCurveEguiEditor {
  fn default() -> Self {
    Self {
      //curve: None,
      offset: Vec2::ZERO,
      scale: Vec2::new(1.0, 1.0),

      grid_step_x: 0.1,
      grid_step_y: 0.1,

      editor_size: Vec2::ZERO,
      hover_point: Vec2::ZERO,

      ron_path: None,
    }
  }
}

impl LookupCurveEguiEditor {

  pub fn with_save_path(path: String) -> Self {
    Self {
      ron_path: Some(path),
      ..Default::default()
    }
  }

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

  pub fn ui(&mut self, ui: &mut Ui, curve: &mut LookupCurve, sample: Option<f32>) {
    ui.label(format!("x = {}, y = {}", self.hover_point.x, self.hover_point.y));

    if self.ron_path.is_some() && ui.button("Save").clicked() {
      if let Err(e) = save_lookup_curve(self.ron_path.as_ref().unwrap().as_str(), curve) {
        bevy_log::error!("Failed to save curve {}", e);
      } else {
        bevy_log::info!("Curve saved successfully.");
      }
    }

    Frame::canvas(ui.style()).show(ui, |ui| {
      let (mut response, painter) =
        ui.allocate_painter(emath::Vec2::new(ui.available_width(), ui.available_height()), Sense::click_and_drag());

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

        // Zooming
        ui.input(|input| {
          let scroll_delta = input.scroll_delta.y;
          if scroll_delta != 0.0 {
            self.scale *= 1.0 + -scroll_delta * 0.001;
            // TODO: adjust offset accordingly
          }
        });
      } else {
        self.hover_point = Vec2::ZERO;
      }

      // Panning
      if response.dragged_by(egui::PointerButton::Middle) {
        self.offset -= self.canvas_to_curve_vec(response.drag_delta());
      }
      
      response = response.context_menu(|ui| {
        let menu_pos = ui.min_rect().left_top();// hacky and not entirely correct
        if ui.button("Add knot").clicked() {
          curve.add_knot(Knot {
            position: self.canvas_to_curve(to_canvas.transform_pos(menu_pos)),
            ..Default::default()
          });
          ui.close_menu();
        }
      });

      self.paint_grid(&painter, &to_screen);

      // Draw the curve
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
                to_screen.transform_pos(self.curve_to_canvas(prev_knot.position + prev_knot.right_tangent_corrected(Some(knot)))),
                to_screen.transform_pos(self.curve_to_canvas(knot.position + knot.left_tangent_corrected(Some(prev_knot)))),
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
      let mut deleted_knot_index = None;
      for (i, knot) in curve.knots().iter().enumerate() {
        let prev_knot = curve.prev_knot(i);
        let next_knot = curve.next_knot(i);

        let point_in_screen = to_screen.transform_pos(self.curve_to_canvas(knot.position));
        let interact_rect = Rect::from_center_size(point_in_screen, emath::Vec2::splat(2.0 * knot_radius));
        let interact_id = response.id.with(knot.id);
        let interact_response = ui.interact(interact_rect, interact_id, Sense::drag());

        if interact_response.dragged_by(egui::PointerButton::Primary) {
          modified_knot = Some((i, Knot {
            position: knot.position + self.canvas_to_curve_vec(interact_response.drag_delta()),
            ..*knot
          }));
        }

        interact_response.context_menu(|ui| {
          ui.label("Interpolation");
          if ui.radio(matches!(knot.interpolation, KnotInterpolation::Constant), "Constant").clicked() {
            modified_knot = Some((i, Knot {
              interpolation: KnotInterpolation::Constant,
              ..*knot
            }));
            ui.close_menu();
          }
          if ui.radio(matches!(knot.interpolation, KnotInterpolation::Linear), "Linear").clicked() {
            modified_knot = Some((i, Knot {
              interpolation: KnotInterpolation::Linear,
              ..*knot
            }));
            ui.close_menu();
          }
          if ui.radio(matches!(knot.interpolation, KnotInterpolation::Bezier), "Bezier").clicked() {
            modified_knot = Some((i, Knot {
              interpolation: KnotInterpolation::Bezier,
              ..*knot
            }));
            ui.close_menu();
          }

          ui.label("Position");
          ui.horizontal(|ui| {
            ui.label("x:");
            ui.add(egui::DragValue::from_get_set(|v|
              match v {
                Some(v) => {
                  modified_knot = Some((i, Knot {
                    position: Vec2::new(v as f32, knot.position.y),
                    ..*knot
                  }));
                  v
                },
                _ => knot.position.x as f64,
              }
            ).speed(0.001));
            ui.label("y:");
            ui.add(egui::DragValue::from_get_set(|v|
              match v {
                Some(v) => {
                  modified_knot = Some((i, Knot {
                    position: Vec2::new(knot.position.x, v as f32),
                    ..*knot
                  }));
                  v
                },
                _ => knot.position.y as f64,
              }
            ).speed(0.001));
          });
          
          ui.label("Actions");
          if ui.button("Delete knot").clicked() {
            deleted_knot_index = Some(i);
            ui.close_menu();
          }
        });

        painter.add(Shape::circle_filled(
          to_screen.transform_pos(self.curve_to_canvas(knot.position)),
          3.0,
          Color32::LIGHT_GREEN
        ));

        // right tangent
        if matches!(knot.interpolation, KnotInterpolation::Bezier) {
          let point_in_screen = to_screen.transform_pos(self.curve_to_canvas(knot.position + knot.right_tangent.position));
          let interact_rect = Rect::from_center_size(point_in_screen, emath::Vec2::splat(2.0 * knot_radius));
          let interact_id = interact_id.with(0);
          let interact_response = ui.interact(interact_rect, interact_id, Sense::drag());

          if interact_response.dragged_by(egui::PointerButton::Primary) {
            modified_knot = Some((i, knot.with_right_tangent_position(knot.right_tangent.position + self.canvas_to_curve_vec(interact_response.drag_delta()))));
          }

          let corrected = knot.right_tangent_corrected(next_knot);
          let corrected_in_screen = to_screen.transform_pos(self.curve_to_canvas(knot.position + corrected));
          if corrected != knot.right_tangent.position {
            painter.add(Shape::dashed_line(&[to_screen.transform_pos(self.curve_to_canvas(knot.position)), corrected_in_screen], Stroke::new(1.0, Color32::GRAY), 4.0, 2.0));
            painter.add(Shape::dashed_line(&[corrected_in_screen, point_in_screen], Stroke::new(1.0, Color32::RED), 4.0, 2.0));
            painter.add(Shape::circle_filled(
              corrected_in_screen,
              2.0,
              Color32::RED
            ));
          } else {
            painter.add(Shape::dashed_line(&[to_screen.transform_pos(self.curve_to_canvas(knot.position)), corrected_in_screen], Stroke::new(1.0, Color32::GRAY), 4.0, 2.0));
          }

          painter.add(Shape::circle_filled(
            point_in_screen,
            3.0,
            Color32::LIGHT_GRAY
          ));
        }

        // left tangent
        if prev_knot.is_some() && matches!(prev_knot.unwrap().interpolation, KnotInterpolation::Bezier){
          let point_in_screen = to_screen.transform_pos(self.curve_to_canvas(knot.position + knot.left_tangent.position));
          let interact_rect = Rect::from_center_size(point_in_screen, emath::Vec2::splat(2.0 * knot_radius));
          let interact_id = interact_id.with(1);
          let interact_response = ui.interact(interact_rect, interact_id, Sense::drag());

          if interact_response.dragged_by(egui::PointerButton::Primary) {
            modified_knot = Some((i, knot.with_left_tangent_position(knot.left_tangent.position + self.canvas_to_curve_vec(interact_response.drag_delta()))));
          }

          let corrected = knot.left_tangent_corrected(prev_knot);
          let corrected_in_screen = to_screen.transform_pos(self.curve_to_canvas(knot.position + corrected));
          if corrected != knot.left_tangent.position {
            painter.add(Shape::dashed_line(&[to_screen.transform_pos(self.curve_to_canvas(knot.position)), corrected_in_screen], Stroke::new(1.0, Color32::GRAY), 4.0, 2.0));
            painter.add(Shape::dashed_line(&[corrected_in_screen, point_in_screen], Stroke::new(1.0, Color32::RED), 4.0, 2.0));
            painter.add(Shape::circle_filled(
              corrected_in_screen,
              2.0,
              Color32::RED
            ));
          } else {
            painter.add(Shape::dashed_line(&[to_screen.transform_pos(self.curve_to_canvas(knot.position)), corrected_in_screen], Stroke::new(1.0, Color32::GRAY), 4.0, 2.0));
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
        curve.modify_knot(i, knot);
      }
      if let Some(i) = deleted_knot_index {
        curve.delete_knot(i);
      }

      // Sample to visualize and test find_y_given_x
      if let Some(sample) = sample {
        painter.add(Shape::circle_filled(
          to_screen.transform_pos(self.curve_to_canvas(Vec2::new(sample, curve.find_y_given_x(sample)))),
          3.0,
          Color32::RED
        ));
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
            color: Color32::from_rgb(42, 42, 42),
          },
        });

        painter.text(
          to_screen.transform_pos(Pos2::new(self.curve_to_canvas(line_from).x, self.editor_size.y - 5.)),
          egui::Align2::CENTER_BOTTOM,
          format!("{:.1}", line_from.x),
          egui::FontId::default(),
          Color32::WHITE
        );
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
            color: Color32::from_rgb(42, 42, 42),
          },
        });

        let text_canvas_pos = Pos2::new(5., self.curve_to_canvas(line_from).y);
        if text_canvas_pos.y < self.editor_size.y - 30. {
          painter.text(
            to_screen.transform_pos(text_canvas_pos),
            egui::Align2::LEFT_CENTER,
            format!("{:.1}", line_from.y),
            egui::FontId::default(),
            Color32::WHITE
          );
        }
      }
    }

  }
}
