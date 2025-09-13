//! Shows manual creation and usage of `LookupCurve`, printing out beziers and lookups to console.
//!
//! Illustrates usage without dependencies on Bevy or egui.
use bevy_lookup_curve::prelude::*;
use bevy_math::Vec2;

fn main() {
    // Manually constructed linear curve
    let linear_curve = LookupCurve::new(vec![
        Knot {
            position: Vec2::ZERO,
            interpolation: KnotInterpolation::Linear,
            ..Default::default()
        },
        Knot {
            position: Vec2::ONE,
            ..Default::default()
        },
    ]);
    print_curve(&linear_curve, "Linear");

    // Manually constructed cubic curve
    let cubic_curve = LookupCurve::new(vec![
        Knot {
            position: Vec2::ZERO,
            interpolation: KnotInterpolation::Cubic,
            right_tangent: Tangent {
                slope: 7.6,
                weight: Some(0.15),
                ..Default::default()
            },
            ..Default::default()
        },
        Knot {
            position: Vec2::ONE,
            left_tangent: Tangent {
                slope: 2.0,
                ..Default::default()
            },
            ..Default::default()
        },
    ]);
    print_curve(&cubic_curve, "Cubic");

    // Curve loaded from file
    #[cfg(feature = "ron")]
    {
        let path = "./assets/example.curve.ron";
        let file_curve = LookupCurve::load_from_file(path).expect("Failed to load curve");
        print_curve(&file_curve, path);
    }
}

fn print_curve(curve: &LookupCurve, name: &str) {
    println!("--- {} ---\n", name);

    // Print ASCII graph
    for y in (0..=10).rev() {
        let fy = y as f32 / 10.0;
        for x in 0..=40 {
            let fx = x as f32 / 40.0;
            let v = curve.lookup(fx);
            if (v - fy).abs() < 0.05 {
                print!("*");
            } else if y == 0 {
                print!("-");
            } else if x == 0 {
                print!("|");
            } else {
                print!(" ");
            }
        }
        println!();
    }
    println!();

    // Print samples
    for t in 0..=10 {
        let t = t as f32 / 10.0;
        let v = curve.lookup(t);
        println!("{:.1} => {:.3}", t, v);
    }

    println!();
}
