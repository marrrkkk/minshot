//! Region capture: CSS rect (overlay window) -> physical pixels -> PNG bytes.
//!
//! The overlay window spans the virtual screen. Its `devicePixelRatio`
//! converts CSS px to physical px. The capture is taken from the monitor
//! containing the selection center and clamped to that monitor, so
//! same-DPI setups are pixel-accurate. On mixed-DPI desktops the overlay
//! scale matches the monitor hosting most of the overlay window; a
//! selection on a differently-scaled monitor can be off by the scale ratio.
//! That trade-off is documented in the README (per-monitor overlays would be
//! needed for exact mixed-DPI mapping).

use image::{ImageEncoder, RgbaImage};

#[derive(Debug, Clone)]
pub struct Captured {
    pub width: u32,
    pub height: u32,
    pub png_bytes: Vec<u8>,
}

/// Normalize a drag rect. Returns `None` for zero/tiny selections (a click,
/// not a capture).
pub fn normalize_rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Option<(i32, i32, u32, u32)> {
    let x = x0.min(x1).round() as i32;
    let y = y0.min(y1).round() as i32;
    let w = (x0 - x1).abs().round() as i64;
    let h = (y0 - y1).abs().round() as i64;
    if w < 2 || h < 2 {
        return None;
    }
    Some((x, y, w as u32, h as u32))
}

/// Capture a rect given in overlay-window CSS px (`scale` =
/// `window.devicePixelRatio`).
pub fn capture_css_rect(x: f64, y: f64, w: f64, h: f64, scale: f64) -> Result<Captured, String> {
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    let (vx, vy, _, _) = crate::win::virtual_screen_rect();
    let phys = normalize_rect(
        vx as f64 + x * scale,
        vy as f64 + y * scale,
        vx as f64 + (x + w) * scale,
        vy as f64 + (y + h) * scale,
    )
    .ok_or_else(|| "selection too small".to_string())?;
    capture_physical_rect(phys.0, phys.1, phys.2, phys.3)
}

/// Capture a physical-pixel rect, clamped to the monitor holding its center.
pub fn capture_physical_rect(x: i32, y: i32, w: u32, h: u32) -> Result<Captured, String> {
    let cx = x + (w / 2) as i32;
    let cy = y + (h / 2) as i32;
    let monitor = xcap::Monitor::from_point(cx, cy)
        .map_err(|e| format!("no monitor at ({cx},{cy}): {e}"))?;
    let full = monitor
        .capture_image()
        .map_err(|e| format!("screen capture failed: {e}"))?;

    let mx = monitor.x().map_err(|e| format!("monitor x failed: {e}"))?;
    let my = monitor.y().map_err(|e| format!("monitor y failed: {e}"))?;
    let mw = monitor
        .width()
        .map_err(|e| format!("monitor width failed: {e}"))?;
    let mh = monitor
        .height()
        .map_err(|e| format!("monitor height failed: {e}"))?;
    // Intersect requested rect with monitor bounds.
    let ix0 = (x.max(mx)) as u32;
    let iy0 = (y.max(my)) as u32;
    let ix1 = ((x as i64 + w as i64).min(mx as i64 + mw as i64)) as u32;
    let iy1 = ((y as i64 + h as i64).min(my as i64 + mh as i64)) as u32;
    if ix1 <= ix0 || iy1 <= iy0 {
        return Err("selection is outside all monitors".to_string());
    }
    let (cw, ch) = (ix1 - ix0, iy1 - iy0);
    let local_x = ix0 - mx as u32;
    let local_y = iy0 - my as u32;
    let cropped: RgbaImage =
        image::imageops::crop_imm(&full, local_x, local_y, cw, ch).to_image();

    let mut png_bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png_bytes)
        .write_image(cropped.as_raw(), cw, ch, image::ExtendedColorType::Rgba8)
        .map_err(|e| format!("PNG encode failed: {e}"))?;

    Ok(Captured {
        width: cw,
        height: ch,
        png_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_normalizes_inverted_drag() {
        assert_eq!(normalize_rect(100.0, 100.0, 10.0, 20.0), Some((10, 20, 90, 80)));
    }

    #[test]
    fn rect_rejects_zero_and_tiny() {
        assert_eq!(normalize_rect(5.0, 5.0, 5.0, 5.0), None);
        assert_eq!(normalize_rect(0.0, 0.0, 1.0, 50.0), None);
        assert_eq!(normalize_rect(0.0, 0.0, 50.0, 1.0), None);
    }

    #[test]
    fn rect_rounds_to_whole_pixels() {
        assert_eq!(normalize_rect(10.4, 20.6, 100.4, 200.6), Some((10, 21, 90, 180)));
    }

    /// Hits the real screen (DXGI) — verifies xcap + crop + PNG encode on hardware.
    #[test]
    fn smoke_capture_real_screen() {
        let cap = capture_physical_rect(10, 10, 64, 64).expect("screen capture failed");
        assert_eq!((cap.width, cap.height), (64, 64));
        assert!(cap.png_bytes.len() > 100);
        assert_eq!(&cap.png_bytes[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }
}
