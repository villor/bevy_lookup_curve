# bevy_lookup_curve

Editable lookup curve for Bevy that can be used for animations or other gameplay elements where user-adjustable "feel" is needed.

If you have used AnimationCurve in Unity, this would be an attempt at something similar for Bevy.

## Features
* LookupCurve type with modifiable knots and tangents. Three types of interpolation: Constant, Linear, and Bezier
* Egui based editor
* Asset loader and save functionality 

## Todo
- [ ] LookupCurve: Optimize bezier (precompute CubicSegments)
- [ ] Editor: Clean up code
- [ ] Editor: Snap to grid
- [ ] Editor: Adaptive grid size
- [ ] Editor: Natural zoom
- [ ] Editor: Show/hide knots/tangents
- [ ] bevy_inspector_egui support
- [ ] Crate features: 'editor' (if no editor is needed, we dont need deps for bevy_egui etc), 'assets' (if curve is not loaded as asset we don't need bevy_asset, serde, ron, thiserror etc)
- [ ] LookupTable generated from pre-sampling a LookupCurve (for performance). With lerp

## Contributing
Contributions are welcome. Feel free to make a PR! 

## License

Dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))