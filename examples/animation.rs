use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass};

use bevy_lookup_curve::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, animate)
        .add_systems(EguiPrimaryContextPass, editor_ui)
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

fn animate(
    mut animate: Query<(
        &mut Transform,
        &mut AnimateX,
        &AnimateWithCurve,
        &mut AnimationCache,
    )>,
    time: Res<Time>,
) {
    for (mut transform, mut animate, curve, mut cache) in animate.iter_mut() {
        // update t
        animate.t += animate.dir * animate.speed * time.delta_secs();
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
    }
}

fn editor_ui(
    mut animate: Query<(Entity, &AnimateX, &mut AnimateWithCurve, &mut EditorWindow)>,
    mut contexts: EguiContexts,
) -> Result {
    for (entity, animate, mut curve, mut editor) in animate.iter_mut() {
        // draw editor
        editor
            .0
            .ui_window(contexts.ctx_mut()?, entity, &mut curve.0, Some(animate.t));
    }
    Ok(())
}
