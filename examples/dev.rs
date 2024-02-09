use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, AssetInspectorPlugin};

use bevy_lookup_curve::{
  LookupCurve,
  Knot,
  KnotInterpolation,
  editor::LookupCurveEditor,
};

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)

    .init_asset::<LookupCurveAsset>()
    .add_plugins(AssetInspectorPlugin::<LookupCurveAsset>::default())

    .register_type::<LookupCurveEditorResource>()
    .add_plugins(ResourceInspectorPlugin::<LookupCurveEditorResource>::default())
    
    .add_systems(Startup, setup)
    .add_systems(Update, editor_window)
    .run();
}

#[derive(Asset, Reflect)]
struct LookupCurveAsset(LookupCurve);

#[derive(Resource, Default, Reflect)]
struct LookupCurveEditorResource {
  curve_handle: Handle<LookupCurveAsset>,
  editor: LookupCurveEditor,
  sample: f32,
  sample_dir: f32,
  move_sample: bool,
}

fn setup(
  mut commands: Commands,
  mut lookup_curves: ResMut<Assets<LookupCurveAsset>>,
) {
  let handle = lookup_curves.add(LookupCurveAsset(LookupCurve::new(vec![
    Knot { position: Vec2::ZERO, interpolation: KnotInterpolation::Constant, ..default() },
    Knot { position: Vec2::new(0.2, 0.4), interpolation: KnotInterpolation::Linear, ..default() },
    Knot { position: Vec2::ONE, interpolation: KnotInterpolation::Linear, ..default() }
  ])));

  commands.insert_resource(LookupCurveEditorResource {
    curve_handle: handle,
    editor: LookupCurveEditor::default(),
    sample: 0.0,
    sample_dir: 1.0,
    move_sample: true,
  });
}

fn editor_window(
  mut editor: ResMut<LookupCurveEditorResource>,
  mut contexts: EguiContexts,
  mut curves: ResMut<Assets<LookupCurveAsset>>,
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
        editor.editor.ui(ui, &mut curve.0, Some(sample));
      });
  }
}
