//! Content-aware sidebar sizing.
//!
//! The sidebar target width grows continuously with the amount of content,
//! gated so it never exceeds a maximum and can collapse entirely.
//!
//! Run with: cargo run --example content_aware

fn relu_step(t: f64, low: f64, high: f64) -> f64 {
    let scale = high - low;
    if scale == 0.0 {
        return 0.0;
    }
    let u = (t - low) / scale;
    f64::max(0.0, u) - f64::max(0.0, u - 1.0)
}

/// Compute a sidebar width that adapts to content length.
///
/// * `content_chars`   - how many characters need to be displayed
/// * `min_content`     - below this, sidebar is fully collapsed
/// * `max_content`     - above this, sidebar is fully open
/// * `max_width`       - largest desired sidebar width
fn content_aware_sidebar(content_chars: f64, min_content: f64, max_content: f64, max_width: f64) -> u16 {
    let gate = relu_step(content_chars, min_content, max_content);
    let width = gate * max_width;
    width.round() as u16
}

fn main() {
    let scenarios = [
        ("empty", 0.0),
        ("tiny", 5.0),
        ("small menu", 15.0),
        ("medium menu", 35.0),
        ("large menu", 60.0),
        ("huge menu", 100.0),
    ];

    println!("Content-aware sidebar widths (max 30 cols):");
    println!("{:<15} {:>10} {:>10}", "scenario", "content", "width");
    println!("{}", "-".repeat(40));

    for (name, content) in scenarios {
        let width = content_aware_sidebar(content, 10.0, 50.0, 30.0);
        println!("{:<15} {:>10.0} {:>10}", name, content, width);
    }
}
