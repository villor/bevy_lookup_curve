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
- [x] LookupCurve type with modifiable knots and tangents. Three types of interpolation: Constant, Linear, and Cubic
- [x] Asset loader and save functionality
- [x] Egui based editor
- [x] Integration with [bevy-inspector-egui](https://github.com/jakobhellermann/bevy-inspector-egui) for quick and easy tweaking

https://github.com/villor/bevy_lookup_curve/assets/7102243/180aed95-ca9a-4e3b-97c4-2516055ea648

## Usage
See [examples](https://github.com/villor/bevy_lookup_curve/tree/main/examples) for now

## Feature flags
|Feature|Default|Description|
|---|---|---|
|**editor**|**Yes**|Enables the [egui](https://github.com/emilk/egui)-based editor|
|**inspector-egui**|No|Integration with [bevy-inspector-egui](https://github.com/jakobhellermann/bevy-inspector-egui)|

## Bevy support
|bevy|bevy_lookup_curve|
|---|---|
|0.14|0.3|
|0.13|0.1-0.2|

## Contributing
Contributions are welcome. Feel free to make a PR! 

## License

Dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
