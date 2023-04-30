use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_lookup_curve::{
  LookupCurve,
  Key,
  KeyInterpolation,
  editor::LookupCurveEditor,
};

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(EguiPlugin)
    .add_asset::<LookupCurve>()
    .add_startup_system(curve_test_setup)
    .add_system(curve_test_window)
    .run();
}

#[derive(Resource)]
struct LookupCurveEditorResource {
  curve_handle: Handle<LookupCurve>,
  editor: LookupCurveEditor,
}

fn curve_test_setup(
  mut commands: Commands,
  mut lookup_curves: ResMut<Assets<LookupCurve>>,
) {
  let handle = lookup_curves.add(LookupCurve {
    keys: vec![
      Key { position: Vec2::ZERO, interpolation: KeyInterpolation::Constant },
      Key { position: Vec2::new(0.2, 0.4), interpolation: KeyInterpolation::Linear },
      Key { position: Vec2::ONE, interpolation: KeyInterpolation::Linear }
    ]
  });

  commands.insert_resource(LookupCurveEditorResource {
    curve_handle: handle,
    editor: LookupCurveEditor::default()
  });
}

fn curve_test_window(
  mut editor: ResMut<LookupCurveEditorResource>,
  mut contexts: EguiContexts,
  mut curves: ResMut<Assets<LookupCurve>>
) {
  if let Some(curve) = curves.get_mut(&editor.curve_handle) {
    egui::Window::new("Lookup curve")
      .show(contexts.ctx_mut(), |ui| {
        editor.editor.ui(ui, std::iter::once(curve));
      });
  }
}
