use bevy_lookup_curve::*;
use bevy_math::Vec2;
use criterion::{Criterion, black_box, criterion_group, criterion_main};

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
                curve.lookup(black_box(x));
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
                curve.lookup(black_box(x));
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
                curve.lookup(black_box(x));
            })
        })
    });
}

criterion_group!(benches, linear, unweighted_cubic, weighted_cubic,);
criterion_main!(benches);
