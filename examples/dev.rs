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
        .add_systems(Update, move_sample)
        .run();
}

#[derive(Resource, Default, Reflect)]
struct LookupCurveDevState {
    curve_handle: Handle<LookupCurve>,
    curve_noasset: LookupCurve,
    sample_dir: f32,
    sample: f32,
    move_sample: bool,
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let handle = assets.load("example.curve.ron");

    commands.spawn(LookupCurveEditor {
        sample: Some(0.0),
        ..LookupCurveEditor::with_save_path(
            handle.clone(),
            "./assets/example.curve.ron".to_string(),
        )
    });

    commands.insert_resource(LookupCurveDevState {
        curve_handle: handle,
        curve_noasset: LookupCurve::default().with_name("Not asset"),
        sample_dir: 1.0,
        sample: 0.0,
        move_sample: true,
    });
}

fn move_sample(
    mut dev_state: ResMut<LookupCurveDevState>,
    mut editor: Query<&mut LookupCurveEditor>,
    time: Res<Time>,
) {
    if let Ok(mut editor) = editor.single_mut() {
        if dev_state.move_sample {
            if dev_state.sample >= 1.5 {
                dev_state.sample_dir = -1.0;
            } else if dev_state.sample <= -0.5 {
                dev_state.sample_dir = 1.0;
            }
            dev_state.sample += time.delta_secs() * 0.3 * dev_state.sample_dir;
        }
        editor.sample = Some(dev_state.sample)
    }
}
