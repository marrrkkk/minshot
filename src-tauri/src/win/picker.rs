//! The "choose a folder" dialog: the Windows shell's own folder picker.
//!
//! Hand-rolled on purpose: the app ships no extra dependency for it, and
//! the shell dialog is a handful of COM calls.

use std::path::PathBuf;

/// Ask the user for a folder. `None` when they cancel.
pub fn pick_folder() -> Option<PathBuf> {
    crate::win::com::with_ole(pick_folder_impl)
}

#[cfg(target_os = "windows")]
fn pick_folder_impl() -> Option<PathBuf> {
    use windows::core::w;
    use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_INPROC_SERVER};
    use windows::Win32::UI::Shell::{
        FileOpenDialog, IFileOpenDialog, FOS_FORCEFILESYSTEM, FOS_PATHMUSTEXIST, FOS_PICKFOLDERS,
        SIGDN_FILESYSPATH,
    };

    unsafe {
        let dialog: IFileOpenDialog =
            CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER).ok()?;
        let _ = dialog.SetOptions(FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST);
        let _ = dialog.SetTitle(w!("Choose where minshot saves screenshots"));
        // Show() reports an error when the user closes the dialog without
        // picking anything: that is a cancel, not a failure.
        if dialog.Show(None).is_err() {
            return None;
        }
        let item = dialog.GetResult().ok()?;
        let name = item.GetDisplayName(SIGDN_FILESYSPATH).ok()?;
        let path = name.to_string().ok();
        CoTaskMemFree(Some(name.0.cast()));
        path.map(PathBuf::from)
    }
}

#[cfg(not(target_os = "windows"))]
fn pick_folder_impl() -> Option<PathBuf> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The folder picker must actually put a dialog on the screen.
    /// Needs a desktop session: `cargo test -- --include-ignored` runs it.
    #[test]
    #[ignore = "needs an interactive desktop session"]
    fn the_picker_opens_a_dialog() {
        let handle = std::thread::spawn(pick_folder);
        std::thread::sleep(std::time::Duration::from_secs(3));
        // Close the dialog again, so the test does not wait for a person.
        close_the_dialog();
        let picked = handle.join().expect("picker panicked");
        assert_eq!(picked, None, "a closed dialog picks nothing");
    }

    /// Close the picker dialog by name.
    fn close_the_dialog() {
        use windows::core::w;
        use windows::Win32::Foundation::{LPARAM, WPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, PostMessageW, WM_CLOSE};
        unsafe {
            let title = w!("Choose where minshot saves screenshots");
            if let Ok(window) = FindWindowW(w!("#32770"), title) {
                let _ = PostMessageW(Some(window), WM_CLOSE, WPARAM(0), LPARAM(0));
            }
        }
    }
}
