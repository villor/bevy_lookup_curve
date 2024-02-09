use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiContexts};

use bevy_lookup_curve::{
  LookupCurve,
  Knot,
  KnotInterpolation,
  editor::LookupCurveEditor,
};

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(EguiPlugin)

    .add_startup_system(setup)
    .add_system(update)
    .run();
}

#[derive(Component)]
struct AnimateX {
  from: f32,
  to: f32,
  t: f32,
  dir: f32,
  speed: f32,
}

#[derive(Component)]
struct AnimateWithCurve(LookupCurve);

#[derive(Component)]
struct EditorWindow(LookupCurveEditor);

fn setup(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
) {
  commands.spawn(Camera2dBundle::default());
  commands.spawn((
    SpriteBundle {
      texture: asset_server.load("bevy_icon.png"),
      transform: Transform::from_xyz(0., -200., 0.).with_scale(Vec3::splat(0.5)),
      ..default()
    },
    AnimateX {
      from: -400.0,
      to: 400.0,
      t: 0.0,
      dir: 1.0,
      speed: 0.3,
    },
    AnimateWithCurve(LookupCurve::new(vec![
      Knot { position: Vec2::ZERO, interpolation: KnotInterpolation::Linear, ..default() },
      Knot { position: Vec2::ONE, interpolation: KnotInterpolation::Linear, ..default() }
    ])),
    EditorWindow(LookupCurveEditor::default()),
  ));
}

fn update(
  mut animate: Query<(&mut Transform, &mut AnimateX, &mut AnimateWithCurve, &mut EditorWindow)>,
  mut contexts: EguiContexts,
  time: Res<Time>,
) {
  for (mut transform, mut animate, mut animate_curve, mut editor) in animate.iter_mut() {

    // update t
    animate.t += animate.dir * animate.speed * time.delta_seconds();
    if animate.t >= 1.0 {
      animate.dir = -1.0;
      animate.t = 1.0;
    }
    if animate.t <= 0.0 {
      animate.dir = 1.0;
      animate.t = 0.0;
    }

    // animate sprite
    transform.translation.x = animate.from + (animate.to - animate.from) * animate_curve.0.find_y_given_x(animate.t);

    // draw editor
    egui::Window::new("Lookup curve")
      .show(contexts.ctx_mut(), |ui| {
        editor.0.ui(ui, &mut animate_curve.0, Some(animate.t));
      });
  }
}
