//! Smooth value transitions for continuous layout parameters.
//!
//! A [`Transition`] linearly interpolates a `f64` value towards a target at a
//! given speed. It is useful for animating ReLU-gated layout parameters without
//! discrete jumps.

use std::time::Duration;

/// A value that moves smoothly towards a target over time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transition {
    current: f64,
    target: f64,
    speed: f64,
}

impl Transition {
    /// Create a new transition starting at `initial` and moving `speed` units per second.
    pub fn new(initial: f64, speed: f64) -> Self {
        Self {
            current: initial,
            target: initial,
            speed: speed.max(0.0),
        }
    }

    /// Set the target value the transition should move towards.
    pub fn set_target(&mut self, target: f64) {
        self.target = target;
    }

    /// Toggle the target between `low` and `high` based on the current target.
    pub fn toggle_between(&mut self, low: f64, high: f64) {
        let midpoint = (low + high) / 2.0;
        self.target = if self.target > midpoint { low } else { high };
    }

    /// The current interpolated value.
    pub fn current(&self) -> f64 {
        self.current
    }

    /// The target value.
    pub fn target(&self) -> f64 {
        self.target
    }

    /// Returns `true` if the current value has reached the target.
    pub fn is_done(&self) -> bool {
        (self.current - self.target).abs() < f64::EPSILON
    }

    /// Advance the transition by `dt`, moving the current value towards the target.
    pub fn update(&mut self, dt: Duration) {
        let step = self.speed * dt.as_secs_f64();
        if self.current < self.target {
            self.current = f64::min(self.current + step, self.target);
        } else if self.current > self.target {
            self.current = f64::max(self.current - step, self.target);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transition_reaches_target() {
        let mut tr = Transition::new(0.0, 10.0);
        tr.set_target(5.0);
        tr.update(Duration::from_millis(100));
        assert!((tr.current() - 1.0).abs() < 1e-9);
        tr.update(Duration::from_secs(10));
        assert!((tr.current() - 5.0).abs() < 1e-9);
        assert!(tr.is_done());
    }

    #[test]
    fn transition_clamps_to_target() {
        let mut tr = Transition::new(0.0, 100.0);
        tr.set_target(1.0);
        tr.update(Duration::from_secs(10));
        assert!((tr.current() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn transition_toggle() {
        let mut tr = Transition::new(0.0, 1.0);
        tr.toggle_between(0.0, 1.0);
        assert_eq!(tr.target(), 1.0);
        tr.toggle_between(0.0, 1.0);
        assert_eq!(tr.target(), 0.0);
    }
}
