use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, AssetInspectorPlugin};

use bevy_lookup_curve::{
  LookupCurve,
  // Knot,
  // KnotInterpolation,
  LookupCurvePlugin,
  editor::LookupCurveEditor,
};

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(LookupCurvePlugin)

    .add_plugins(AssetInspectorPlugin::<LookupCurve>::default())

    .register_type::<LookupCurveEditorResource>()
    .add_plugins(ResourceInspectorPlugin::<LookupCurveEditorResource>::default())
    
    .add_systems(Startup, setup)
    .add_systems(Update, editor_window)
    .run();
}

#[derive(Resource, Default, Reflect)]
struct LookupCurveEditorResource {
  curve_handle: Handle<LookupCurve>,
  editor: LookupCurveEditor,
  sample: f32,
  sample_dir: f32,
  move_sample: bool,
}

fn setup(
  mut commands: Commands,
  //mut lookup_curves: ResMut<Assets<LookupCurve>>,
  assets: Res<AssetServer>,
) {
  // let handle = lookup_curves.add(LookupCurve(LookupCurve::new(vec![
  //   Knot { position: Vec2::ZERO, interpolation: KnotInterpolation::Constant, ..default() },
  //   Knot { position: Vec2::new(0.2, 0.4), interpolation: KnotInterpolation::Linear, ..default() },
  //   Knot { position: Vec2::ONE, interpolation: KnotInterpolation::Linear, ..default() }
  // ])));
  let handle = assets.load("example.curve.ron");

  commands.insert_resource(LookupCurveEditorResource {
    curve_handle: handle,
    editor: LookupCurveEditor::with_save_path("./assets/example.curve.ron".to_string()),
    sample: 0.0,
    sample_dir: 1.0,
    move_sample: true,
  });
}

fn editor_window(
  mut editor: ResMut<LookupCurveEditorResource>,
  mut contexts: EguiContexts,
  mut curves: ResMut<Assets<LookupCurve>>,
  time: Res<Time>,
) {
  
  if editor.move_sample {
    if editor.sample >= 1.5 {
      editor.sample_dir = -1.0;
    } else if editor.sample <= -0.5 {
      editor.sample_dir = 1.0;
    }
    editor.sample += time.delta_seconds() * 0.3 * editor.sample_dir;
  }

  if let Some(curve) = curves.get_mut(&editor.curve_handle) {
    egui::Window::new("Lookup curve")
      .show(contexts.ctx_mut(), |ui| {
        let sample = editor.sample;
        editor.editor.ui(ui, curve, Some(sample));
      });
  }
}
