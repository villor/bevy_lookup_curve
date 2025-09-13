use bevy_math::Vec2;
use egui::{
    Color32, Frame, Id, Painter, Pos2, Rect, Sense, Shape, Stroke, Ui, emath,
    epaint::CubicBezierShape,
};

use crate::{Knot, KnotInterpolation, LookupCurve, TangentMode, TangentSide};

#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
/// Lookup curve editor implemented using `egui`.
///
/// Holds the editor state.
pub struct LookupCurveEguiEditor {
    pub offset: Vec2,
    pub scale: Vec2,

    pub grid_step_x: f32,
    pub grid_step_y: f32,

    pub editor_size: Vec2,
    pub hover_point: Vec2,

    #[cfg(feature = "ron")]
    pub ron_path: Option<String>,
}

impl Default for LookupCurveEguiEditor {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            scale: Vec2::new(1.0, 1.0),

            grid_step_x: 0.1,
            grid_step_y: 0.1,

            editor_size: Vec2::ZERO,
            hover_point: Vec2::ZERO,

            #[cfg(feature = "ron")]
            ron_path: None,
        }
    }
}

impl LookupCurveEguiEditor {
    /// Constructs a [LookupCurveEguiEditor] with the supplied `path` as save path.
    #[cfg(feature = "ron")]
    pub fn with_save_path(path: String) -> Self {
        Self {
            ron_path: Some(path),
            ..Default::default()
        }
    }

    /// Constructs a [LookupCurveEguiEditor] with the viewport adjusted to fit the supplied [LookupCurve].
    pub fn fitted_to_curve(curve: &LookupCurve) -> Self {
        let mut editor = LookupCurveEguiEditor::default();
        editor.fit_to_curve(curve);
        editor
    }

    /// Fits the editor viewport to the supplied [LookupCurve] by updating scale and offset.
    pub fn fit_to_curve(&mut self, curve: &LookupCurve) {
        let knots = curve.knots();
        let (min, max) = match knots.len() {
            0 => (Vec2::ZERO, Vec2::ONE),
            1 => {
                let pos = knots[0].position;
                let padding = Vec2::splat(0.5);
                (pos - padding, pos + padding)
            }
            _ => knots
                .iter()
                .fold((Vec2::INFINITY, Vec2::NEG_INFINITY), |(min, max), knot| {
                    (min.min(knot.position), max.max(knot.position))
                }),
        };

        let diff = max - min;

        self.offset = min - 0.2 * diff;
        self.scale = diff * 1.4;
    }

    // TODO : Rename these functions and make them clearer
    // Move to a paintcontext? with access to to_screeen / to_canvas

    fn curve_to_canvas(&self, curve: Vec2) -> Pos2 {
        let canvas = (curve - self.offset) * self.editor_size / self.scale;
        Pos2::new(canvas.x, self.editor_size.y - canvas.y)
    }

    fn curve_to_canvas_vec(&self, curve: Vec2) -> emath::Vec2 {
        let canvas = curve * self.editor_size / self.scale;
        emath::Vec2::new(canvas.x, -canvas.y)
    }

    fn canvas_to_curve(&self, canvas: Pos2) -> Vec2 {
        let canvas = Vec2::new(canvas.x, self.editor_size.y - canvas.y);
        self.offset + canvas / self.editor_size * self.scale
    }

    fn canvas_to_curve_vec(&self, canvas: emath::Vec2) -> Vec2 {
        let canvas = Vec2::new(canvas.x, -canvas.y);
        canvas / self.editor_size * self.scale
    }

    /// Display the editor in a window
    ///
    /// If a `sample` is supplied, it will be displayed as a red dot on the curve.
    ///
    /// Returns `true` if the curve was changed during this update
    pub fn ui_window(
        &mut self,
        ctx: &mut egui::Context,
        id: impl std::hash::Hash,
        curve: &mut LookupCurve,
        sample: Option<f32>,
    ) -> bool {
        let mut changed = false;
        egui::Window::new(curve.name_or_default())
            .id(Id::new(id))
            .show(ctx, |ui| {
                changed = self.ui(ui, curve, sample);
            });
        changed
    }

    /// Display the editor
    ///
    /// If a `sample` is supplied, it will be displayed as a red dot on the curve.
    ///
    /// Returns `true` if the curve was changed during this update
    pub fn ui(&mut self, ui: &mut Ui, curve: &mut LookupCurve, sample: Option<f32>) -> bool {
        let mut changed = false;
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            if ui.button("Refocus curve").clicked() {
                changed = true;
                self.fit_to_curve(curve);
            }

            ui.label(format!(
                "x = {}, y = {}",
                self.hover_point.x, self.hover_point.y
            ));
        });

        #[cfg(feature = "ron")]
        if self.ron_path.is_some() && ui.button("Save").clicked() {
            if let Err(e) = curve.save_to_file(self.ron_path.as_ref().unwrap().as_str()) {
                #[cfg(feature = "bevy_app")]
                bevy_log::error!("Failed to save curve {}", e);
                #[cfg(not(feature = "bevy_app"))]
                println!("Failed to save curve {}", e);
            } else {
                #[cfg(feature = "bevy_app")]
                bevy_log::info!("Curve saved successfully.");
                #[cfg(not(feature = "bevy_app"))]
                println!("Curve saved successfully.");
            }
        }

        Frame::canvas(ui.style()).show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(
                emath::Vec2::new(ui.available_width(), ui.available_height()),
                Sense::click_and_drag(),
            );

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
                    let scroll_delta = input.raw_scroll_delta.y;
                    if scroll_delta != 0.0 {
                        self.scale *= 1.0 + -scroll_delta * 0.001;
                        // TODO: adjust offset accordingly
                    }
                });
            } else {
                self.hover_point = Vec2::ZERO;
            }

            // Panning
            if response.dragged() || response.dragged_by(egui::PointerButton::Middle) {
                self.offset -= self.canvas_to_curve_vec(response.drag_delta());
            }

            response.context_menu(|ui| {
                let menu_pos = ui.min_rect().left_top(); // hacky and not entirely correct
                if ui.button("Add knot").clicked() {
                    curve.add_knot(Knot {
                        position: self.canvas_to_curve(to_canvas.transform_pos(menu_pos)),
                        ..Default::default()
                    });
                    changed = true;
                    ui.close();
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
                            painter.add(Shape::line(
                                vec![
                                    to_screen
                                        .transform_pos(self.curve_to_canvas(prev_knot.position)),
                                    to_screen.transform_pos(self.curve_to_canvas(Vec2::new(
                                        knot.position.x,
                                        prev_knot.position.y,
                                    ))),
                                    to_screen.transform_pos(self.curve_to_canvas(knot.position)),
                                ],
                                curve_stroke,
                            ));
                        }
                        KnotInterpolation::Linear => {
                            painter.add(Shape::line(
                                vec![
                                    to_screen
                                        .transform_pos(self.curve_to_canvas(prev_knot.position)),
                                    to_screen.transform_pos(self.curve_to_canvas(knot.position)),
                                ],
                                curve_stroke,
                            ));
                        }
                        KnotInterpolation::Cubic => {
                            painter.add(CubicBezierShape::from_points_stroke(
                                prev_knot
                                    .compute_bezier_to(knot)
                                    .map(|p| to_screen.transform_pos(self.curve_to_canvas(p))),
                                false,
                                Color32::TRANSPARENT,
                                curve_stroke,
                            ));
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
                let interact_rect =
                    Rect::from_center_size(point_in_screen, emath::Vec2::splat(2.0 * knot_radius));
                let interact_id = response.id.with(knot.id);
                let interact_response =
                    ui.interact(interact_rect, interact_id, Sense::click_and_drag());

                if interact_response.dragged_by(egui::PointerButton::Primary) {
                    modified_knot = Some((
                        i,
                        Knot {
                            position: knot.position
                                + self.canvas_to_curve_vec(interact_response.drag_delta()),
                            ..*knot
                        },
                    ));
                }

                interact_response.context_menu(|ui| {
                    ui.label("Interpolation");
                    if ui
                        .radio(
                            matches!(knot.interpolation, KnotInterpolation::Constant),
                            "Constant",
                        )
                        .clicked()
                    {
                        modified_knot = Some((
                            i,
                            Knot {
                                interpolation: KnotInterpolation::Constant,
                                ..*knot
                            },
                        ));
                        ui.close();
                    }
                    if ui
                        .radio(
                            matches!(knot.interpolation, KnotInterpolation::Linear),
                            "Linear",
                        )
                        .clicked()
                    {
                        modified_knot = Some((
                            i,
                            Knot {
                                interpolation: KnotInterpolation::Linear,
                                ..*knot
                            },
                        ));
                        ui.close();
                    }
                    if ui
                        .radio(
                            matches!(knot.interpolation, KnotInterpolation::Cubic),
                            "Cubic",
                        )
                        .clicked()
                    {
                        modified_knot = Some((
                            i,
                            Knot {
                                interpolation: KnotInterpolation::Cubic,
                                ..*knot
                            },
                        ));
                        ui.close();
                    }

                    ui.label("Position");
                    ui.horizontal(|ui| {
                        ui.label("x:");
                        ui.add(
                            egui::DragValue::from_get_set(|v| match v {
                                Some(v) => {
                                    modified_knot = Some((
                                        i,
                                        Knot {
                                            position: Vec2::new(v as f32, knot.position.y),
                                            ..*knot
                                        },
                                    ));
                                    v
                                }
                                _ => knot.position.x as f64,
                            })
                            .speed(0.001),
                        );
                        ui.label("y:");
                        ui.add(
                            egui::DragValue::from_get_set(|v| match v {
                                Some(v) => {
                                    modified_knot = Some((
                                        i,
                                        Knot {
                                            position: Vec2::new(knot.position.x, v as f32),
                                            ..*knot
                                        },
                                    ));
                                    v
                                }
                                _ => knot.position.y as f64,
                            })
                            .speed(0.001),
                        );
                    });

                    ui.label("Actions");
                    if ui.button("Delete knot").clicked() {
                        deleted_knot_index = Some(i);
                        ui.close();
                    }
                });

                painter.add(Shape::circle_filled(
                    to_screen.transform_pos(self.curve_to_canvas(knot.position)),
                    3.0,
                    Color32::LIGHT_GREEN,
                ));

                // tangents
                const UNWEIGHTED_TANGENT_LEN: f32 = 60.;
                let mut tangent_ui = |side: TangentSide| {
                    let (tangent, bezier, dir) = match side {
                        TangentSide::Left => (
                            knot.left_tangent,
                            prev_knot.unwrap().compute_bezier_to(knot),
                            -1.,
                        ),
                        TangentSide::Right => (
                            knot.right_tangent,
                            knot.compute_bezier_to(next_knot.unwrap()),
                            1.,
                        ),
                    };
                    let (endpoint, intermediate) = match side {
                        TangentSide::Left => (bezier[3], bezier[2]),
                        TangentSide::Right => (bezier[0], bezier[1]),
                    };
                    let point_in_canvas = if tangent.weight.is_some() {
                        self.curve_to_canvas(intermediate)
                    } else {
                        self.curve_to_canvas(knot.position)
                            + self
                                .curve_to_canvas_vec(intermediate - knot.position)
                                .normalized()
                                * UNWEIGHTED_TANGENT_LEN
                    };

                    let point_in_screen = to_screen.transform_pos(point_in_canvas);

                    let interact_rect = Rect::from_center_size(
                        point_in_screen,
                        emath::Vec2::splat(2.0 * knot_radius),
                    );
                    let interact_id = interact_id.with(side);
                    let interact_response =
                        ui.interact(interact_rect, interact_id, Sense::click_and_drag());

                    if interact_response.dragged_by(egui::PointerButton::Primary) {
                        let mut c = self.canvas_to_curve(
                            to_canvas
                                .transform_pos(interact_response.interact_pointer_pos().unwrap()),
                        );

                        if tangent.weight.is_none() {
                            // Unweighted x is always 1/3 of dx
                            let x = (bezier[3].x - bezier[0].x) * dir / 3.;
                            let relative_c = c - endpoint;
                            c = endpoint + relative_c * (x / relative_c.x);
                        };

                        let (new_slope, new_weight) =
                            slope_weight_from_bezier(bezier[0], bezier[3], endpoint, c, dir);

                        let mut knot = knot.with_tangent_slope(side, new_slope);
                        if tangent.weight.is_some() {
                            knot = knot.with_tangent_weight(side, Some(new_weight));
                        }

                        modified_knot = Some((i, knot));
                    }

                    interact_response.context_menu(|ui| {
                        ui.label("Edit mode");
                        if ui
                            .radio(matches!(tangent.mode, TangentMode::Free), "Free")
                            .clicked()
                        {
                            modified_knot =
                                Some((i, knot.with_tangent_mode(side, TangentMode::Free)));
                            ui.close();
                        }
                        if ui
                            .radio(matches!(tangent.mode, TangentMode::Aligned), "Aligned")
                            .clicked()
                        {
                            modified_knot =
                                Some((i, knot.with_tangent_mode(side, TangentMode::Aligned)));
                            ui.close();
                        }

                        ui.label("Slope:");
                        ui.add(
                            egui::DragValue::from_get_set(|v| match v {
                                Some(v) => {
                                    modified_knot =
                                        Some((i, knot.with_tangent_slope(side, v as f32)));
                                    v
                                }
                                _ => tangent.slope as f64,
                            })
                            .speed(0.001),
                        );

                        let mut weighted = tangent.weight.is_some();
                        if ui.checkbox(&mut weighted, "Weighted").changed() {
                            if weighted && tangent.weight.is_none() {
                                modified_knot =
                                    Some((i, knot.with_tangent_weight(side, Some(1. / 3.))));
                            } else if !weighted {
                                modified_knot = Some((i, knot.with_tangent_weight(side, None)));
                            }
                        };

                        if tangent.weight.is_some() {
                            ui.horizontal(|ui| {
                                ui.label("Weight:");
                                ui.add(
                                    egui::DragValue::from_get_set(|v| match v {
                                        Some(v) => {
                                            modified_knot = Some((
                                                i,
                                                knot.with_tangent_weight(side, Some(v as f32)),
                                            ));
                                            v
                                        }
                                        _ => tangent.weight.unwrap() as f64,
                                    })
                                    .speed(0.001),
                                );
                            });
                        }
                    });

                    painter.add(Shape::dashed_line(
                        &[
                            to_screen.transform_pos(self.curve_to_canvas(knot.position)),
                            point_in_screen,
                        ],
                        Stroke::new(1.0, Color32::GRAY),
                        4.0,
                        2.0,
                    ));

                    painter.add(Shape::circle_filled(
                        point_in_screen,
                        3.0,
                        Color32::LIGHT_GRAY,
                    ));
                };

                // right tangent
                if matches!(knot.interpolation, KnotInterpolation::Cubic) && next_knot.is_some() {
                    tangent_ui(TangentSide::Right);
                }

                // left tangent
                if prev_knot.is_some()
                    && matches!(prev_knot.unwrap().interpolation, KnotInterpolation::Cubic)
                {
                    tangent_ui(TangentSide::Left);
                }
            }

            // Apply modifications
            if let Some((i, knot)) = modified_knot {
                curve.modify_knot(i, knot);
                changed = true;
            }
            if let Some(i) = deleted_knot_index {
                curve.delete_knot(i);
                changed = true;
            }

            // Sample to visualize and test find_y_given_x
            if let Some(sample) = sample {
                painter.add(Shape::circle_filled(
                    to_screen.transform_pos(
                        self.curve_to_canvas(Vec2::new(sample, curve.lookup(sample))),
                    ),
                    3.0,
                    Color32::RED,
                ));
            }
        });

        changed
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
                        to_screen.transform_pos(self.curve_to_canvas(line_to)),
                    ],
                    stroke: Stroke {
                        width: 1.0,
                        color: Color32::from_rgb(42, 42, 42),
                    },
                });

                painter.text(
                    to_screen.transform_pos(Pos2::new(
                        self.curve_to_canvas(line_from).x,
                        self.editor_size.y - 5.,
                    )),
                    egui::Align2::CENTER_BOTTOM,
                    format!("{:.1}", line_from.x),
                    egui::FontId::default(),
                    Color32::WHITE,
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
                        to_screen.transform_pos(self.curve_to_canvas(line_to)),
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
                        Color32::WHITE,
                    );
                }
            }
        }
    }
}

fn slope_weight_from_bezier(
    c0: Vec2,
    c3: Vec2,
    endpoint: Vec2,
    intermediate: Vec2,
    dir: f32,
) -> (f32, f32) {
    if c3.x == c0.x {
        return (0.0, 1. / 3.);
    }

    let dx = c3.x - c0.x;
    let weight = (intermediate.x - endpoint.x) * dir / dx;
    ((intermediate.y - endpoint.y) * dir / (dx * weight), weight)
}
