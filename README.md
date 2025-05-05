# bevy_lookup_curve 📈

[<img alt="github" src="https://img.shields.io/badge/github-villor/bevy_lookup_curve-8da0cb?logo=github" height="20">](https://github.com/villor/bevy_lookup_curve)
[![Latest version](https://img.shields.io/crates/v/bevy_lookup_curve
)](https://crates.io/crates/bevy_lookup_curve)
[![Documentation](https://docs.rs/bevy_lookup_curve/badge.svg)](https://docs.rs/bevy_lookup_curve)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/emilk/egui/blob/master/LICENSE-MIT)
[![Apache](https://img.shields.io/badge/license-Apache-blue.svg)](https://github.com/emilk/egui/blob/master/LICENSE-APACHE)

Editable lookup curve for Bevy that can be used for many things, for example:
- Animation
- Gameplay progressiom (control different aspects over time or other variables)
- Physics (for example: tweakable feel on a character controller)
- Probability control (for item drops etc)
- Shaders
- ... just about anything where you need a formula (x -> y) that you can fine tune, with a GUI instead of diving into math

If you have used AnimationCurve in Unity, this would be an attempt at something similar for Bevy.

## Features
- [x] `LookupCurve` type with modifiable knots and tangents. Three types of interpolation: Constant, Linear, and Cubic
- [x] `LookupCurve` implements `bevy_math::Curve<f32>` to fit into the ecosystem, giving access to resampling and other conveniences.
- [x] Asset loader and save functionality
- [x] `egui`-based editor
- [x] Integration with [bevy-inspector-egui](https://github.com/jakobhellermann/bevy-inspector-egui) for quick and easy tweaking

https://github.com/villor/bevy_lookup_curve/assets/7102243/180aed95-ca9a-4e3b-97c4-2516055ea648

## Usage
See [examples](https://github.com/villor/bevy_lookup_curve/tree/main/examples) for now

## Feature flags
|Feature|Default|Description|
|---|---|---|
|**serialize**|**Yes**|Enable serde serialization/deserialization for the LookupCurve|
|**ron**|**Yes**|Enable loading/saving the curve as a ron file|
|**bevy_reflect**|**Yes**|Implement Reflect on most types in the crate|
|**bevy_asset**|**Yes**|Implement AssetLoader for LookupCurve|
|**editor_egui**|**Yes**|Enables the [egui](https://github.com/emilk/egui)-based editor|
|**editor_bevy**|**Yes**|ECS component for convenient spawning of editor windows inside Bevy|
|**inspector-egui**|No|Integration with [bevy-inspector-egui](https://github.com/jakobhellermann/bevy-inspector-egui)|

## Bevy support
|bevy|bevy_lookup_curve|
|---|---|
|0.16|0.9|
|0.15|0.6-0.8|
|0.14|0.3-0.5|
|0.13|0.1-0.2|

## Using without Bevy
This crate can be used without Bevy as well (except for `bevy_math` which is a core dependency).

Just set `default-features = false`. And enable `serialize`, `ron`, and/or `editor_egui` if needed.

See the `egui_only` example. It can also be used as a standalone curve editor.

## Contributing
Contributions are welcome. Feel free to make a PR!

## License

Dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
