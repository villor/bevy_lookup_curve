//! This example demonstrates the usage of the `inspector-egui` feature.
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::{AssetInspectorPlugin, ResourceInspectorPlugin};

use bevy_lookup_curve::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(LookupCurvePlugin)
        .add_plugins(AssetInspectorPlugin::<LookupCurve>::default())
        .register_type::<LookupCurveDevState>()
        .add_plugins(ResourceInspectorPlugin::<LookupCurveDevState>::default())
        .add_systems(Startup, setup)
        .run();
}

#[derive(Resource, Default, Reflect)]
struct LookupCurveDevState {
    curve_handle: Handle<LookupCurve>,
    curve_noasset: LookupCurve,
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let handle = assets.load("example.curve.ron");

    commands.insert_resource(LookupCurveDevState {
        curve_handle: handle,
        curve_noasset: LookupCurve::default(),
    });
}
