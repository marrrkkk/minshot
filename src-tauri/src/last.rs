//! The last capture: its PNG, its file, and the replace rule.
//!
//! Only one capture is alive at a time. A new one replaces the old, and
//! the old temp file goes with it, so `%TEMP%/minshot` never grows.
//!
//! The pixels themselves are not kept: the PNG has them, and they are
//! only needed when the user asks for a copy on the clipboard.

use std::path::PathBuf;
use std::sync::Mutex;

use crate::files;

/// One capture: the PNG we still have in memory and where it landed
/// on disk.
pub struct LastCapture {
    /// The PNG bytes: the file on disk may be gone, and a clipboard copy
    /// needs the pixels.
    pub png: Vec<u8>,
    /// The file the capture was written to (the drag needs a path).
    pub path: PathBuf,
    /// Whether the file is ours to delete (a temp file) or the user's to
    /// keep (a saved screenshot).
    pub temporary: bool,
}

#[derive(Default)]
pub struct CaptureState {
    last: Mutex<Option<LastCapture>>,
}

impl CaptureState {
    /// Make `cap` the current capture and drop the previous one's file
    /// (unless that file is one the user asked us to keep).
    pub fn replace(&self, cap: LastCapture) {
        let old = {
            let mut guard = self.last.lock().expect("capture state poisoned");
            guard.replace(cap)
        };
        if let Some(old) = old {
            if old.temporary {
                files::remove_file_silent(&old.path);
            }
        }
    }

    /// Run `f` with the current capture, if any. The lock is held for the
    /// duration, so the caller can use the capture without copying it.
    pub fn with<R>(&self, f: impl FnOnce(Option<&LastCapture>) -> R) -> R {
        let guard = self.last.lock().expect("capture state poisoned");
        f(guard.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capture() -> LastCapture {
        files::ensure_dir().expect("temp dir failed");
        let path = files::unique_png_path();
        std::fs::write(&path, b"png").expect("write temp png failed");
        LastCapture {
            png: b"png".to_vec(),
            path,
            temporary: true,
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
        files::remove_file_silent(&second_path);
    }

    #[test]
    fn a_saved_capture_keeps_its_file() {
        let state = CaptureState::default();
        let mut first = capture();
        first.temporary = false;
        let first_path = first.path.clone();
        state.replace(first);
        let second = capture();
        let second_path = second.path.clone();
        state.replace(second);
        assert!(
            first_path.exists(),
            "a screenshot the user saved must not be deleted"
        );
        files::remove_file_silent(&first_path);
        files::remove_file_silent(&second_path);
    }

    #[test]
    fn the_first_capture_keeps_its_file() {
        let state = CaptureState::default();
        let first = capture();
        let path = first.path.clone();
        state.replace(first);
        assert!(path.exists());
        files::remove_file_silent(&path);
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
        assert_eq!(held, Some(second_path.clone()));
        assert_ne!(held, Some(first_path));
        files::remove_file_silent(&second_path);
    }
}
