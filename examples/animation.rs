use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin};

use bevy_lookup_curve::{
    editor::LookupCurveEguiEditor, Knot, KnotInterpolation, LookupCache, LookupCurve,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, update)
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
struct AnimationCache(LookupCache);

#[derive(Component)]
struct EditorWindow(LookupCurveEguiEditor);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_image(asset_server.load("bevy_icon.png")),
        Transform::from_xyz(0., -200., 0.).with_scale(Vec3::splat(0.5)),
        AnimateX {
            from: -400.0,
            to: 400.0,
            t: 0.0,
            dir: 1.0,
            speed: 0.3,
        },
        AnimateWithCurve(
            LookupCurve::new(vec![
                Knot {
                    position: Vec2::ZERO,
                    interpolation: KnotInterpolation::Linear,
                    ..Default::default()
                },
                Knot {
                    position: Vec2::ONE,
                    interpolation: KnotInterpolation::Linear,
                    ..default()
                },
            ])
            .with_name("Animation curve"),
        ),
        AnimationCache(LookupCache::new()),
        EditorWindow(LookupCurveEguiEditor::default()),
    ));
}

fn update(
    mut animate: Query<(
        Entity,
        &mut Transform,
        &mut AnimateX,
        &mut AnimateWithCurve,
        &mut AnimationCache,
        &mut EditorWindow,
    )>,
    mut contexts: EguiContexts,
    time: Res<Time>,
) {
    for (entity, mut transform, mut animate, mut curve, mut cache, mut editor) in animate.iter_mut()
    {
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
        transform.translation.x = animate.from
            + (animate.to - animate.from) * curve.0.lookup_cached(animate.t, &mut cache.0);

        // draw editor
        editor
            .0
            .ui_window(contexts.ctx_mut(), entity, &mut curve.0, Some(animate.t));
    }
}
