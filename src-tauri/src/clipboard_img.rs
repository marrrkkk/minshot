//! Clipboard image copy: a CF_DIBV5 (32-bpp) written straight into the
//! clipboard's own memory.
//!
//! The pixels go in as they are: the capture is already RGBA, so there is
//! no PNG encode/decode round trip on the way to the clipboard. The rows go
//! in bottom-up, which is what a DIB with a positive height expects, so no
//! separate flipped copy of the image is needed.

/// Put an RGBA image on the clipboard.
pub fn copy_rgba_to_clipboard(rgba: &[u8], width: u32, height: u32) -> Result<(), String> {
    imp::copy_rgba_to_clipboard(rgba, width, height)
}

#[cfg(target_os = "windows")]
mod imp {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Graphics::Gdi::{BITMAPV5HEADER, BI_BITFIELDS};
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
    };
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    use windows::Win32::System::Ole::CF_DIBV5;

    pub fn copy_rgba_to_clipboard(rgba: &[u8], width: u32, height: u32) -> Result<(), String> {
        let (w, h) = (width as usize, height as usize);
        let pixels = w * h * 4;
        if w == 0 || h == 0 || rgba.len() < pixels {
            return Err("pixel buffer does not match the image size".to_string());
        }
        unsafe {
            let header_size = std::mem::size_of::<BITMAPV5HEADER>();
            let hmem = GlobalAlloc(GMEM_MOVEABLE, header_size + pixels)
                .map_err(|e| format!("clipboard memory failed: {e}"))?;
            let ptr = GlobalLock(hmem).cast::<u8>();
            if ptr.is_null() {
                let _ = windows::Win32::Foundation::GlobalFree(Some(hmem));
                return Err("clipboard memory could not be locked".to_string());
            }

            let header = BITMAPV5HEADER {
                bV5Size: header_size as u32,
                bV5Width: w as i32,
                // A positive height means the rows are stored bottom-up.
                bV5Height: h as i32,
                bV5Planes: 1,
                bV5BitCount: 32,
                bV5Compression: BI_BITFIELDS,
                bV5SizeImage: pixels as u32,
                bV5XPelsPerMeter: 0,
                bV5YPelsPerMeter: 0,
                bV5ClrUsed: 0,
                bV5ClrImportant: 0,
                bV5RedMask: 0x00ff_0000,
                bV5GreenMask: 0x0000_ff00,
                bV5BlueMask: 0x0000_00ff,
                bV5AlphaMask: 0xff00_0000,
                bV5CSType: 0x7352_4742, // LCS_sRGB
                bV5Endpoints: Default::default(),
                bV5GammaRed: 0,
                bV5GammaGreen: 0,
                bV5GammaBlue: 0,
                bV5Intent: 4, // LCS_GM_IMAGES
                bV5ProfileData: 0,
                bV5ProfileSize: 0,
                bV5Reserved: 0,
            };
            std::ptr::copy_nonoverlapping(
                &header as *const BITMAPV5HEADER as *const u8,
                ptr,
                header_size,
            );

            // The rows, bottom-up, with R and B swapped: a DIB holds BGR.
            let row = w * 4;
            let dst = ptr.add(header_size);
            for index in 0..h {
                let src = rgba.as_ptr().add((h - 1 - index) * row);
                let out = dst.add(index * row);
                std::ptr::copy_nonoverlapping(src, out, row);
                let line = std::slice::from_raw_parts_mut(out, row);
                for px in line.chunks_exact_mut(4) {
                    px.swap(0, 2);
                }
            }
            let _ = GlobalUnlock(hmem);

            // From here on the clipboard owns the memory.
            let placed = (|| -> Result<(), String> {
                OpenClipboard(None).map_err(|e| format!("open clipboard failed: {e}"))?;
                let result = (|| -> Result<(), String> {
                    EmptyClipboard().map_err(|e| format!("clipboard clear failed: {e}"))?;
                    SetClipboardData(CF_DIBV5.0 as u32, Some(HANDLE(hmem.0)))
                        .map_err(|e| format!("clipboard copy failed: {e}"))?;
                    Ok(())
                })();
                let _ = CloseClipboard();
                result
            })();
            if placed.is_err() {
                let _ = windows::Win32::Foundation::GlobalFree(Some(hmem));
            }
            placed
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    pub fn copy_rgba_to_clipboard(_rgba: &[u8], _width: u32, _height: u32) -> Result<(), String> {
        Err("the clipboard is Windows-only".to_string())
    }
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
        let mut rgba = Vec::new();
        for i in 0..(8 * 8 * 4) {
            rgba.push((i % 251) as u8);
        }
        copy_rgba_to_clipboard(&rgba, 8, 8).expect("clipboard copy failed");
        let back = read_clipboard_dib().expect("clipboard read-back failed");
        assert_eq!((back.0, back.1), (8, 8));
        assert_eq!(back.2, rgba);
    }

    /// Read the DIB back off the clipboard: (width, height, rgba).
    #[cfg(target_os = "windows")]
    fn read_clipboard_dib() -> Result<(u32, u32, Vec<u8>), String> {
        use windows::Win32::Foundation::HGLOBAL;
        use windows::Win32::Graphics::Gdi::BITMAPV5HEADER;
        use windows::Win32::System::DataExchange::{
            CloseClipboard, GetClipboardData, OpenClipboard,
        };
        use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};
        use windows::Win32::System::Ole::CF_DIBV5;

        unsafe {
            OpenClipboard(None).map_err(|e| format!("open clipboard failed: {e}"))?;
            let handle = GetClipboardData(CF_DIBV5.0 as u32)
                .map_err(|e| format!("no DIB on the clipboard: {e}"))?;
            let memory = HGLOBAL(handle.0);
            let ptr = GlobalLock(memory).cast::<u8>();
            if ptr.is_null() {
                let _ = CloseClipboard();
                return Err("clipboard memory could not be locked".to_string());
            }
            let header = std::ptr::read_unaligned(ptr as *const BITMAPV5HEADER);
            let (w, h) = (
                header.bV5Width as usize,
                header.bV5Height.unsigned_abs() as usize,
            );
            let data = ptr.add(std::mem::size_of::<BITMAPV5HEADER>());
            let row = w * 4;
            let mut rgba = vec![0u8; w * h * 4];
            for index in 0..h {
                let src = data.add(index * row);
                let out = rgba.as_mut_ptr().add((h - 1 - index) * row);
                std::ptr::copy_nonoverlapping(src, out, row);
                let line = std::slice::from_raw_parts_mut(out, row);
                for px in line.chunks_exact_mut(4) {
                    px.swap(0, 2);
                }
            }
            let _ = GlobalUnlock(memory);
            let _ = CloseClipboard();
            Ok((w as u32, h as u32, rgba))
        }
    }
}
