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

  //#[reflect(ignore)]
  //bezier: Bezier<Vec2>,
  // #[reflect(ignore)]
  // computed_curve: CubicCurve<Vec2>,
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

  // fn linear(start: Vec2, end: Vec2) {
  //   Self {
  //     keys: vec![
  //       Key { position: start, interpolation: KeyInterpolation::Linear, left_tangent: Vec2::ZERO, right_tangent: Vec2::ZERO },
  //       Key { position: end, interpolation: KeyInterpolation::Linear }
  //     ]
  //   }
  // }

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
    let new_i = self.find_key_index_given_x(new_value.position.x);
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

  fn find_key_index_given_x(&self, x: f32) -> usize {
    let result = self.keys.binary_search_by(|key|
      key.position.x.partial_cmp(&x).expect("NaN is not allowed")
    );
    match result {
      Ok(i) => i,
      Err(i) => i,
    }
  }

  //pub fn move_key(&mut self, )

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
    // TODO: Optimize with binary search?
    let (i, key_a) = self.keys
      .iter()
      .enumerate()
      .rev()
      .find(|(_, k)| x >= k.position.x)
      .unwrap();

    match key_a.interpolation {
      KeyInterpolation::Constant => key_a.position.y,
      KeyInterpolation::Linear => {
        let key_b = &self.keys[i+1];
        let s = (x - key_a.position.x) / (key_b.position.x - key_a.position.x);
        key_a.position.lerp(key_b.position, s).y
      },
      KeyInterpolation::Bezier => {
        key_a.position.y //todo
      }
    }

    // binary search for the key
    // match interpolation
    //   Constant => key.postition.y,
    //   Linear | Bezier => {
    //     get next key
    //     calculate local x
    //     match interpolation
    //       Linear => lerp
    //       Bezier => {
    //         constrain tangents to make sure there are no loops
    //         convert to cubicsegment<Vec2> using new_bezier
    //         return segment.ease()
    //       }
    //   
  }
}


