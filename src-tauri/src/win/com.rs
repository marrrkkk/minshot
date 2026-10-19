//! COM on this thread: make it available, and undo only what we set up.
//!
//! The webview (wry) already brings COM up on the UI thread. Calling
//! `OleInitialize` there says "already there" and does *not* raise the
//! count, so the matching `OleUninitialize` would take COM down again
//! under the app's feet. So: ask first, undo only our own.

/// Is COM already up on this thread?
pub fn initialized() -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Com::{CoGetApartmentType, APTTYPE, APTTYPEQUALIFIER};
        unsafe {
            let mut apt = APTTYPE::default();
            let mut qualifier = APTTYPEQUALIFIER::default();
            CoGetApartmentType(&mut apt, &mut qualifier).is_ok()
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

/// Run `f` with OLE available on this thread (COM plus drag and drop).
pub fn with_ole<T>(f: impl FnOnce() -> T) -> T {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Ole::{OleInitialize, OleUninitialize};
        let had = initialized();
        if !had {
            unsafe {
                let _ = OleInitialize(None);
            }
        }
        let out = f();
        if !had {
            unsafe {
                OleUninitialize();
            }
        }
        out
    }
    #[cfg(not(target_os = "windows"))]
    {
        f()
    }
}
