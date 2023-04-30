// fn lookup_curve_ui(
//   value: &mut dyn Any,
//   ui: &mut egui::Ui,
//   _: &dyn Any,
//   _: egui::Id,
//   _: InspectorUi<'_, '_>,
// ) -> bool {
//   let lookup_curve = value.downcast_mut::<LookupCurve>().unwrap();
//   lookup_curve_miniature(lookup_curve, ui);
//   false
// }

// fn lookup_curve_ui_readonly(
//   value: &dyn Any,
//   ui: &mut egui::Ui,
//   _: &dyn Any,
//   _: egui::Id,
//   _: InspectorUi<'_, '_>,
// ) {
//   let lookup_curve = value.downcast_ref::<LookupCurve>().unwrap();
//   lookup_curve_miniature(lookup_curve, ui);
// }

// fn lookup_curve_ui_many(
//   ui: &mut egui::Ui,
//   _options: &dyn Any,
//   _id: egui::Id,
//   _env: InspectorUi<'_, '_>,
//   _values: &mut [&mut dyn Reflect],
//   _projector: &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
// ) -> bool {
//   ui.label("LookupCurve doesn't support multi-editing");
//   false
// }

// fn lookup_curve_miniature(
//   curve: &LookupCurve,
//   ui: &mut egui::Ui, 
// ) {
//   // let computed_curve = curve.bezier.to_curve();

//   // let points: egui::plot::PlotPoints = computed_curve
//   //   .iter_positions(100)
//   //   .map(|p| [p.x as f64, p.y as f64])
//   //   .collect();

  
//   // let line = egui::plot::Line::new(points);
//   // egui::plot::Plot::new("my_plot")
//   //   .allow_drag(false)
//   //   .allow_scroll(false)
//   //   .allow_zoom(false)
//   //   .allow_boxed_zoom(false)
//   //   .show_axes([false, false])
//   //   .show(ui, |plot_ui| {
//   //     plot_ui.line(line);
//   //   });
// }