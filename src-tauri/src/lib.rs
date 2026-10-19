//! minshot — minimal Windows screenshot-to-terminal utility.
//!
//! UX: global shortcut -> fullscreen region overlay -> floating thumbnail.
//! The thumbnail can be dragged (native file drop of a temp PNG) into a
//! terminal, and the image is always on the clipboard for Ctrl+V.

mod capture;
mod clipboard_img;
mod last;
mod tempfiles;
mod win;

use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, RunEvent, WebviewWindow};

use last::CaptureState;

/// App-level status the frontend asks about at startup.
#[derive(Default)]
struct AppStatus {
    shortcut_error: Mutex<Option<String>>,
}

impl AppStatus {
    fn set_shortcut_error(&self, msg: String) {
        *self.shortcut_error.lock().expect("app status poisoned") = Some(msg);
    }

    fn shortcut_error(&self) -> Option<String> {
        self.shortcut_error
            .lock()
            .expect("app status poisoned")
            .clone()
    }
}

fn overlay_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    app.get_webview_window("overlay")
        .ok_or_else(|| "overlay window missing".to_string())
}

fn main_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    app.get_webview_window("main")
        .ok_or_else(|| "main window missing".to_string())
}

fn show_overlay_inner(app: &AppHandle) -> Result<(), String> {
    // Single-window UX: hide the main window while selecting, so it never
    // covers the target area (or ends up in its own screenshot).
    if let Ok(main) = main_window(app) {
        let _ = main.hide();
    }
    let overlay = overlay_window(app)?;
    // Span the whole virtual screen so every monitor is selectable.
    let (vx, vy, vw, vh) = win::virtual_screen_rect();
    overlay
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: vx,
            y: vy,
        }))
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

fn show_main_inner(app: &AppHandle) -> Result<(), String> {
    let main = main_window(app)?;
    main.show().map_err(|e| e.to_string())?;
    main.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn hide_overlay(app: AppHandle) -> Result<(), String> {
    overlay_window(&app)?.hide().map_err(|e| e.to_string())
}

#[tauri::command]
fn show_main(app: AppHandle) -> Result<(), String> {
    show_main_inner(&app)
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn copy_to_clipboard(state: tauri::State<'_, CaptureState>) -> Result<(), String> {
    // The lock is held while the pixels are copied out: no clone of the
    // capture needed.
    state.with(|last| {
        let last = last.ok_or("no capture yet")?;
        clipboard_img::copy_rgba_to_clipboard(&last.rgba, last.width, last.height)
    })
}

/// The global shortcut could not be registered (another app has it). The
/// frontend asks for this at startup and warns the user.
#[tauri::command]
fn shortcut_error(status: tauri::State<'_, AppStatus>) -> Option<String> {
    status.shortcut_error()
}

/// Resize the main window to exactly fit its rendered content (logical px,
/// measured by the frontend after the image loads). Kills the "extra space at
/// the bottom" class of bugs: the window is only ever as tall as its content.
#[tauri::command]
fn fit_main(app: AppHandle, width: f64, height: f64) -> Result<(), String> {
    // The lower bound keeps the action row (New capture / Copy) usable.
    let w = width.clamp(300.0, 900.0);
    let h = height.clamp(120.0, 800.0);
    main_window(&app)?
        .set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: w,
            height: h,
        }))
        .map_err(|e| e.to_string())
}

/// Take a region screenshot.
///
/// `async` so the work happens on a runtime thread, not on the UI thread:
/// a 4K capture takes a while to grab and encode, and the app must stay
/// responsive (tray, shortcut) while it does.
#[tauri::command(async)]
fn capture_region(
    app: AppHandle,
    state: tauri::State<'_, CaptureState>,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    scale: f64,
) -> Result<(), String> {
    // Hide the overlay first so the dim layer is not part of the screenshot,
    // and give the compositor a beat to redraw without it.
    if let Ok(overlay) = overlay_window(&app) {
        let _ = overlay.hide();
    }
    // shortcut: 180ms compositor beat can miss on slow GPUs (dim remnant in shot), raise if seen.
    std::thread::sleep(std::time::Duration::from_millis(180));

    let cap = capture::capture_css_rect(x, y, w, h, scale)?;
    let png = capture::encode_png(&cap.rgba, cap.width, cap.height)?;

    // Temp PNG (created only because file-drag integrations need a path).
    tempfiles::ensure_dir().map_err(|e| format!("temp dir failed: {e}"))?;
    let path = tempfiles::unique_png_path();
    std::fs::write(&path, &png).map_err(|e| format!("write temp PNG failed: {e}"))?;

    // Clipboard copy is the primary terminal workflow (Ctrl+V in OpenCode).
    // Best effort: a clipboard failure must not lose the capture.
    let clipboard_ok =
        clipboard_img::copy_rgba_to_clipboard(&cap.rgba, cap.width, cap.height).is_ok();

    // Replace the previous capture (no history); the old temp file goes.
    state.replace(last::LastCapture {
        path: path.clone(),
        rgba: cap.rgba,
        width: cap.width,
        height: cap.height,
    });

    // Single-window UX: the main window reappears as the screenshot view.
    // No focus steal — the user is usually headed back to the terminal to
    // paste. The frontend resizes the window to fit via `fit_main`.
    main_window(&app)?.show().map_err(|e| e.to_string())?;

    // The frontend loads the shot from this path (asset protocol), so no
    // base64 copy of the PNG ever crosses IPC.
    app.emit_to(
        "main",
        "capture-ready",
        serde_json::json!({
            "path": path.to_string_lossy(),
            "width": cap.width,
            "height": cap.height,
            "clipboard": clipboard_ok,
        }),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn start_native_drag(app: AppHandle, state: tauri::State<'_, CaptureState>) -> Result<(), String> {
    let path = state.with(|last| last.ok_or("no capture yet").map(|l| l.path.clone()))?;
    if !path.is_file() {
        // Temp file lost after capture (external cleaners, AV): rebuild it
        // from the pixels we still hold.
        let (rgba, width, height) = state.with(|last| {
            let last = last.ok_or("no capture yet")?;
            Ok::<_, String>((last.rgba.clone(), last.width, last.height))
        })?;
        let png = capture::encode_png(&rgba, width, height)?;
        tempfiles::ensure_dir().map_err(|e| format!("temp dir failed: {e}"))?;
        std::fs::write(&path, &png).map_err(|e| format!("rewrite temp PNG failed: {e}"))?;
    }
    win::drag::start_file_drag_main_thread(app, path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tempfiles::cleanup_stale(24 * 3600);

    let mut builder = tauri::Builder::default()
        .manage(CaptureState::default())
        .manage(AppStatus::default());

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
                // available from the tray menu — but the user has to know,
                // so the error is shown in the window.
                if let Err(e) = app.global_shortcut().register("Ctrl+Shift+4") {
                    eprintln!("[minshot] shortcut register failed: {e}");
                    app.state::<AppStatus>().set_shortcut_error(e.to_string());
                }
            }

            // Minimal tray: Open + Capture + Quit.
            use tauri::menu::{IsMenuItem, Menu, MenuItem};
            let open_item = MenuItem::with_id(app, "open", "Open minshot", true, None::<&str>)?;
            let capture_item =
                MenuItem::with_id(app, "capture", "Capture region", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit minshot", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[
                    &open_item as &dyn IsMenuItem<tauri::Wry>,
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
                    "open" => {
                        let _ = show_main_inner(app);
                    }
                    "capture" => {
                        let _ = show_overlay_inner(app);
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the main window quits the whole app (Snipping-Tool-style:
            // no hidden background mode). Temp files are cleaned on exit.
            if window.label() == "main"
                && matches!(event, tauri::WindowEvent::CloseRequested { .. })
            {
                window.app_handle().exit(0);
            }
        })
        .invoke_handler(tauri::generate_handler![
            show_overlay,
            show_main,
            quit_app,
            copy_to_clipboard,
            shortcut_error,
            fit_main,
            hide_overlay,
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
