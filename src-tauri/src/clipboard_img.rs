//! Clipboard image copy (Windows CF_DIB/CF_BITMAP via `arboard`).
//! PNG bytes are decoded to RGBA for the clipboard; screenshots are opaque
//! so the DIB alpha caveat does not apply.

use arboard::ImageData;
use std::borrow::Cow;

pub fn copy_png_to_clipboard(png_bytes: &[u8]) -> Result<(), String> {
    let img =
        image::load_from_memory(png_bytes).map_err(|e| format!("decode PNG failed: {e}"))?;
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width() as usize, rgba.height() as usize);
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("open clipboard failed: {e}"))?;
    clipboard
        .set_image(ImageData {
            width: w,
            height: h,
            bytes: Cow::from(rgba.into_raw()),
        })
        .map_err(|e| format!("clipboard copy failed: {e}"))
}
