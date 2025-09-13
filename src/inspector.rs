use std::{
    any::{Any, TypeId},
    sync::{Arc, Mutex},
};

use crate::{LookupCache, LookupCurve, editor::LookupCurveEguiEditor};
use bevy_app::{App, Plugin};
use bevy_asset::{Assets, Handle};
use bevy_inspector_egui::reflect_inspector::InspectorUi;
use bevy_inspector_egui::{
    inspector_egui_impls::InspectorEguiImpl, reflect_inspector::ProjectorReflect,
};
use bevy_reflect::{PartialReflect, TypeRegistry};

pub(crate) struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        let type_registry = app.world().resource::<bevy_ecs::prelude::AppTypeRegistry>();
        let mut type_registry = type_registry.write();

        type_registry.register::<LookupCurve>();
        add_raw::<LookupCurve>(
            &mut type_registry,
            lookup_curve_ui,
            lookup_curve_ui_readonly,
            many_unimplemented,
        );

        type_registry.register::<Handle<LookupCurve>>();
        add_raw::<Handle<LookupCurve>>(
            &mut type_registry,
            lookup_curve_handle_ui,
            lookup_curve_handle_ui_readonly,
            many_unimplemented,
        );
    }
}

type InspectorEguiImplFn =
    fn(&mut dyn Any, &mut egui::Ui, &dyn Any, egui::Id, InspectorUi<'_, '_>) -> bool;
type InspectorEguiImplFnReadonly =
    fn(&dyn Any, &mut egui::Ui, &dyn Any, egui::Id, InspectorUi<'_, '_>);
type InspectorEguiImplFnMany = for<'a> fn(
    &mut egui::Ui,
    &dyn Any,
    egui::Id,
    InspectorUi<'_, '_>,
    &mut [&mut dyn PartialReflect],
    &dyn ProjectorReflect,
) -> bool;

fn add_raw<T: 'static>(
    type_registry: &mut TypeRegistry,
    fn_mut: InspectorEguiImplFn,
    fn_readonly: InspectorEguiImplFnReadonly,
    fn_many: InspectorEguiImplFnMany,
) {
    type_registry
        .get_mut(TypeId::of::<T>())
        .unwrap_or_else(|| panic!("{} not registered", std::any::type_name::<T>()))
        .insert(InspectorEguiImpl::new(fn_mut, fn_readonly, fn_many));
}

fn many_unimplemented(
    ui: &mut egui::Ui,
    _options: &dyn Any,
    _id: egui::Id,
    _env: InspectorUi<'_, '_>,
    _values: &mut [&mut dyn PartialReflect],
    _projector: &dyn ProjectorReflect,
) -> bool {
    ui.label("LookupCurve does not support multi-editing.");
    false
}

fn lookup_curve_ui(
    curve: &mut dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    id: egui::Id,
    _: InspectorUi<'_, '_>,
) -> bool {
    let curve = curve.downcast_mut::<LookupCurve>().unwrap();
    lookup_curve_miniature_with_edit(curve, id, ui)
}

fn lookup_curve_ui_readonly(
    curve: &dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    id: egui::Id,
    _: InspectorUi<'_, '_>,
) {
    let curve = curve.downcast_ref::<LookupCurve>().unwrap();
    lookup_curve_miniature(curve, id, ui);
}

fn lookup_curve_handle_ui(
    handle: &mut dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    id: egui::Id,
    env: InspectorUi<'_, '_>,
) -> bool {
    let Some(world) = &mut env.context.world else {
        ui.label("no world in context");
        return false;
    };
    let mut curves = match world.get_resource_mut::<Assets<LookupCurve>>() {
        Ok(curves) => curves,
        Err(_) => {
            ui.label("no Assets<LookupCurve> in world");
            return false;
        }
    };

    let handle = handle.downcast_ref::<Handle<LookupCurve>>().unwrap();
    let Some(curve) = curves.get_mut(handle) else {
        ui.label("dead asset handle");
        return false;
    };

    lookup_curve_miniature_with_edit(curve, id, ui)
}

fn lookup_curve_handle_ui_readonly(
    handle: &dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    id: egui::Id,
    env: InspectorUi<'_, '_>,
) {
    let Some(world) = &mut env.context.world else {
        ui.label("no world in context");
        return;
    };
    let curves = match world.get_resource_mut::<Assets<LookupCurve>>() {
        Ok(curves) => curves,
        Err(_) => {
            ui.label("no Assets<LookupCurve> in world");
            return;
        }
    };

    let handle = handle.downcast_ref::<Handle<LookupCurve>>().unwrap();
    let Some(curve) = curves.get(handle) else {
        ui.label("dead asset handle");
        return;
    };

    lookup_curve_miniature(curve, id, ui);
}

fn lookup_curve_miniature(curve: &LookupCurve, id: egui::Id, ui: &mut egui::Ui) -> egui::Response {
    let rect = ui.available_rect_before_wrap();
    let plot_response = egui_plot::Plot::new(id.with("plot"))
        .allow_drag(false)
        .allow_scroll(false)
        .allow_zoom(false)
        .allow_boxed_zoom(false)
        .allow_double_click_reset(false)
        .show_axes([false, false])
        .show_grid([false, false])
        .show_y(false)
        .show_x(false)
        .width(rect.width())
        .height(rect.height())
        .show(ui, |plot_ui| {
            let points = if curve.knots().len() < 2 {
                egui_plot::PlotPoints::from_iter(std::iter::empty())
            } else {
                const SAMPLE_COUNT: i32 = 50;
                let min_x = curve.knots().first().unwrap().position.x;
                let max_x = curve.knots().last().unwrap().position.x;
                let length = max_x - min_x;
                let mut cache = LookupCache::new();
                egui_plot::PlotPoints::from_iter(
                    (0..SAMPLE_COUNT)
                        .map(|i| min_x + (i as f32 * length / SAMPLE_COUNT as f32))
                        .map(|x| [x as f64, curve.lookup_cached(x, &mut cache) as f64]),
                )
            };

            let line =
                egui_plot::Line::new(curve.name_or_default(), points).color(egui::Color32::GREEN);
            plot_ui.line(line);
        });

    plot_response.response
}

fn lookup_curve_miniature_with_edit(
    curve: &mut LookupCurve,
    id: egui::Id,
    ui: &mut egui::Ui,
) -> bool {
    let editor_id = id.with("editor");
    let editor_state = ui.memory(|mem| {
        mem.data
            .get_temp::<Arc<Mutex<LookupCurveEguiEditor>>>(editor_id)
    });

    let button_response = lookup_curve_miniature(curve, id, ui);
    if button_response.clicked() {
        ui.memory_mut(|mem| {
            mem.data.insert_temp(
                editor_id,
                Arc::new(Mutex::new(LookupCurveEguiEditor::fitted_to_curve(curve))),
            )
        });
    }

    let mut changed = false;
    if let Some(editor_state) = editor_state {
        let mut open = true;
        egui::Window::new(curve.name_or_default())
            .id(id.with("window"))
            .open(&mut open)
            .show(ui.ctx(), |ui| {
                changed = editor_state.lock().unwrap().ui(ui, curve, None);
            });

        if !open {
            ui.memory_mut(|mem| {
                mem.data
                    .remove::<Arc<Mutex<LookupCurveEguiEditor>>>(editor_id)
            });
        }
    }

    changed
}
