use bevy_app::{App, Plugin};
use bevy_asset::{Assets, Handle};
use bevy_ecs::prelude::{Component, Entity, Query, ResMut, Result};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};

use super::LookupCurveEguiEditor;
use crate::LookupCurve;

pub(crate) struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, lookup_curve_editor_ui);
    }
}

#[derive(Component)]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
/// Component for convience of spawning lookup curve editor windows
///
/// Holds a `curve_handle` to the loaded lookup curve asset
pub struct LookupCurveEditor {
    pub curve_handle: Handle<LookupCurve>,
    pub egui_editor: LookupCurveEguiEditor,
    pub sample: Option<f32>,
}

impl LookupCurveEditor {
    /// Constructs a [LookupCurveEditor] with the supplied `curve_handle`.
    pub fn new(curve_handle: Handle<LookupCurve>) -> Self {
        Self {
            curve_handle,
            egui_editor: LookupCurveEguiEditor::default(),
            sample: None,
        }
    }

    /// Constructs a [LookupCurveEditor] with the supplied `curve_handle` and `path` as save path.
    pub fn with_save_path(curve_handle: Handle<LookupCurve>, path: String) -> Self {
        Self {
            egui_editor: LookupCurveEguiEditor {
                ron_path: Some(path),
                ..Default::default()
            },
            ..LookupCurveEditor::new(curve_handle)
        }
    }
}

fn lookup_curve_editor_ui(
    mut editors: Query<(Entity, &mut LookupCurveEditor)>,
    mut contexts: EguiContexts,
    mut curves: ResMut<Assets<LookupCurve>>,
) -> Result {
    for (entity, mut editor) in &mut editors {
        if let Some(curve) = curves.get_mut(&editor.curve_handle) {
            let sample = editor.sample;
            editor
                .egui_editor
                .ui_window(contexts.ctx_mut()?, entity, curve, sample);
        }
    }
    Ok(())
}
