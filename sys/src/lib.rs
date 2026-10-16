//! C FFI bindings for `ratatui-relu`.
//!
//! This crate exposes the animation primitives so they can be driven from C
//! or other languages that speak C.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::c_void;
use std::time::Duration;

use ratatui_relu::animation::Transition;
use ratatui_relu::multi_panel::MultiPanelState;

// ============================================================
// Transition
// ============================================================

/// Create a new [`Transition`] starting at `value` and moving with `speed`.
///
/// Returns an opaque pointer that must be freed with
/// `ratatui_relu_transition_free`.
#[unsafe(no_mangle)]
pub extern "C" fn ratatui_relu_transition_new(value: f64, speed: f64) -> *mut Transition {
    Box::into_raw(Box::new(Transition::new(value, speed)))
}

/// Free a [`Transition`] created by `ratatui_relu_transition_new`.
#[unsafe(no_mangle)]
pub extern "C" fn ratatui_relu_transition_free(ptr: *mut Transition) {
    if !ptr.is_null() {
        unsafe { drop(Box::from_raw(ptr)) };
    }
}

/// Set the target value of a [`Transition`].
#[unsafe(no_mangle)]
pub extern "C" fn ratatui_relu_transition_set_target(ptr: *mut Transition, target: f64) {
    if let Some(t) = unsafe { ptr.as_mut() } {
        t.set_target(target);
    }
}

/// Advance a [`Transition`] by `dt_ms` milliseconds.
#[unsafe(no_mangle)]
pub extern "C" fn ratatui_relu_transition_update(ptr: *mut Transition, dt_ms: u64) {
    if let Some(t) = unsafe { ptr.as_mut() } {
        t.update(Duration::from_millis(dt_ms));
    }
}

/// Get the current value of a [`Transition`].
#[unsafe(no_mangle)]
pub extern "C" fn ratatui_relu_transition_current(ptr: *const Transition) -> f64 {
    unsafe { ptr.as_ref() }
        .map(|t| t.current())
        .unwrap_or(0.0)
}

/// Get the target value of a [`Transition`].
#[unsafe(no_mangle)]
pub extern "C" fn ratatui_relu_transition_target(ptr: *const Transition) -> f64 {
    unsafe { ptr.as_ref() }
        .map(|t| t.target())
        .unwrap_or(0.0)
}

/// Drive a [`Transition`] for `steps` updates of `dt_ms` milliseconds,
/// calling `callback` after each update. This demonstrates Rust invoking C.
#[unsafe(no_mangle)]
pub extern "C" fn ratatui_relu_transition_drive(
    ptr: *mut Transition,
    dt_ms: u64,
    steps: usize,
    callback: Option<unsafe extern "C" fn(current: f64, target: f64, userdata: *mut c_void)>,
    userdata: *mut c_void,
) {
    if let Some(t) = unsafe { ptr.as_mut() } {
        for _ in 0..steps {
            t.update(Duration::from_millis(dt_ms));
            if let Some(cb) = callback {
                unsafe { cb(t.current(), t.target(), userdata) };
            }
        }
    }
}

// ============================================================
// MultiPanelState
// ============================================================

/// Create a new [`MultiPanelState`] with all panels open.
#[unsafe(no_mangle)]
pub extern "C" fn ratatui_relu_multi_panel_new(speed: f64) -> *mut MultiPanelState {
    Box::into_raw(Box::new(MultiPanelState::open(speed)))
}

/// Free a [`MultiPanelState`] created by `ratatui_relu_multi_panel_new`.
#[unsafe(no_mangle)]
pub extern "C" fn ratatui_relu_multi_panel_free(ptr: *mut MultiPanelState) {
    if !ptr.is_null() {
        unsafe { drop(Box::from_raw(ptr)) };
    }
}

/// Advance all panel transitions by `dt_ms` milliseconds.
#[unsafe(no_mangle)]
pub extern "C" fn ratatui_relu_multi_panel_update(ptr: *mut MultiPanelState, dt_ms: u64) {
    if let Some(s) = unsafe { ptr.as_mut() } {
        s.update(Duration::from_millis(dt_ms));
    }
}

/// Toggle the header panel between fully open and fully closed.
#[unsafe(no_mangle)]
pub extern "C" fn ratatui_relu_multi_panel_toggle_header(ptr: *mut MultiPanelState) {
    if let Some(s) = unsafe { ptr.as_mut() } {
        let target = if s.header.target() > 0.5 { 0.0 } else { 1.0 };
        s.header.set_target(target);
    }
}

/// Toggle the left sidebar between fully open and fully closed.
#[unsafe(no_mangle)]
pub extern "C" fn ratatui_relu_multi_panel_toggle_left(ptr: *mut MultiPanelState) {
    if let Some(s) = unsafe { ptr.as_mut() } {
        let target = if s.left_sidebar.target() > 0.5 { 0.0 } else { 1.0 };
        s.left_sidebar.set_target(target);
    }
}

/// Toggle the right sidebar between fully open and fully closed.
#[unsafe(no_mangle)]
pub extern "C" fn ratatui_relu_multi_panel_toggle_right(ptr: *mut MultiPanelState) {
    if let Some(s) = unsafe { ptr.as_mut() } {
        let target = if s.right_sidebar.target() > 0.5 { 0.0 } else { 1.0 };
        s.right_sidebar.set_target(target);
    }
}

/// Toggle the footer panel between fully open and fully closed.
#[unsafe(no_mangle)]
pub extern "C" fn ratatui_relu_multi_panel_toggle_footer(ptr: *mut MultiPanelState) {
    if let Some(s) = unsafe { ptr.as_mut() } {
        let target = if s.footer.target() > 0.5 { 0.0 } else { 1.0 };
        s.footer.set_target(target);
    }
}

/// Toggle all panels at once.
#[unsafe(no_mangle)]
pub extern "C" fn ratatui_relu_multi_panel_toggle_all(ptr: *mut MultiPanelState) {
    if let Some(s) = unsafe { ptr.as_mut() } {
        s.toggle_all();
    }
}

/// Read the current header size (0.0 = closed, 1.0 = open).
#[unsafe(no_mangle)]
pub extern "C" fn ratatui_relu_multi_panel_header(ptr: *const MultiPanelState) -> f64 {
    unsafe { ptr.as_ref() }.map(|s| s.header.current()).unwrap_or(0.0)
}

/// Read the current left sidebar size (0.0 = closed, 1.0 = open).
#[unsafe(no_mangle)]
pub extern "C" fn ratatui_relu_multi_panel_left(ptr: *const MultiPanelState) -> f64 {
    unsafe { ptr.as_ref() }
        .map(|s| s.left_sidebar.current())
        .unwrap_or(0.0)
}

/// Read the current right sidebar size (0.0 = closed, 1.0 = open).
#[unsafe(no_mangle)]
pub extern "C" fn ratatui_relu_multi_panel_right(ptr: *const MultiPanelState) -> f64 {
    unsafe { ptr.as_ref() }
        .map(|s| s.right_sidebar.current())
        .unwrap_or(0.0)
}

/// Read the current footer size (0.0 = closed, 1.0 = open).
#[unsafe(no_mangle)]
pub extern "C" fn ratatui_relu_multi_panel_footer(ptr: *const MultiPanelState) -> f64 {
    unsafe { ptr.as_ref() }.map(|s| s.footer.current()).unwrap_or(0.0)
}
