//! Gradient-based layout optimization.
//!
//! We optimize a continuous toggle so that:
//!   - the sidebar is wide enough to show 20 columns of content when open,
//!   - the main panel is at least 80 columns,
//!   - the total width is respected.
//!
//! Run with: cargo run --example optimized_layout

/// ReLU ramp that is 0 below `low` and 1 above `high`.
fn relu_step(t: f64, low: f64, high: f64) -> f64 {
    let scale = high - low;
    if scale == 0.0 {
        return 0.0;
    }
    let u = (t - low) / scale;
    f64::max(0.0, u) - f64::max(0.0, u - 1.0)
}

/// Differentiable sidebar width.
fn sidebar_width(t: f64, target_sidebar: f64) -> f64 {
    let gate = relu_step(t, 0.5, 1.5);
    gate * target_sidebar
}

/// Main width is whatever is left.
fn main_width(t: f64, total_width: f64, target_sidebar: f64) -> f64 {
    total_width - sidebar_width(t, target_sidebar)
}

/// Differentiable penalty for violating a minimum value.
fn below_penalty(value: f64, minimum: f64) -> f64 {
    let diff = minimum - value;
    if diff > 0.0 {
        diff * diff
    } else {
        0.0
    }
}

fn loss(t: f64, total_width: f64, target_sidebar: f64, min_main: f64) -> f64 {
    let sw = sidebar_width(t, target_sidebar);
    let mw = main_width(t, total_width, target_sidebar);

    // We want the sidebar close to the target when open.
    let fit_penalty = (sw - target_sidebar).powi(2);

    // We want the main panel to be wide enough.
    let main_penalty = below_penalty(mw, min_main);

    fit_penalty + 10.0 * main_penalty
}

/// Numerical gradient.
fn grad_loss(t: f64, total_width: f64, target_sidebar: f64, min_main: f64) -> f64 {
    let eps = 1e-6;
    (loss(t + eps, total_width, target_sidebar, min_main)
        - loss(t - eps, total_width, target_sidebar, min_main))
        / (2.0 * eps)
}

fn main() {
    let total_width = 120.0;
    let target_sidebar = 30.0;
    let min_main = 80.0;

    // Start inside the ReLU ramp (0.5 < t < 1.5) so the gate has a non-zero
    // gradient. Starting at t=0 would be a "dead ReLU" region.
    let mut t = 0.6;
    let lr = 0.0002;

    println!("Optimizing layout toggle with gradient descent...");
    println!(
        "{:<6} {:<8} {:<10} {:<10} {:<10}",
        "iter", "t", "sidebar", "main", "loss"
    );
    println!("{}", "-".repeat(50));

    for i in 0..50 {
        let g = grad_loss(t, total_width, target_sidebar, min_main);
        t -= lr * g;
        t = f64::clamp(t, 0.0, 2.0);

        println!(
            "{:<6} {:<8.4} {:<10.2} {:<10.2} {:<10.4}",
            i,
            t,
            sidebar_width(t, target_sidebar),
            main_width(t, total_width, target_sidebar),
            loss(t, total_width, target_sidebar, min_main)
        );
    }

    println!();
    println!(
        "Final: toggle={:.4}, sidebar={:.0} cols, main={:.0} cols",
        t,
        sidebar_width(t, target_sidebar),
        main_width(t, total_width, target_sidebar)
    );
}
