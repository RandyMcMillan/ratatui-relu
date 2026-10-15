//! Compares ReLU, sigmoid, and smoothstep gating functions.
//!
//! Run with: cargo run --example smoothstep

fn relu_step(t: f64, low: f64, high: f64) -> f64 {
    let scale = high - low;
    if scale == 0.0 {
        return 0.0;
    }
    let u = (t - low) / scale;
    f64::max(0.0, u) - f64::max(0.0, u - 1.0)
}

fn sigmoid_step(t: f64, center: f64, width: f64) -> f64 {
    1.0 / (1.0 + f64::exp(-(t - center) / width))
}

fn smoothstep(t: f64, low: f64, high: f64) -> f64 {
    let scale = high - low;
    if scale == 0.0 {
        return 0.0;
    }
    let u = f64::clamp((t - low) / scale, 0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

fn main() {
    println!(
        "{:<8} {:>10} {:>10} {:>10}",
        "t", "relu", "sigmoid", "smoothstep"
    );
    println!("{}", "-".repeat(42));

    for i in 0..=20 {
        let t = i as f64 * 0.1;
        println!(
            "{:<8.2} {:>10.3} {:>10.3} {:>10.3}",
            t,
            relu_step(t, 0.5, 1.5),
            sigmoid_step(t, 1.0, 0.2),
            smoothstep(t, 0.5, 1.5)
        );
    }
}
