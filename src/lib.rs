use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};

use bevy_app::{App, Plugin};
use bevy_asset::Asset;
use bevy_math::Vec2;
use bevy_reflect::Reflect;

pub mod asset;
pub mod editor;

/// Registers the asset loader and editor components
pub struct LookupCurvePlugin;

impl Plugin for LookupCurvePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(asset::AssetPlugin);
        app.add_plugins(editor::EditorPlugin);
    }
}

/// How a tangent behaves when a knot or its tangents are moved
#[derive(Reflect, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum TangentMode {
    /// The tangent can be freely moved without affecting the other tangent
    Free,
    /// When moving the tangent, the other tangent will be updated for a smooth curve.
    ///
    /// Both tangents need [TangentMode::Aligned] for this to apply.
    Aligned,
}

/// Tangents are used to control Bezier interpolation for [Knot]s in a [LookupCurve]
#[derive(Reflect, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Tangent {
    pub value: f32,
    pub mode: TangentMode,
}

impl Tangent {
    fn default_left() -> Self {
        Self {
            value: 0.0,
            ..Default::default()
        }
    }

    fn default_right() -> Self {
        Self {
            value: 0.0,
            ..Default::default()
        }
    }

    /// Returns a copy of self, with mode set to `mode`.
    fn with_mode(&self, mode: TangentMode) -> Self {
        Self { mode, ..*self }
    }
}

impl Default for Tangent {
    fn default() -> Self {
        Self {
            value: 0.0,
            mode: TangentMode::Aligned,
        }
    }
}

/// Interpolation used between a [Knot] the next knot
#[derive(Reflect, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum KnotInterpolation {
    Constant,
    Linear,
    Cubic,
}

#[derive(Reflect, Copy, Clone, Debug, Serialize, Deserialize)]
/// A knot in a [LookupCurve].
pub struct Knot {
    /// The position of this knot in curve space
    pub position: Vec2,

    /// Interpolation used between this and the next knot
    pub interpolation: KnotInterpolation,

    /// Left tangent relative to knot position. x above 0 will be clamped to 0
    pub left_tangent: Tangent,
    /// Right tangent relative to knot position. x below 0 will be clamped to 0
    pub right_tangent: Tangent,

    /// Identifier used by editor operations because index might change during modification
    ///
    /// There should not be any need to change this as it will be set internally.
    #[serde(skip_serializing, default = "unique_knot_id")]
    pub id: usize,
}

fn unique_knot_id() -> usize {
    static KNOT_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);
    KNOT_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

impl Knot {
    /// Returns a new knot copied from self, with the right tangent value set to `value`. This might also affect the left tangent depending on [`TangentMode`].
    fn with_right_tangent_value(&self, value: f32) -> Self {
        let mut knot = *self;
        knot.right_tangent.value = value;

        if matches!(
            (self.left_tangent.mode, self.right_tangent.mode),
            (TangentMode::Aligned, TangentMode::Aligned)
        ) {
            // TODO
            return knot;
        }

        knot
    }

    /// Returns a new knot copied from self, with the left tangent value set to `value`. This might also affect the right tangent depending on [`TangentMode`].
    fn with_left_tangent_value(&self, value: f32) -> Self {
        let mut knot = *self;
        knot.left_tangent.value = value;

        if matches!(
            (self.left_tangent.mode, self.right_tangent.mode),
            (TangentMode::Aligned, TangentMode::Aligned)
        ) {
            // TODO
            return knot;
        }

        knot
    }
}

impl Default for Knot {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            interpolation: KnotInterpolation::Linear,
            id: unique_knot_id(),
            right_tangent: Tangent::default_right(),
            left_tangent: Tangent::default_left(),
        }
    }
}

const fn max_iters_default() -> u8 {
    8
}
const fn max_error_default() -> f32 {
    1e-5
}

/// Two-dimensional spline that only allows a single y-value per x-value
#[derive(Asset, Debug, Reflect, Serialize, Deserialize)]
pub struct LookupCurve {
    knots: Vec<Knot>,
    #[serde(default = "max_iters_default")]
    max_iters: u8,
    #[serde(default = "max_error_default")]
    max_error: f32,
}

impl LookupCurve {
    pub fn new(mut knots: Vec<Knot>) -> Self {
        knots.sort_by(|a, b| {
            a.position
                .x
                .partial_cmp(&b.position.x)
                .expect("NaN is not allowed")
        });

        Self {
            knots,
            ..Default::default()
        }
    }

    /// Max iterations used for solving for y given x in cubic segments (Bezier)
    pub fn max_iters(&self) -> u8 {
        self.max_iters
    }

    /// Set max iterations used for solving for y given x in cubic segments (Bezier)
    pub fn set_max_iters(&mut self, max_iters: u8) {
        self.max_iters = max_iters;
    }

    /// Consumes the curve and returns it with max_iters set to the new value
    pub fn with_max_iters(mut self, max_iters: u8) -> Self {
        self.set_max_iters(max_iters);
        self
    }

    /// Max error allowed when solving for y given x in cubic segments (Bezier).
    pub fn max_error(&self) -> f32 {
        self.max_error
    }

    /// Set max error allowed when solving for y given x in cubic segments (Bezier).
    pub fn set_max_error(&mut self, max_error: f32) {
        self.max_error = max_error;
    }

    /// Consumes the curve and returns it with max_errors set to the new value
    pub fn with_max_error(mut self, max_error: f32) -> Self {
        self.set_max_error(max_error);
        self
    }

    /// Returns the knots in the curve as a slice
    pub fn knots(&self) -> &[Knot] {
        self.knots.as_slice()
    }

    #[inline]
    /// Given a knot index, returns the previous knot in the curve, or `None` if there is no previous knot.
    pub fn prev_knot(&self, i: usize) -> Option<&Knot> {
        if i > 0 {
            Some(&self.knots[i - 1])
        } else {
            None
        }
    }

    /// Given a knot index, returns the next knot in the curve, or `None` if there is no next knot.
    #[inline]
    pub fn next_knot(&self, i: usize) -> Option<&Knot> {
        if i < self.knots.len() - 1 {
            Some(&self.knots[i + 1])
        } else {
            None
        }
    }

    /// Adds a knot to the curve. Returns the index of the added knot.
    pub fn add_knot(&mut self, knot: Knot) -> usize {
        if self.knots.is_empty() || knot.position.x > self.knots.last().unwrap().position.x {
            self.knots.push(knot);
            return self.knots.len() - 1;
        }

        let i = self
            .knots
            .partition_point(|k| k.position.x < knot.position.x);
        self.knots.insert(i, knot);
        i
    }

    /// Modifies an existing knot in the lookup curve. Returns the new (possibly unchanged) index of the knot.
    pub fn modify_knot(&mut self, i: usize, new_value: Knot) -> usize {
        let old_value = self.knots[i];

        if old_value.position.x == new_value.position.x {
            // The knot has not been moved on the x axis, simply overwrite it
            self.knots[i] = new_value;
            return i;
        }

        // binary seach for new idx
        let new_i = self
            .knots
            .partition_point(|knot| knot.position.x < new_value.position.x);
        if new_i == i {
            // knot stays in the same spot even though position was changed, overwrite it
            self.knots[i] = new_value;
            return i;
        }

        self.knots.remove(i);

        let insert_i = if i < new_i { new_i - 1 } else { new_i };
        self.knots.insert(insert_i, new_value);

        insert_i
    }

    /// Deletes a knot given index
    pub fn delete_knot(&mut self, i: usize) {
        self.knots.remove(i);
    }

    /// Solves for y, given x
    pub fn find_y_given_x(&self, x: f32) -> f32 {
        // Return repeated constant values outside of knot range
        if self.knots.is_empty() {
            return 0.0;
        }
        if self.knots.len() == 1 || x <= self.knots[0].position.x {
            return self.knots[0].position.y;
        }
        if x >= self.knots[self.knots.len() - 1].position.x {
            return self.knots[self.knots.len() - 1].position.y;
        }

        // Find left knot
        let i = self.knots.partition_point(|knot| knot.position.x < x) - 1;
        let knot_a = self.knots[i];

        // Interpolate
        match knot_a.interpolation {
            KnotInterpolation::Constant => knot_a.position.y,
            KnotInterpolation::Linear => {
                let knot_b = &self.knots[i + 1];
                let s = (x - knot_a.position.x) / (knot_b.position.x - knot_a.position.x);
                knot_a.position.lerp(knot_b.position, s).y
            }
            KnotInterpolation::Cubic => knot_a.position.y,
            // KnotInterpolation::Bezier => {
            //     let knot_b = &self.knots[i + 1];
            //     // TODO: Optimize (we only need to calculate the coefficients when the knot is added/modified)
            //     CubicSegment::from_bezier_points([
            //         knot_a.position,
            //         knot_a.position + knot_a.right_tangent_corrected(Some(knot_b)),
            //         knot_b.position + knot_b.left_tangent_corrected(Some(&knot_a)),
            //         knot_b.position,
            //     ])
            //     .find_y_given_x(x, self.max_error, self.max_iters)
            // }
        }
    }
}

impl Default for LookupCurve {
    fn default() -> Self {
        Self {
            knots: vec![],
            max_iters: max_iters_default(),
            max_error: max_error_default(),
        }
    }
}
