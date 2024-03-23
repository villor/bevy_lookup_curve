use bevy_lookup_curve::*;
use bevy_math::Vec2;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::seq::SliceRandom;
use rand::thread_rng;

pub fn linear(c: &mut Criterion) {
    let curve = LookupCurve::new(vec![
        Knot {
            position: Vec2::ZERO,
            interpolation: KnotInterpolation::Linear,
            ..Default::default()
        },
        Knot {
            position: Vec2::ONE,
            interpolation: KnotInterpolation::Linear,
            ..Default::default()
        },
    ]);
    c.bench_function("0_1_linear_1000", |b| {
        b.iter(|| {
            (0..1000).map(|i| i as f32 / 1000.0).for_each(|x| {
                curve.sample(black_box(x));
            })
        })
    });
}

pub fn unweighted_cubic(c: &mut Criterion) {
    let curve = LookupCurve::new(vec![
        Knot {
            position: Vec2::ZERO,
            interpolation: KnotInterpolation::Cubic,
            ..Default::default()
        },
        Knot {
            position: Vec2::ONE,
            interpolation: KnotInterpolation::Cubic,
            ..Default::default()
        },
    ]);
    c.bench_function("0_1_unweighted_cubic_1000", |b| {
        b.iter(|| {
            (0..1000).map(|i| i as f32 / 1000.0).for_each(|x| {
                curve.sample(black_box(x));
            })
        })
    });
}

pub fn weighted_cubic(c: &mut Criterion) {
    let curve = LookupCurve::new(vec![
        Knot {
            position: Vec2::ZERO,
            interpolation: KnotInterpolation::Cubic,
            right_tangent: Tangent {
                weight: Some(0.5),
                ..Default::default()
            },
            ..Default::default()
        },
        Knot {
            position: Vec2::ONE,
            left_tangent: Tangent {
                weight: Some(0.5),
                ..Default::default()
            },
            ..Default::default()
        },
    ]);
    c.bench_function("0_1_weighted_cubic_1000", |b| {
        b.iter(|| {
            (0..1000).map(|i| i as f32 / 1000.0).for_each(|x| {
                curve.sample(black_box(x));
            })
        })
    });
}

pub fn knot_search(c: &mut Criterion) {
    let curve = LookupCurve::new(vec![
        Knot {
            position: Vec2::new(0.0, 0.0),
            interpolation: KnotInterpolation::Constant,
            ..Default::default()
        },
        Knot {
            position: Vec2::new(0.15585108, 0.30673748),
            interpolation: KnotInterpolation::Constant,
            ..Default::default()
        },
        Knot {
            position: Vec2::new(0.20063695, 0.2114623),
            interpolation: KnotInterpolation::Constant,
            ..Default::default()
        },
        Knot {
            position: Vec2::new(0.28874725, 0.54187226),
            interpolation: KnotInterpolation::Constant,
            ..Default::default()
        },
        Knot {
            position: Vec2::new(0.4267516, 0.407065),
            interpolation: KnotInterpolation::Constant,
            ..Default::default()
        },
        Knot {
            position: Vec2::new(0.4872616, 0.6898962),
            interpolation: KnotInterpolation::Constant,
            ..Default::default()
        },
        Knot {
            position: Vec2::new(0.5605096, 0.57359153),
            interpolation: KnotInterpolation::Constant,
            ..Default::default()
        },
        Knot {
            position: Vec2::new(0.69639015, 0.60002416),
            interpolation: KnotInterpolation::Constant,
            ..Default::default()
        },
        Knot {
            position: Vec2::new(0.8067938, 0.33305302),
            interpolation: KnotInterpolation::Constant,
            ..Default::default()
        },
        Knot {
            position: Vec2::new(0.8842887, 0.7321883),
            interpolation: KnotInterpolation::Constant,
            ..Default::default()
        },
        Knot {
            position: Vec2::new(0.9564754, 0.63967353),
            interpolation: KnotInterpolation::Constant,
            ..Default::default()
        },
        Knot {
            position: Vec2::new(1.0, 1.0),
            ..Default::default()
        },
    ]);

    let sweep_samples: Vec<f32> = (0..1).map(|i| i as f32 / 1000.).collect();

    c.bench_function("sweep_no_cache_1000", |b| {
        b.iter(|| {
            sweep_samples.iter().for_each(|x| {
                curve.sample(black_box(*x));
            })
        })
    });

    let mut random_samples = sweep_samples.clone();
    random_samples.shuffle(&mut thread_rng());

    c.bench_function("random_no_cache_1000", |b| {
        b.iter(|| {
            random_samples.iter().for_each(|x| {
                curve.sample(black_box(*x));
            })
        })
    });
}

criterion_group!(
    benches,
    linear,
    unweighted_cubic,
    weighted_cubic,
    knot_search
);
criterion_main!(benches);
