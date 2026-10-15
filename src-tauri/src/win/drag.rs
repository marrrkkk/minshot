//! Native file-drag source (Windows CF_HDROP) via the `drag` crate.
//!
//! Terminals (Windows Terminal, conhost, WezTerm, …) do not accept bitmap
//! drops — a file drop lands as text (the file path). OpenCode's TUI reads a
//! dropped PNG/JPEG/GIF/WebP *path* as an image attachment, so dragging the
//! temp PNG file is the correct integration. True pixel transfer into a
//! terminal is not possible; clipboard paste (Ctrl+V) is the pixel path.

use std::path::PathBuf;

/// Start a blocking OS drag of `file`. Returns after the drop (or cancel).
/// Must be called with the preview window so the drag has a valid HWND context.
pub fn start_file_drag(window: &tauri::WebviewWindow, file: PathBuf) -> Result<(), String> {
    use drag::{DragItem, Image, Options};

    if !file.is_file() {
        return Err(format!("nothing to drag (missing {})", file.display()));
    }
    let image = Image::File(file.clone());
    drag::start_drag(
        window,
        DragItem::Files(vec![file]),
        image,
        |_result, _pos| {},
        Options::default(),
    )
    .map_err(|e| format!("native drag failed: {e}"))
}
