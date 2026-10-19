//! Clipboard image copy (Windows CF_DIB/CF_BITMAP via `arboard`).
//!
//! The pixels go in as they are: the capture is already RGBA, so there is
//! no PNG encode/decode round trip on the way to the clipboard.

use arboard::ImageData;
use std::borrow::Cow;

pub fn copy_rgba_to_clipboard(rgba: &[u8], width: u32, height: u32) -> Result<(), String> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("open clipboard failed: {e}"))?;
    clipboard
        .set_image(ImageData {
            width: width as usize,
            height: height as usize,
            bytes: Cow::Borrowed(rgba),
        })
        .map_err(|e| format!("clipboard copy failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Round-trips a generated image through the real Windows clipboard.
    /// Overwrites the session clipboard — dev-machine only, so it is
    /// skipped in CI: `cargo test -- --include-ignored` runs it.
    #[test]
    #[ignore = "needs an interactive desktop session"]
    fn clipboard_round_trip() {
        let rgba = vec![9u8; 8 * 8 * 4];
        copy_rgba_to_clipboard(&rgba, 8, 8).expect("clipboard copy failed");
        let mut cb = arboard::Clipboard::new().expect("clipboard open failed");
        let back = cb.get_image().expect("clipboard read-back failed");
        assert_eq!((back.width, back.height), (8, 8));
    }
}
