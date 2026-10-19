//! Clipboard image copy (Windows CF_DIB/CF_BITMAP via `arboard`).
//! PNG bytes are decoded to RGBA for the clipboard; screenshots are opaque
//! so the DIB alpha caveat does not apply.

use arboard::ImageData;
use std::borrow::Cow;

pub fn copy_png_to_clipboard(png_bytes: &[u8]) -> Result<(), String> {
    // We always write the PNG ourselves, so skip format sniffing.
    let img = image::load_from_memory_with_format(png_bytes, image::ImageFormat::Png)
        .map_err(|e| format!("decode PNG failed: {e}"))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Round-trips a generated image through the real Windows clipboard.
    /// Overwrites the session clipboard — dev-machine only.
    #[test]
    fn clipboard_round_trip() {
        let img: image::RgbaImage = image::RgbaImage::from_pixel(8, 8, image::Rgba([9, 9, 9, 255]));
        let mut png = Vec::new();
        {
            use image::ImageEncoder;
            image::codecs::png::PngEncoder::new(&mut png)
                .write_image(img.as_raw(), 8, 8, image::ExtendedColorType::Rgba8)
                .unwrap();
        }
        copy_png_to_clipboard(&png).expect("clipboard copy failed");
        let mut cb = arboard::Clipboard::new().expect("clipboard open failed");
        let back = cb.get_image().expect("clipboard read-back failed");
        assert_eq!((back.width, back.height), (8, 8));
    }
}
