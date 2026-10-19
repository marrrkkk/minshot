//! Temporary PNG lifecycle: unique names, replace-on-capture, stale cleanup.
//! Screenshots live in memory; a temp file exists only for integrations
//! (file-drag to terminals) that require a path.

use std::path::PathBuf;
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

pub fn remove_file_silent(path: &PathBuf) {
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
}
