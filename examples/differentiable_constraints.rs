//! Differentiable layout constraints as soft penalties.
//!
//! Instead of hard `if` checks, encode layout rules as penalties and
//! minimize the total constraint violation with gradient descent.
//!
//! Run with: cargo run --example differentiable_constraints

fn relu_step(t: f64, low: f64, high: f64) -> f64 {
    let scale = high - low;
    if scale == 0.0 {
        return 0.0;
    }
    let u = (t - low) / scale;
    f64::max(0.0, u) - f64::max(0.0, u - 1.0)
}

fn sidebar_width(t: f64, max_width: f64) -> f64 {
    relu_step(t, 0.5, 1.5) * max_width
}

fn penalty(value: f64, target: f64, weight: f64) -> f64 {
    weight * (value - target).powi(2)
}

fn hinge_below(value: f64, minimum: f64, weight: f64) -> f64 {
    let diff = minimum - value;
    if diff > 0.0 {
        weight * diff * diff
    } else {
        0.0
    }
}

fn hinge_above(value: f64, maximum: f64, weight: f64) -> f64 {
    let diff = value - maximum;
    if diff > 0.0 {
        weight * diff * diff
    } else {
        0.0
    }
}

fn layout_cost(t: f64, total_width: f64) -> f64 {
    let sw = sidebar_width(t, 30.0);
    let mw = total_width - sw;

    let mut cost = 0.0;

    // Prefer sidebar width near 30 when open.
    cost += penalty(sw, 30.0, 1.0);

    // Main panel must be at least 70 cols.
    cost += hinge_below(mw, 70.0, 10.0);

    // Sidebar must not exceed 40 cols.
    cost += hinge_above(sw, 40.0, 10.0);

    // Slight preference for the sidebar being either fully open or closed.
    cost += penalty(relu_step(t, 0.5, 1.5), 1.0, 0.5);

    cost
}

fn grad(t: f64, total_width: f64) -> f64 {
    let eps = 1e-6;
    (layout_cost(t + eps, total_width) - layout_cost(t - eps, total_width)) / (2.0 * eps)
}

fn main() {
    let total_width = 100.0;
    // Start inside the ReLU ramp (0.5 < t < 1.5) so the gate has a non-zero
    // gradient. Starting at t=0 would be a "dead ReLU" region.
    let mut t = 0.6;
    let lr = 0.0002;

    println!("Optimizing layout with differentiable constraints...");
    println!(
        "{:<6} {:<8} {:<10} {:<10} {:<10}",
        "iter", "t", "sidebar", "main", "cost"
    );
    println!("{}", "-".repeat(50));

    for i in 0..50 {
        let g = grad(t, total_width);
        t -= lr * g;
        t = f64::clamp(t, 0.0, 2.0);

        let sw = sidebar_width(t, 30.0);
        let mw = total_width - sw;
        println!(
            "{:<6} {:<8.4} {:<10.2} {:<10.2} {:<10.4}",
            i,
            t,
            sw,
            mw,
            layout_cost(t, total_width)
        );
    }
}
