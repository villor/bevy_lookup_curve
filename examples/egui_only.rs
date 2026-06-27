//! Demonstrates using bevy_lookup_curve with just egui, no bevy app.
//!
//! This example has no dependencies on bevy except `bevy_math`.
use eframe::egui;

use bevy_lookup_curve::prelude::*;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 400.0]),
        ..Default::default()
    };

    let path = "./assets/example.curve.ron";
    let lookup_curve = LookupCurve::load_from_file(path).expect("Failed to load curve");

    eframe::run_native(
        "Lookup Curve (egui only example)",
        options,
        Box::new(|_| {
            Ok(Box::new(MyApp {
                lookup_curve,
                editor: LookupCurveEguiEditor::with_save_path(path.to_string()),
            }))
        }),
    )
}

struct MyApp {
    lookup_curve: LookupCurve,
    editor: LookupCurveEguiEditor,
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .show_inside(ui, |ui| self.editor.ui(ui, &mut self.lookup_curve, None));
    }
}
