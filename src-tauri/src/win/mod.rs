//! Windows-specific helpers, isolated from the portable app logic.
//!
//! Everything in this module only compiles to meaningful code on Windows.
//! Other platforms get safe stubs so `cargo check` stays portable.

/// Virtual screen rectangle (physical pixels): (x, y, width, height).
/// Covers all monitors, so one overlay window can span every display.
/// Negative x/y happens when a secondary monitor sits left of / above primary.
#[cfg(target_os = "windows")]
pub fn virtual_screen_rect() -> (i32, i32, u32, u32) {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
        SM_YVIRTUALSCREEN,
    };
    unsafe {
        let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let w = GetSystemMetrics(SM_CXVIRTUALSCREEN).max(1) as u32;
        let h = GetSystemMetrics(SM_CYVIRTUALSCREEN).max(1) as u32;
        (x, y, w, h)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn virtual_screen_rect() -> (i32, i32, u32, u32) {
    (0, 0, 800, 600)
}

pub mod drag;
