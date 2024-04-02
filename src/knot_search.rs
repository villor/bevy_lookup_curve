use crate::Knot;

/// Search algorithms for finding knots. Expects a sorted slice and searches are expected to be within knot range.
pub trait KnotSearch {
    fn search_knots_binary(&self, x: f32) -> usize;
    fn search_knots_linear(&self, x: f32) -> usize;
    fn search_knots_linear_rev(&self, x: f32) -> usize;
    fn search_knots(&self, x: f32) -> usize;
    fn search_knots_with_cache(&self, x: f32, cached_index: &mut Option<usize>) -> usize;
}

impl KnotSearch for [Knot] {
    #[inline]
    fn search_knots_binary(&self, x: f32) -> usize {
        self.partition_point(|knot| knot.position.x < x) - 1
    }

    #[inline]
    fn search_knots_linear(&self, x: f32) -> usize {
        self.iter().position(|knot| knot.position.x >= x).unwrap() - 1
    }

    #[inline]
    fn search_knots_linear_rev(&self, x: f32) -> usize {
        self.len()
            - self
                .iter()
                .rev()
                .position(|knot| knot.position.x < x)
                .unwrap()
            - 1
    }

    #[inline]
    fn search_knots(&self, x: f32) -> usize {
        self.search_knots_binary(x)
    }

    #[inline]
    fn search_knots_with_cache(&self, x: f32, cached_index: &mut Option<usize>) -> usize {
        let i = if cached_index.is_none() || cached_index.unwrap() > self.len() - 2 {
            // if the cached_index is overflowing or points to the last knot (not interpolatable), we consider the cache faulty and do a full search. This might happen if the curve is modified while the cache is being used.
            self.search_knots(x)
        } else {
            let cached_index = cached_index.unwrap();
            let cached_knot = self[cached_index];
            if x <= cached_knot.position.x {
                self[..cached_index].search_knots_linear_rev(x)
            } else {
                cached_index + self[cached_index..].search_knots_linear(x)
            }
        };

        *cached_index = Some(i);
        i
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_math::Vec2;

    fn knots() -> [Knot; 3] {
        [
            Knot {
                position: Vec2::splat(0.0),
                ..Default::default()
            },
            Knot {
                position: Vec2::splat(0.334),
                ..Default::default()
            },
            Knot {
                position: Vec2::splat(1.0),
                ..Default::default()
            },
        ]
    }

    #[test]
    fn binary_finds_knots() {
        let knots = knots();
        assert_eq!(knots.search_knots_binary(0.1), 0);
        assert_eq!(knots.search_knots_binary(0.334), 0);
        assert_eq!(knots.search_knots_binary(0.335), 1);
    }

    #[test]
    fn linear_finds_knots() {
        let knots = knots();
        assert_eq!(knots.search_knots_linear(0.1), 0);
        assert_eq!(knots.search_knots_linear(0.334), 0);
        assert_eq!(knots.search_knots_linear(0.335), 1);
    }

    #[test]
    fn linear_rev_finds_knots() {
        let knots = knots();
        assert_eq!(knots.search_knots_linear_rev(0.1), 0);
        assert_eq!(knots.search_knots_linear_rev(0.334), 0);
        assert_eq!(knots.search_knots_linear_rev(0.335), 1);
    }

    #[test]
    fn cached_finds_knots() {
        let knots = knots();
        assert_eq!(knots.search_knots_with_cache(0.1, &mut None), 0);
        assert_eq!(knots.search_knots_with_cache(0.1, &mut Some(0)), 0);
        assert_eq!(knots.search_knots_with_cache(0.1, &mut Some(1)), 0);

        assert_eq!(knots.search_knots_with_cache(0.334, &mut None), 0);
        assert_eq!(knots.search_knots_with_cache(0.334, &mut Some(0)), 0);
        assert_eq!(knots.search_knots_with_cache(0.334, &mut Some(1)), 0);

        assert_eq!(knots.search_knots_with_cache(0.335, &mut None), 1);
        assert_eq!(knots.search_knots_with_cache(0.335, &mut Some(0)), 1);
        assert_eq!(knots.search_knots_with_cache(0.335, &mut Some(1)), 1);
    }

    #[test]
    fn cached_index_is_updated() {
        let knots = knots();
        let mut cached_index = None;
        knots.search_knots_with_cache(0.5, &mut cached_index);
        assert_eq!(cached_index, Some(1));
    }

    #[test]
    fn cached_handles_index_overflow() {
        let knots = knots();
        knots.search_knots_with_cache(0.5, &mut Some(9999));
    }
}
