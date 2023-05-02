use bevy_math::Vec2;
use bevy_reflect::{Reflect, FromReflect, TypeUuid};

pub mod editor;

#[derive(Reflect, FromReflect, Copy, Clone, Debug)]
pub enum KeyInterpolation {
  Constant,
  Linear,
  Bezier,
}

#[derive(Reflect, FromReflect, Copy, Clone, Debug)]
pub struct Key {
  pub position: Vec2,
  /// Interpolation used between this and the next key
  pub interpolation: KeyInterpolation,

  pub id: usize,
  pub left_tangent: Vec2,
  pub right_tangent: Vec2,
}

impl Default for Key {
  fn default() -> Self {
    Self {
      position: Vec2::ZERO,
      interpolation: KeyInterpolation::Linear,
      id: 0,
      right_tangent: Vec2::new(0.1, 0.0),
      left_tangent: Vec2::new(-0.1, 0.0),
    }
  }
}

/// Two-dimensional spline that only allows a single y-value per x-value
#[derive(Debug, TypeUuid, Reflect, FromReflect)]
#[uuid = "3219b5f0-fff6-42fd-9fc8-fd98ff8dae35"]
pub struct LookupCurve {
  keys: Vec<Key>,
}

impl LookupCurve {
  pub fn new(mut keys: Vec<Key>) -> Self {
    keys.sort_by(|a, b|
      a.position.x
        .partial_cmp(&b.position.x)
        .expect("NaN is not allowed")
    );
    
    Self {
      keys,
    }
  }

  pub fn keys(&self) -> &[Key] {
    self.keys.as_slice()
  }

  /// Modifies an existing key in the lookup curve. Returns the new (possibly unchanged) index of the key.
  fn modify_key(&mut self, i: usize, new_value: &Key) -> usize {
    let old_value = self.keys[i];
    if old_value.position == new_value.position {
      // The key has not been moved, simply overwrite it
      self.keys[i] = *new_value;
      return i;
    }

    // binary seach for new idx
    let new_i = self.keys.partition_point(|key| key.position.x < new_value.position.x);
    if new_i == i {
      // Key stays in the same spot even though position was changed, overwrite it
      self.keys[i] = *new_value;
      return i;
    }

    self.keys.remove(i);

    let insert_i = if i < new_i { new_i - 1 } else { new_i };
    self.keys.insert(insert_i, *new_value);

    insert_i
  }

  /// Find y given x
  pub fn find_y_given_x(&self, x: f32) -> f32 {
    // Return repeated constant values outside of key range
    if self.keys.is_empty() {
      return 0.0;
    }
    if self.keys.len() == 1 || x <= self.keys[0].position.x {
      return self.keys[0].position.y;
    }
    if x >= self.keys[self.keys.len() - 1].position.x {
      return self.keys[self.keys.len() - 1].position.y;
    }

    // Find left key
    let i = self.keys.partition_point(|key| key.position.x < x) - 1;
    let key_a = self.keys[i];

    // Interpolate
    match key_a.interpolation {
      KeyInterpolation::Constant => key_a.position.y,
      KeyInterpolation::Linear => {
        let key_b = &self.keys[i+1];
        let s = (x - key_a.position.x) / (key_b.position.x - key_a.position.x);
        key_a.position.lerp(key_b.position, s).y
      },
      KeyInterpolation::Bezier => {
        let key_b = &self.keys[i+1];
        // TODO: Optimize (we only need to calculate the coefficients when the key is added/modified)
        CubicSegment::from_bezier_points([
          key_a.position,
          key_a.position + key_a.right_tangent,
          key_b.position + key_b.left_tangent,
          key_b.position,
        ]).find_y_given_x(x)
      }
    }
  }
}

/// Mostly a copy of code from https://github.com/bevyengine/bevy/blob/main/crates/bevy_math/src/cubic_splines.rs
/// 
/// Copied because the cubic_splines module does not exactly fit the API we need:
/// 1. Allow constructing a single CubicSegment from bezier points (without allocating a cubiccurve and without restricting c0 and c1 to 0 and 1)
/// 2. find_y_given_x needs to be accessible
#[derive(Clone, Debug, Default, PartialEq)]
struct CubicSegment{
  coeff: [Vec2; 4],
}

impl CubicSegment {
  /// Instantaneous position of a point at parametric value `t`.
  #[inline]
  pub fn position(&self, t: f32) -> Vec2 {
    let [a, b, c, d] = self.coeff;
    a + b * t + c * t.powi(2) + d * t.powi(3)
  }

  /// Instantaneous velocity of a point at parametric value `t`.
  #[inline]
  pub fn velocity(&self, t: f32) -> Vec2 {
    let [_, b, c, d] = self.coeff;
    b + c * 2.0 * t + d * 3.0 * t.powi(2)
  }

  #[inline]
  fn find_y_given_x(&self, x: f32) -> f32 {
    const MAX_ERROR: f32 = 1e-5;
    const MAX_ITERS: u8 = 8;
  
    let mut t_guess = x;
    let mut pos_guess = Vec2::ZERO;
    for _ in 0..MAX_ITERS {
      pos_guess = self.position(t_guess);
      let error = pos_guess.x - x;
      if error.abs() <= MAX_ERROR {
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
