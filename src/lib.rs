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
    pub position: Vec2,
    pub mode: TangentMode,
}

impl Tangent {
    fn default_left() -> Self {
        Self {
            position: Vec2::new(-0.1, 0.0),
            ..Default::default()
        }
    }

    fn default_right() -> Self {
        Self {
            position: Vec2::new(0.1, 0.0),
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
            position: Vec2::ZERO,
            mode: TangentMode::Aligned,
        }
    }
}

/// Interpolation used between a [Knot] the next knot
#[derive(Reflect, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum KnotInterpolation {
    Constant,
    Linear,
    Bezier,
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
    // TODO: Refactor repetitive *_corrected-implementations

    /// Returns the left tangent of this knot, corrected between the previous knot and this one.
    /// Ensures the curve does not ever go backwards.
    fn left_tangent_corrected(&self, prev_knot: Option<&Knot>) -> Vec2 {
        let left_tangent = self.left_tangent.position;
        if left_tangent.x >= 0.0 {
            return Vec2::new(0.0, left_tangent.y);
        }

        if let Some(prev_knot) = prev_knot {
            let min_x = prev_knot.position.x - self.position.x;
            if left_tangent.x < min_x {
                return Vec2::new(min_x, left_tangent.y * (min_x / left_tangent.x));
            }
        }

        left_tangent
    }

    /// Returns the right tangent of this knot, corrected between the next knot and this one.
    /// Ensures the curve does not ever go backwards.
    fn right_tangent_corrected(&self, next_knot: Option<&Knot>) -> Vec2 {
        let right_tangent = self.right_tangent.position;
        if right_tangent.x <= 0.0 {
            return Vec2::new(0.0, right_tangent.y);
        }

        if let Some(next_knot) = next_knot {
            let max_x = next_knot.position.x - self.position.x;
            if right_tangent.x > max_x {
                return Vec2::new(max_x, right_tangent.y * (max_x / right_tangent.x));
            }
        }

        right_tangent
    }

    /// Returns a new knot copied from self, with the right tangent moved to `position`. This might also affect the left tangent depending on [`TangentMode`].
    fn with_right_tangent_position(&self, position: Vec2) -> Self {
        let mut knot = *self;
        knot.right_tangent.position = position;

        if matches!(
            (self.left_tangent.mode, self.right_tangent.mode),
            (TangentMode::Aligned, TangentMode::Aligned)
        ) {
            knot.left_tangent.position =
                -position.normalize_or_zero() * knot.left_tangent.position.length();
        }

        knot
    }

    /// Returns a new knot copied from self, with the left tangent moved to `position`. This might also affect the right tangent depending on [`TangentMode`].
    fn with_left_tangent_position(&self, position: Vec2) -> Self {
        let mut knot = *self;
        knot.left_tangent.position = position;

        if matches!(
            (self.left_tangent.mode, self.right_tangent.mode),
            (TangentMode::Aligned, TangentMode::Aligned)
        ) {
            knot.right_tangent.position =
                -position.normalize_or_zero() * knot.right_tangent.position.length();
        }

        knot
    }
}

impl Default for Knot {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            interpolation: KnotInterpolation::Bezier,
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
            KnotInterpolation::Bezier => {
                let knot_b = &self.knots[i + 1];
                // TODO: Optimize (we only need to calculate the coefficients when the knot is added/modified)
                CubicSegment::from_bezier_points([
                    knot_a.position,
                    knot_a.position + knot_a.right_tangent_corrected(Some(knot_b)),
                    knot_b.position + knot_b.left_tangent_corrected(Some(&knot_a)),
                    knot_b.position,
                ])
                .find_y_given_x(x, self.max_error, self.max_iters)
            }
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

/// Mostly a copy of code from https://github.com/bevyengine/bevy/blob/main/crates/bevy_math/src/cubic_splines.rs
///
/// Copied because the cubic_splines module does not exactly fit the API we need:
/// 1. Allow constructing a single CubicSegment from bezier points (without allocating a CubicCurve, and without restricting c0 and c1 to 0 and 1)
/// 2. find_y_given_x needs to be accessible
/// 3. max_iters and max_error should be configurable
#[derive(Clone, Debug, Default, PartialEq)]
struct CubicSegment {
    coeff: [Vec2; 4],
}

impl CubicSegment {
    /// Instantaneous position of a point at parametric value `t`.
    #[inline]
    fn position(&self, t: f32) -> Vec2 {
        let [a, b, c, d] = self.coeff;
        a + b * t + c * t.powi(2) + d * t.powi(3)
    }

    /// Instantaneous velocity of a point at parametric value `t`.
    #[inline]
    fn velocity(&self, t: f32) -> Vec2 {
        let [_, b, c, d] = self.coeff;
        b + c * 2.0 * t + d * 3.0 * t.powi(2)
    }

    #[inline]
    fn find_y_given_x(&self, x: f32, max_error: f32, max_iters: u8) -> f32 {
        let mut t_guess = x;
        let mut pos_guess = Vec2::ZERO;
        for _ in 0..max_iters {
            pos_guess = self.position(t_guess);
            let error = pos_guess.x - x;
            if error.abs() <= max_error {
                break;
            }
            // Using Newton's method, use the tangent line to estimate a better guess value.
            let slope = self.velocity(t_guess).x; // dx/dt
            t_guess -= error / slope;
        }
        pos_guess.y
    }

    #[inline]
    fn from_bezier_points(control_points: [Vec2; 4]) -> CubicSegment {
        let char_matrix = [
            [1., 0., 0., 0.],
            [-3., 3., 0., 0.],
            [3., -6., 3., 0.],
            [-1., 3., -3., 1.],
        ];

        Self::coefficients(control_points, 1.0, char_matrix)
    }

    #[inline]
    fn coefficients(p: [Vec2; 4], multiplier: f32, char_matrix: [[f32; 4]; 4]) -> CubicSegment {
        let [c0, c1, c2, c3] = char_matrix;
        // These are the polynomial coefficients, computed by multiplying the characteristic
        // matrix by the point matrix.
        let mut coeff = [
            p[0] * c0[0] + p[1] * c0[1] + p[2] * c0[2] + p[3] * c0[3],
            p[0] * c1[0] + p[1] * c1[1] + p[2] * c1[2] + p[3] * c1[3],
            p[0] * c2[0] + p[1] * c2[1] + p[2] * c2[2] + p[3] * c2[3],
            p[0] * c3[0] + p[1] * c3[1] + p[2] * c3[2] + p[3] * c3[3],
        ];
        coeff.iter_mut().for_each(|c| *c *= multiplier);
        CubicSegment { coeff }
    }
}
