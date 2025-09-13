use bevy_lookup_curve::Knot;
use bevy_lookup_curve::knot_search::KnotSearch;
use bevy_math::Vec2;
use criterion::BenchmarkId;
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn generate_knots(n: usize) -> Vec<Knot> {
    let step = 1. / (n - 1) as f32;
    (0..n)
        .map(|i| Knot {
            position: Vec2::new(step * i as f32, 0.),
            ..Default::default()
        })
        .collect()
}

pub fn knot_search(c: &mut Criterion) {
    let curve_sizes = [
        3, 4, 5, 6, 7, 8, 9, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55, 60, 65, 70, 75, 80, 85, 90,
        95, 100,
    ];

    let mut group = c.benchmark_group("Knot search (sweep)");
    let sweep_samples: Vec<f32> = (1..999).map(|i| i as f32 / 1000.).collect();
    for i in curve_sizes.iter() {
        let knots = generate_knots(*i);
        group.bench_with_input(BenchmarkId::new("Binary", i), i, |b, _| {
            b.iter(|| {
                let knots = knots.clone(); // cache bust
                sweep_samples.iter().for_each(|x| {
                    knots.search_knots_binary(black_box(*x));
                })
            })
        });
        group.bench_with_input(BenchmarkId::new("Linear", i), i, |b, _| {
            b.iter(|| {
                let knots = knots.clone(); // cache bust
                sweep_samples.iter().for_each(|x| {
                    knots.search_knots_linear(black_box(*x));
                })
            })
        });
        group.bench_with_input(BenchmarkId::new("Hybrid", i), i, |b, _| {
            b.iter(|| {
                let knots = knots.clone(); // cache bust
                sweep_samples.iter().for_each(|x| {
                    knots.search_knots(black_box(*x));
                })
            })
        });
        group.bench_with_input(BenchmarkId::new("Hybrid (with cache)", i), i, |b, _| {
            b.iter(|| {
                let knots = knots.clone(); // cache bust
                let mut cache = None;
                sweep_samples.iter().for_each(|x| {
                    knots.search_knots_with_cache(black_box(*x), &mut cache);
                })
            })
        });
    }
    group.finish();
}

criterion_group!(benches2, knot_search);
criterion_main!(benches2);
