use bevy::prelude::*;
use bevy_lookup_curve::{
    editor::LookupCurveEditor, Knot, KnotInterpolation, LookupCache, LookupCurve, LookupCurvePlugin,
};

fn main() {
    App::new()
        .register_type::<AnimateWithCurve>()
        .add_plugins(DefaultPlugins)
        .add_plugins(LookupCurvePlugin)
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_editor.after(setup))
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

#[derive(Component, Reflect)]
struct AnimateWithCurve(LookupCurve);

#[derive(Component)]
struct AnimationCache(LookupCache);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
        ])),
        AnimationCache(LookupCache::new()),
    ));
}

fn setup_editor(world: &mut World) {
    let entities: Vec<Entity> = world
        .query_filtered::<Entity, With<AnimateWithCurve>>()
        .iter(world)
        .collect();
    let component_id = world.component_id::<AnimateWithCurve>().unwrap();
    for entity in entities {
        world.spawn(LookupCurveEditor::new_from_component(
            entity,
            component_id,
            ".0",
        ));
    }
}

fn update(
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
    }
}
