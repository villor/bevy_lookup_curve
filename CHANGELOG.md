# Changelog

## [0.9.0] - 05-May-2025

Bevy 0.16 🥳

### Updated
- bevy to `0.16`
- bevy_egui to `0.34`
- thiserror to `2`
- bevy_inspector_egui to `0.31`
- egui_plot to `0.32`

### Migration guide
`bevy_egui` no longer recommends adding `EguiPlugin` from public plugins. `EguiPlugin` now has to be added by the consumer for the editor to work:
```rust
app.add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
```

## [0.8.0] - 17-Mar-2025

### Updated
- egui to `0.31`
- bevy_egui to `0.33`
- bevy_inspector_egui to `0.30`

#### Contributors
- [@steflon19](https://github.com/steflon19)

## [0.7.0] - 31-Jan-2025

### Added
- Implement `bevy_math::Curve<f32>` for `LookupCurve`.

### Updated
- egui to `0.30`
- bevy_inspector_egui to `0.29`

## [0.6.0] - 03-Dec-2024
Update to Bevy 0.15 🥳

### Contributors
- [@andriyDev](https://github.com/andriyDev)

## [0.5.1] - 20-Sep-2024
Fix docs.rs build

## [0.5.0] - 20-Sep-2024

### Added
- `inspector_egui` example
- `LookupCurveEguiEditor::fit_to_curve`: Adjusts the editor viewport to contain a curve.
- `LookupCurveEguiEditor::fitted_to_curve`: Same as above but as a constructor.
- A button in the egui editor to re-fit the curve

### Updated
- bevy_egui to 0.29
- bevy-inspector-egui to 0.26

### Contributors
- [@ArneCJacobs](https://github.com/ArneCJacobs)
- [@TotalKrill](https://github.com/TotalKrill)

## [0.4.1] - 02-Aug-2024

Fixed build errors on some features.

## [0.4.0] - 31-Jul-2024

Feature separation!

### Changed
- Separated the crate into smaller features, allowing the use of `bevy_lookup_curve` in projects that does not want to pull in full Bevy as a dependency. See README for a full list of features.
- BREAKING: Moved `save_lookup_curve` from the `asset` module to `LookupCurve::save_to_file`, see below.

### Added
- `load_from_file` and `save_to_file` to `LookupCurve`. Requires feature `ron` to be enabled.

## [0.3.0] - 07-Jul-2024

Bevy 0.14 🥳

### Added
- Integration with bevy-inspector-egui. Enable the feature `inspector-egui` and the curves will show up in the inspector. Click on the miniature to open the editor.
- Optional feature flag for the editor: `editor`

### Changed
- BREAKING: Removed `title` from `LookupCurveEditor`. Instead we use the (optional) name from `LookupCurve` which is shown as editor window title.
- BREAKING: `LookupCurveEguiEditor::ui_window` now requires an id (impl Hash), to ensure there are no ambiguities.
- `LookupCurveEditor::ui_window`, `LookupCurveEguiEditor::ui_window` and `LookupCurveEguiEditor::ui` now return a bool to indicate whether the curve was changed/modified during this update/render.

### Updated
- bevy to 0.14
- bevy_egui to 0.28

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