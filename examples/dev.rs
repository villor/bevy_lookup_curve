use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, AssetInspectorPlugin};

use bevy_lookup_curve::{
  LookupCurve,
  Key,
  KeyInterpolation,
  editor::LookupCurveEditor,
};

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)

    .add_asset::<LookupCurve>()
    .add_plugin(AssetInspectorPlugin::<LookupCurve>::default())

    .register_type::<LookupCurveEditorResource>()
    .add_plugin(ResourceInspectorPlugin::<LookupCurveEditorResource>::default())
    
    .add_startup_system(setup)
    .add_system(editor_window)
    .run();
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
struct LookupCurveEditorResource {
  curve_handle: Handle<LookupCurve>,
  editor: LookupCurveEditor,
  sample: f32,
  sample_dir: f32,
  move_sample: bool,
}

fn setup(
  mut commands: Commands,
  mut lookup_curves: ResMut<Assets<LookupCurve>>,
) {
  let handle = lookup_curves.add(LookupCurve::new(vec![
    Key { id: 0, position: Vec2::ZERO, interpolation: KeyInterpolation::Constant, ..default() },
    Key { id: 1, position: Vec2::new(0.2, 0.4), interpolation: KeyInterpolation::Linear, ..default() },
    Key { id: 2, position: Vec2::ONE, interpolation: KeyInterpolation::Linear, ..default() }
  ]));

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
        editor.editor.ui(ui, std::iter::once(curve), Some(sample));
      });
  }
}
