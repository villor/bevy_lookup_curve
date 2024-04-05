# Changelog

## [0.2.1] - 05-Apr-2024

### Changed
- Downgraded `bevy_egui` to 0.25 to fix problems with `bevy-inspector-egui`. Waiting for `bevy_egui` 0.27 instead, which should be supported by `bevy-inspector-egui`.

## [0.2.0] - 04-Apr-2024

This release has multiple breaking changes within the `LookupCurve` struct. If you have previously saved curves, you will need to make new ones using the new version.

### Changed
- BREAKING: `Bezier` interpolation has been replaced with `Cubic`. Inspired by other game engines. The cubic variant allows both unweighted (fast) and weighted cubic (equivalent to Bezier) interpolation. The tangents have been changed from being Bezier control points, to defining slope and optional weight.
- BREAKING: `find_y_given_x` has been renamed to `lookup`
- Removed `Knot::id` from reflect serialization. Will now be set internally when loading a scene containing curves.
- `LookupCurve` now implements `Clone`
- `max_iters` and `max_error` are now public fields, and the getters and setters have been removed.

### Added
- `LookupCache` - near constant speed for coherent lookups (like animations)
  - Use with `LookupCurve::lookup_cached`, which takes a mutable reference to the cache that can be stored on a component (or wherever you need).

### Updated
- bevy_egui to 0.26


## [0.1.0] - 23-Feb-2024
Initial release.