//! Where the PNG goes: the temp folder (deleted on the next capture) or a
//! folder the user picked (kept).
//!
//! A temp file exists only for integrations (file-drag to terminals) that
//! need a path; a saved screenshot is the user's file and stays where they
//! put it.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// `%TEMP%/minshot`
pub fn dir() -> PathBuf {
    std::env::temp_dir().join("minshot")
}

pub fn ensure_dir() -> std::io::Result<()> {
    std::fs::create_dir_all(dir())
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// `minshot-<pid>-<nanos>-<n>.png` — unique per capture (counter guarantees
/// it even when two captures land in the same clock tick) so a stale drop
/// target can never resolve to a newer screenshot.
pub fn unique_png_path() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    dir().join(format!("minshot-{}-{nanos}-{n}.png", std::process::id()))
}

/// The name a saved screenshot gets: `minshot-2026-10-20-15-34-22.png`.
/// Unique: a second capture in the same second gets a counter.
pub fn saved_png_path(dir: &Path) -> PathBuf {
    let (y, mo, d, h, mi, s) = local_stamp();
    let base = format!("minshot-{y:04}-{mo:02}-{d:02}-{h:02}-{mi:02}-{s:02}");
    let mut path = dir.join(format!("{base}.png"));
    let mut n = 2;
    while path.exists() {
        path = dir.join(format!("{base}-{n}.png"));
        n += 1;
    }
    path
}

/// The local date and time: (year, month, day, hour, minute, second).
fn local_stamp() -> (u16, u16, u16, u16, u16, u16) {
    #[cfg(target_os = "windows")]
    unsafe {
        let t = windows::Win32::System::SystemInformation::GetLocalTime();
        (t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        (
            1970,
            1,
            1,
            (secs / 3600) as u16,
            ((secs / 60) % 60) as u16,
            (secs % 60) as u16,
        )
    }
}

pub fn remove_file_silent(path: &Path) {
    let _ = std::fs::remove_file(path);
}

/// Delete our PNGs older than `max_age_secs` (best effort, runs at startup).
pub fn cleanup_stale(max_age_secs: u64) {
    let Ok(cutoff) = std::time::SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(max_age_secs))
        .ok_or(())
    else {
        return;
    };
    let Ok(entries) = std::fs::read_dir(dir()) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let is_ours = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("minshot-") && n.ends_with(".png"))
            .unwrap_or(false);
        if !is_ours {
            continue;
        }
        let old = entry
            .metadata()
            .and_then(|m| m.modified())
            .map(|t| t < cutoff)
            .unwrap_or(false);
        if old {
            remove_file_silent(&path);
        }
    }
}

/// Remove this process's PNGs (best effort, runs on app exit). Only our own
/// PID's files: a second instance's capture must survive ours quitting.
pub fn cleanup_all() {
    let prefix = format!("minshot-{}-", std::process::id());
    if let Ok(entries) = std::fs::read_dir(dir()) {
        for entry in entries.flatten() {
            let path = entry.path();
            let ours = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with(&prefix) && n.ends_with(".png"))
                .unwrap_or(false);
            if ours {
                remove_file_silent(&path);
            }
        }
    }
    // Succeeds only when no other instance's files remain.
    let _ = std::fs::remove_dir(dir());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_names_differ() {
        assert_ne!(unique_png_path(), unique_png_path());
    }

    #[test]
    fn temp_names_have_png_extension() {
        assert_eq!(
            unique_png_path().extension().and_then(|e| e.to_str()),
            Some("png")
        );
    }

    #[test]
    fn a_saved_screenshot_is_named_after_the_time() {
        let dir = std::env::temp_dir().join("minshot-test-saved");
        let _ = std::fs::create_dir_all(&dir);
        let path = saved_png_path(&dir);
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        assert!(name.starts_with("minshot-"), "unexpected name: {name}");
        assert!(name.ends_with(".png"), "unexpected name: {name}");
        // minshot-YYYY-MM-DD-HH-MM-SS.png
        assert_eq!(name.len(), "minshot-0000-00-00-00-00-00.png".len());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_saved_screenshot_never_overwrites_the_one_before_it() {
        let dir = std::env::temp_dir().join("minshot-test-unique");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create test dir failed");
        let first = saved_png_path(&dir);
        std::fs::write(&first, b"x").expect("write failed");
        let second = saved_png_path(&dir);
        assert_ne!(first, second);
        assert!(!second.exists(), "the second name must be free");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
