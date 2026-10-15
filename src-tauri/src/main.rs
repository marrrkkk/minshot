//! minshot — minimal Windows screenshot-to-terminal utility.
//!
//! UX: global shortcut -> fullscreen region overlay -> floating thumbnail.
//! The thumbnail can be dragged (native file drop of a temp PNG) into a
//! terminal, and the image is always on the clipboard for Ctrl+V.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod clipboard_img;
mod tempfiles;
mod win;

use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, RunEvent, WebviewWindow};

struct LastCapture {
    path: std::path::PathBuf,
    png: Vec<u8>,
}

#[derive(Default)]
struct AppState {
    last: Mutex<Option<LastCapture>>,
}

fn overlay_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    app.get_webview_window("overlay")
        .ok_or_else(|| "overlay window missing".to_string())
}

fn preview_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    app.get_webview_window("preview")
        .ok_or_else(|| "preview window missing".to_string())
}

fn show_overlay_inner(app: &AppHandle) -> Result<(), String> {
    let overlay = overlay_window(app)?;
    // Span the whole virtual screen so every monitor is selectable.
    let (vx, vy, vw, vh) = win::virtual_screen_rect();
    overlay
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x: vx, y: vy }))
        .map_err(|e| e.to_string())?;
    overlay
        .set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: vw,
            height: vh,
        }))
        .map_err(|e| e.to_string())?;
    overlay.show().map_err(|e| e.to_string())?;
    overlay.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn show_overlay(app: AppHandle) -> Result<(), String> {
    show_overlay_inner(&app)
}

#[tauri::command]
fn hide_overlay(app: AppHandle) -> Result<(), String> {
    overlay_window(&app)?.hide().map_err(|e| e.to_string())
}

#[tauri::command]
fn hide_preview(app: AppHandle) -> Result<(), String> {
    preview_window(&app)?.hide().map_err(|e| e.to_string())
}

#[tauri::command]
fn capture_region(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    scale: f64,
) -> Result<serde_json::Value, String> {
    // Hide the overlay first so the dim layer is not part of the screenshot,
    // and give the compositor a beat to redraw without it.
    if let Ok(overlay) = overlay_window(&app) {
        let _ = overlay.hide();
    }
    std::thread::sleep(std::time::Duration::from_millis(180));

    let cap = capture::capture_css_rect(x, y, w, h, scale)?;

    // Temp PNG (created only because file-drag integrations need a path).
    tempfiles::ensure_dir().map_err(|e| format!("temp dir failed: {e}"))?;
    let path = tempfiles::unique_png_path();
    std::fs::write(&path, &cap.png_bytes).map_err(|e| format!("write temp PNG failed: {e}"))?;

    // Clipboard copy is the primary terminal workflow (Ctrl+V in OpenCode).
    // Best effort: a clipboard failure must not lose the capture.
    let clipboard_ok = clipboard_img::copy_png_to_clipboard(&cap.png_bytes).is_ok();

    // Replace the previous capture (no history).
    if let Ok(mut guard) = state.last.lock() {
        if let Some(old) = guard.replace(LastCapture {
            path: path.clone(),
            png: cap.png_bytes.clone(),
        }) {
            tempfiles::remove_file_silent(&old.path);
        }
    }

    // Show the floating thumbnail, sized to the shot's aspect ratio.
    const PREVIEW_W: u32 = 360;
    const CHROME_H: u32 = 96; // title bar + footer
    let aspect_h = ((cap.height as f64 / cap.width.max(1) as f64) * PREVIEW_W as f64) as u32;
    let preview = preview_window(&app)?;
    preview
        .set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: PREVIEW_W as f64,
            height: (aspect_h + CHROME_H) as f64,
        }))
        .map_err(|e| e.to_string())?;
    preview.show().map_err(|e| e.to_string())?;

    app.emit_to(
        "preview",
        "capture-ready",
        serde_json::json!({
            "base64": base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &cap.png_bytes,
            ),
            "width": cap.width,
            "height": cap.height,
            "clipboard": clipboard_ok,
        }),
    )
    .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "width": cap.width,
        "height": cap.height,
        "path": path.to_string_lossy(),
        "clipboard": clipboard_ok,
    }))
}

#[tauri::command]
fn start_native_drag(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let path = {
        let guard = state.last.lock().map_err(|e| e.to_string())?;
        guard
            .as_ref()
            .map(|l| l.path.clone())
            .ok_or_else(|| "no capture yet".to_string())?
    };
    win::drag::start_file_drag_main_thread(app, path)
}

fn main() {
    tempfiles::cleanup_stale(24 * 3600);

    let mut builder = tauri::Builder::default().manage(AppState::default());

    #[cfg(desktop)]
    {
        use tauri_plugin_global_shortcut::ShortcutState;

        builder = builder.plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let _ = show_overlay_inner(app);
                    }
                })
                .build(),
        );
    }

    builder = builder
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::GlobalShortcutExt;
                // Preferred shortcut. If it collides, capture is still
                // available from the tray menu.
                if let Err(e) = app.global_shortcut().register("Ctrl+Shift+4") {
                    eprintln!("[minshot] shortcut register failed: {e}");
                }
            }

            // Minimal tray: Capture + Quit (the main window stays hidden).
            use tauri::menu::{IsMenuItem, Menu, MenuItem};
            let capture_item =
                MenuItem::with_id(app, "capture", "Capture region", true, None::<&str>)?;
            let quit_item =
                MenuItem::with_id(app, "quit", "Quit minshot", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[
                    &capture_item as &dyn IsMenuItem<tauri::Wry>,
                    &quit_item as &dyn IsMenuItem<tauri::Wry>,
                ],
            )?;
            let icon = app
                .default_window_icon()
                .cloned()
                .ok_or("app icon missing (run `tauri icon` first)")?;
            let _tray = tauri::tray::TrayIconBuilder::with_id("main")
                .icon(icon)
                .menu(&menu)
                .tooltip("minshot — Ctrl+Shift+4 to capture")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "capture" => {
                        let _ = show_overlay_inner(app);
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            show_overlay,
            hide_overlay,
            hide_preview,
            capture_region,
            start_native_drag
        ]);

    builder
        .build(tauri::generate_context!())
        .expect("failed to build minshot")
        .run(|_app, event| {
            if matches!(event, RunEvent::Exit) {
                tempfiles::cleanup_all();
            }
        });
}
