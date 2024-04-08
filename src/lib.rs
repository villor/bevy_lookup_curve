use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};

use bevy_app::{App, Plugin};
use bevy_asset::Asset;
use bevy_math::Vec2;
use bevy_reflect::Reflect;

pub mod asset;

pub mod knot_search;

#[cfg(feature = "editor")]
pub mod editor;

#[cfg(feature = "inspector-egui")]
mod inspector;

use knot_search::KnotSearch;

/// Registers the asset loader and editor components
pub struct LookupCurvePlugin;

impl Plugin for LookupCurvePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(asset::AssetPlugin);
        #[cfg(feature = "editor")]
        app.add_plugins(editor::EditorPlugin);
        #[cfg(feature = "inspector-egui")]
        app.add_plugins(inspector::InspectorPlugin);
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

/// Tangents are used to control cubic interpolation for [Knot]s in a [LookupCurve]
#[derive(Reflect, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Tangent {
    pub slope: f32,
    pub mode: TangentMode,
    pub weight: Option<f32>,
}

impl Tangent {
    fn default_left() -> Self {
        Self {
            slope: 0.0,
            ..Default::default()
        }
    }

    fn default_right() -> Self {
        Self {
            slope: 0.0,
            ..Default::default()
        }
    }
}

impl Default for Tangent {
    fn default() -> Self {
        Self {
            slope: 0.0,
            mode: TangentMode::Aligned,
            weight: None,
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

    /// Left tangent defining in slope and weight
    pub left_tangent: Tangent,
    /// Right tangent defining out slope and weight
    pub right_tangent: Tangent,

    /// Identifier used by editor operations because index might change during modification
    ///
    /// There should not be any need to change this as it will be set internally.
    #[serde(skip_serializing, default = "unique_knot_id")]
    #[reflect(skip_serializing, default = "unique_knot_id")]
    pub id: usize,
}

fn unique_knot_id() -> usize {
    static KNOT_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);
    KNOT_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Copy, Clone, Hash)]
enum TangentSide {
    Left,
    Right,
}

impl Knot {
    /// Returns a new knot copied from self, with the tangent slope decided by `side` set to `slope`. This might also affect the other tangent depending on [`TangentMode`].
    fn with_tangent_slope(&self, side: TangentSide, slope: f32) -> Self {
        let mut knot = *self;

        let aligned = matches!(
            (self.left_tangent.mode, self.right_tangent.mode),
            (TangentMode::Aligned, TangentMode::Aligned)
        );

        if matches!(side, TangentSide::Left) || aligned {
            knot.left_tangent.slope = slope;
        }
        if matches!(side, TangentSide::Right) || aligned {
            knot.right_tangent.slope = slope;
        }

        knot
    }

    /// Returns a new knot copied from self, with the tangent mode decided by `side` set to `mode`.
    fn with_tangent_mode(&self, side: TangentSide, mode: TangentMode) -> Self {
        let mut knot = *self;
        match side {
            TangentSide::Left => knot.left_tangent.mode = mode,
            TangentSide::Right => knot.right_tangent.mode = mode,
        }
        knot
    }

    /// Returns a new knot copied from self, with the tangent weight decided by `side` set to `weight`. Weights will be clamped between 0 and 1.
    fn with_tangent_weight(&self, side: TangentSide, weight: Option<f32>) -> Self {
        let mut knot = *self;
        let weight = weight.map(|w| w.clamp(0.0, 1.0));
        match side {
            TangentSide::Left => knot.left_tangent.weight = weight,
            TangentSide::Right => knot.right_tangent.weight = weight,
        }
        knot
    }

    #[inline]
    fn compute_bezier_to(&self, knot_b: &Knot) -> [Vec2; 4] {
        let slope_a = self.right_tangent.slope;
        let weight_a = self.right_tangent.weight.unwrap_or(1. / 3.);
        let slope_b = knot_b.left_tangent.slope;
        let weight_b = knot_b.left_tangent.weight.unwrap_or(1. / 3.);
        let dx = knot_b.position.x - self.position.x;
        [
            self.position,
            Vec2::new(
                self.position.x + weight_a * dx,
                self.position.y + weight_a * slope_a * dx,
            ),
            Vec2::new(
                knot_b.position.x - weight_b * dx,
                knot_b.position.y - weight_b * slope_b * dx,
            ),
            knot_b.position,
        ]
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

/// Cache to speed up coherent lookups, see [LookupCurve::lookup_cached]
#[derive(Reflect, Debug, Clone, Default)]
pub struct LookupCache {
    last_knot_index: Option<usize>,
}

impl LookupCache {
    pub fn new() -> Self {
        Self::default()
    }
}

const fn max_iters_default() -> u8 {
    20
}
const fn max_error_default() -> f32 {
    1e-5
}

/// Two-dimensional spline that only allows a single y-value per x-value
#[derive(Asset, Clone, Debug, Reflect, Serialize, Deserialize)]
pub struct LookupCurve {
    knots: Vec<Knot>,

    /// Max number of iterations used for Newton-Rhapson iteration in weighted cubic segments
    #[serde(default = "max_iters_default")]
    #[reflect(default = "max_iters_default")]
    pub max_iters: u8,
    /// Max error allowed before breaking Newton-Rhapson iteration in weighted cubic segments
    #[serde(default = "max_error_default")]
    #[reflect(default = "max_error_default")]
    pub max_error: f32,

    pub name: Option<String>,
}

impl Default for LookupCurve {
    fn default() -> Self {
        Self {
            knots: vec![],
            max_iters: max_iters_default(),
            max_error: max_error_default(),
            name: None,
        }
    }
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

    /// Consumes the curve and returns it with max_iters set to the new value
    pub fn with_max_iters(mut self, max_iters: u8) -> Self {
        self.max_iters = max_iters;
        self
    }

    /// Consumes the curve and returns it with max_errors set to the new value
    pub fn with_max_error(mut self, max_error: f32) -> Self {
        self.max_error = max_error;
        self
    }

    /// Consumes the curve and returns it with name set
    pub fn with_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    pub(crate) fn name_or_default(&self) -> &str {
        self.name.as_deref().unwrap_or("Unnamed lookup curve")
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

    /// Find y for given x on the curve
    #[inline]
    pub fn lookup(&self, x: f32) -> f32 {
        self.lookup_internal(x, None)
    }

    /// Find y for given x on the curve, with a LookupCache. Can speed up coherent lookups, but might slow down random lookups.
    #[inline]
    pub fn lookup_cached(&self, x: f32, cache: &mut LookupCache) -> f32 {
        self.lookup_internal(x, Some(cache))
    }

    #[inline]
    fn lookup_internal(&self, x: f32, cache: Option<&mut LookupCache>) -> f32 {
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
        let i = if let Some(cache) = cache {
            self.knots
                .search_knots_with_cache(x, &mut cache.last_knot_index)
        } else {
            self.knots.search_knots(x)
        };
        let knot_a = self.knots[i];

        // Interpolate
        match knot_a.interpolation {
            KnotInterpolation::Constant => knot_a.position.y,
            KnotInterpolation::Linear => {
                let knot_b = &self.knots[i + 1];
                let s = (x - knot_a.position.x) / (knot_b.position.x - knot_a.position.x);
                knot_a.position.lerp(knot_b.position, s).y
            }
            KnotInterpolation::Cubic => {
                let knot_b = &self.knots[i + 1];
                if knot_a.right_tangent.weight.is_some() || knot_b.left_tangent.weight.is_some() {
                    weighted_cubic_interp(&knot_a, knot_b, x, self.max_error, self.max_iters)
                } else {
                    unweighted_cubic_interp(&knot_a, knot_b, x)
                }
            }
        }
    }
}

#[inline]
fn unweighted_cubic_interp(knot_a: &Knot, knot_b: &Knot, x: f32) -> f32 {
    let x = (x - knot_a.position.x) / (knot_b.position.x - knot_a.position.x);
    let dx = knot_b.position.x - knot_a.position.x;
    let m0 = knot_a.right_tangent.slope * dx;
    let m1 = knot_b.left_tangent.slope * dx;

    let x2 = x * x;
    let x3 = x2 * x;

    let a = 2. * x3 - 3. * x2 + 1.;
    let b = x3 - 2. * x2 + x;
    let c = x3 - x2;
    let d = -2. * x3 + 3. * x2;

    a * knot_a.position.y + b * m0 + c * m1 + d * knot_b.position.y
}

#[inline]
fn weighted_cubic_interp(
    knot_a: &Knot,
    knot_b: &Knot,
    x: f32,
    max_error: f32,
    max_iters: u8,
) -> f32 {
    CubicSegment::from_bezier_points(knot_a.compute_bezier_to(knot_b))
        .find_y_given_x(x, max_error, max_iters)
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
