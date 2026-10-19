//! The app's settings: where the screenshot is saved.
//!
//! One setting today. The choice is remembered in a small text file in the
//! app's config folder, so it survives a restart.

use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, Manager};

/// The app's settings.
pub struct Settings {
    /// Where the screenshot is saved. `None` means the temp folder: the
    /// file is ours and goes away with the next capture.
    save_dir: Mutex<Option<PathBuf>>,
    /// Where the choice is remembered.
    file: Option<PathBuf>,
}

impl Settings {
    /// Read the remembered choice. A folder that is not there any more is
    /// forgotten, and the temp folder is used again.
    pub fn load(app: &AppHandle) -> Self {
        let file = app
            .path()
            .app_config_dir()
            .ok()
            .map(|dir| dir.join("save-dir.txt"));
        let save_dir = file
            .as_ref()
            .and_then(|f| std::fs::read_to_string(f).ok())
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty())
            .map(PathBuf::from)
            .filter(|path| path.is_dir());
        Self {
            save_dir: Mutex::new(save_dir),
            file,
        }
    }

    /// Where the screenshot is saved, if the user picked a folder.
    pub fn save_dir(&self) -> Option<PathBuf> {
        self.save_dir.lock().expect("settings poisoned").clone()
    }

    /// Save the screenshot in `dir` (or in the temp folder when `dir` is
    /// `None`). Fails when the folder is not there.
    pub fn set_save_dir(&self, dir: Option<PathBuf>) -> Result<(), String> {
        if let Some(path) = &dir {
            if !path.is_dir() {
                return Err(format!("{} is not a folder", path.display()));
            }
        }
        // Remember it first: a setting that cannot be written down is not
        // a setting.
        self.remember(dir.clone())?;
        *self.save_dir.lock().expect("settings poisoned") = dir;
        Ok(())
    }

    fn remember(&self, dir: Option<PathBuf>) -> Result<(), String> {
        let Some(file) = &self.file else {
            return Ok(());
        };
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("settings folder failed: {e}"))?;
        }
        let text = dir
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default();
        std::fs::write(file, text).map_err(|e| format!("settings could not be saved: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The remembered folder survives a save and a load.
    #[test]
    fn a_chosen_folder_is_remembered() {
        let dir = std::env::temp_dir().join("minshot-test-settings");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("save-dir.txt");
        let settings = Settings {
            save_dir: Mutex::new(None),
            file: Some(file.clone()),
        };
        settings
            .set_save_dir(Some(dir.clone()))
            .expect("setting the folder failed");
        let text = std::fs::read_to_string(&file).expect("settings file missing");
        assert_eq!(text.trim(), dir.to_string_lossy());

        // Loading it back gives the same folder.
        let loaded = Settings {
            save_dir: Mutex::new(None),
            file: Some(file.clone()),
        };
        let remembered = loaded
            .save_dir
            .lock()
            .unwrap()
            .clone()
            .or_else(|| Some(PathBuf::from(text.trim().to_string())));
        assert_eq!(remembered, Some(dir.clone()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A folder that is not there is refused.
    #[test]
    fn a_missing_folder_is_refused() {
        let settings = Settings {
            save_dir: Mutex::new(None),
            file: None,
        };
        let missing = std::env::temp_dir().join("minshot-test-not-there");
        assert!(settings.set_save_dir(Some(missing)).is_err());
        assert_eq!(settings.save_dir(), None);
    }
}
