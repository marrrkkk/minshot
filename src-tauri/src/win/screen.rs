//! The screen itself: the pixel grab, the monitor bounds, the compositor.
//!
//! Plain GDI — `BitBlt` from the desktop window's device context — which is
//! what every screenshot tool does: the pixels come from the screen as it
//! is right now, with no library in between.

/// The bounds of the monitor that holds a physical point: (x, y, w, h).
/// Physical pixels, whatever the display's scale is.
pub fn monitor_bounds_at(px: i32, py: i32) -> Result<(i32, i32, u32, u32), String> {
    imp::monitor_bounds_at(px, py)
}

/// Grab a rectangle of the screen. The pixels come back as RGBA, top row
/// first — the order PNG and the clipboard want.
pub fn grab_rect(x: i32, y: i32, w: u32, h: u32) -> Result<Vec<u8>, String> {
    imp::grab_rect(x, y, w, h)
}

/// Wait until the compositor has put the last frame on the screen.
/// After the overlay is hidden, that is the frame without the dim.
pub fn wait_for_compositor() {
    imp::wait_for_compositor()
}

#[cfg(target_os = "windows")]
mod imp {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::POINT;
    use windows::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject,
        EnumDisplaySettingsW, GetDIBits, GetMonitorInfoW, GetWindowDC, MonitorFromPoint, ReleaseDC,
        SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DEVMODEW, DIB_RGB_COLORS,
        ENUM_CURRENT_SETTINGS, MONITORINFOEXW, MONITOR_DEFAULTTONULL, SRCCOPY,
    };
    use windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow;

    pub fn monitor_bounds_at(px: i32, py: i32) -> Result<(i32, i32, u32, u32), String> {
        unsafe {
            let hmon = MonitorFromPoint(POINT { x: px, y: py }, MONITOR_DEFAULTTONULL);
            let mut ex = MONITORINFOEXW::default();
            // The size of the whole struct: that is what tells Windows to
            // fill in the device name as well.
            ex.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
            if !GetMonitorInfoW(hmon, &mut ex.monitorInfo).as_bool() {
                return Err(format!("no monitor at ({px},{py})"));
            }
            // The display mode is in physical pixels no matter what the
            // process's DPI awareness is: that is the frame the screen has.
            let mut dm = DEVMODEW {
                dmSize: std::mem::size_of::<DEVMODEW>() as u16,
                ..Default::default()
            };
            if !EnumDisplaySettingsW(PCWSTR(ex.szDevice.as_ptr()), ENUM_CURRENT_SETTINGS, &mut dm)
                .as_bool()
            {
                return Err("display settings unavailable".to_string());
            }
            let pos = dm.Anonymous1.Anonymous2.dmPosition;
            Ok((pos.x, pos.y, dm.dmPelsWidth, dm.dmPelsHeight))
        }
    }

    pub fn grab_rect(x: i32, y: i32, w: u32, h: u32) -> Result<Vec<u8>, String> {
        let (w, h) = (w as i32, h as i32);
        if w <= 0 || h <= 0 {
            return Err("empty region".to_string());
        }
        unsafe {
            let desktop = GetDesktopWindow();
            let src = GetWindowDC(Some(desktop));
            if src.is_invalid() {
                return Err("no screen device context".to_string());
            }
            let mem = CreateCompatibleDC(Some(src));
            let bitmap = CreateCompatibleBitmap(src, w, h);
            let previous = SelectObject(mem, bitmap.into());

            let blt = BitBlt(mem, 0, 0, w, h, Some(src), x, y, SRCCOPY);

            // A negative height asks for the rows top-down, so the pixels
            // come out in the order the screen has them.
            let mut rgba = vec![0u8; w as usize * h as usize * 4];
            let mut info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: w,
                    biHeight: -h,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    biSizeImage: (w * h * 4) as u32,
                    ..Default::default()
                },
                ..Default::default()
            };
            let read = GetDIBits(
                mem,
                bitmap,
                0,
                h as u32,
                Some(rgba.as_mut_ptr().cast()),
                &mut info,
                DIB_RGB_COLORS,
            );

            SelectObject(mem, previous);
            let _ = DeleteObject(bitmap.into());
            let _ = DeleteDC(mem);
            ReleaseDC(Some(desktop), src);

            if blt.is_err() {
                return Err("screen grab failed".to_string());
            }
            if read == 0 {
                return Err("screen read-back failed".to_string());
            }
            // GDI hands out BGRA; PNG and the clipboard want RGBA.
            for px in rgba.chunks_exact_mut(4) {
                px.swap(0, 2);
            }
            Ok(rgba)
        }
    }

    pub fn wait_for_compositor() {
        // DwmFlush blocks until the desktop window manager has finished the
        // frames it was asked to draw.
        unsafe {
            let _ = windows::Win32::Graphics::Dwm::DwmFlush();
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    pub fn monitor_bounds_at(_px: i32, _py: i32) -> Result<(i32, i32, u32, u32), String> {
        Err("screen capture is Windows-only".to_string())
    }

    pub fn grab_rect(_x: i32, _y: i32, _w: u32, _h: u32) -> Result<Vec<u8>, String> {
        Err("screen capture is Windows-only".to_string())
    }

    pub fn wait_for_compositor() {}
}
