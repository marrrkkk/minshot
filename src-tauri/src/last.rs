//! The last capture: its pixels, its temp file, and the replace rule.
//!
//! Only one capture is alive at a time. A new one replaces the old, and
//! the old temp file goes with it, so `%TEMP%/minshot` never grows.

use std::path::PathBuf;
use std::sync::Mutex;

use crate::tempfiles;

/// One capture: the pixels we still have in memory and where they landed
/// on disk.
pub struct LastCapture {
    /// The temp PNG, for integrations that need a path (file drag).
    pub path: PathBuf,
    /// The pixels, RGBA, for the clipboard.
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Default)]
pub struct CaptureState {
    last: Mutex<Option<LastCapture>>,
}

impl CaptureState {
    /// Make `cap` the current capture and drop the previous one's file.
    pub fn replace(&self, cap: LastCapture) {
        let old = {
            let mut guard = self.last.lock().expect("capture state poisoned");
            guard.replace(cap)
        };
        if let Some(old) = old {
            tempfiles::remove_file_silent(&old.path);
        }
    }

    /// Run `f` with the current capture, if any. The lock is held for the
    /// duration, so the caller can use the pixels without copying them.
    pub fn with<R>(&self, f: impl FnOnce(Option<&LastCapture>) -> R) -> R {
        let guard = self.last.lock().expect("capture state poisoned");
        f(guard.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capture() -> LastCapture {
        tempfiles::ensure_dir().expect("temp dir failed");
        let path = tempfiles::unique_png_path();
        std::fs::write(&path, b"png").expect("write temp png failed");
        LastCapture {
            path,
            rgba: vec![0; 4],
            width: 1,
            height: 1,
        }
    }

    #[test]
    fn replacing_a_capture_deletes_the_previous_temp_file() {
        let state = CaptureState::default();
        let first = capture();
        let first_path = first.path.clone();
        state.replace(first);
        let second = capture();
        let second_path = second.path.clone();
        state.replace(second);
        assert!(!first_path.exists(), "the old temp file must be deleted");
        assert!(second_path.exists(), "the new temp file must stay");
    }

    #[test]
    fn the_first_capture_keeps_its_file() {
        let state = CaptureState::default();
        let first = capture();
        let path = first.path.clone();
        state.replace(first);
        assert!(path.exists());
    }

    #[test]
    fn the_state_holds_the_latest_capture() {
        let state = CaptureState::default();
        let first = capture();
        let first_path = first.path.clone();
        state.replace(first);
        let second = capture();
        let second_path = second.path.clone();
        state.replace(second);
        let held = state.with(|last| last.map(|l| l.path.clone()));
        assert_eq!(held, Some(second_path));
        assert_ne!(held, Some(first_path));
    }
}
