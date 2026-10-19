//! minshot — minimal Windows screenshot-to-terminal utility.
//!
//! UX: global shortcut -> fullscreen region overlay -> floating thumbnail.
//! The thumbnail can be dragged (native file drop of a temp PNG) into a
//! terminal, and the image is always on the clipboard for Ctrl+V.

mod capture;
mod clipboard_img;
mod files;
mod last;
mod settings;
mod win;

use std::path::PathBuf;
use std::sync::Mutex;
use tauri::window::Monitor;
use tauri::{AppHandle, Emitter, Manager, RunEvent, WebviewWindow};

use last::CaptureState;
use settings::Settings;

/// The global shortcut the app registers. The launcher reads it back, so the
/// hint on screen can never drift from the real binding.
const SHORTCUT: &str = "Ctrl+Shift+4";

/// The smallest window the app fits in (logical px, client area). Below this
/// the toolbar and its actions no longer fit side by side.
const MIN_CLIENT_W: f64 = 224.0;
const MIN_CLIENT_H: f64 = 110.0;

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

/// Where the shot window belongs: the work area of the monitor a capture
/// came from (physical pixels), plus the window frame that eats into it.
#[derive(Clone, Copy, Debug)]
struct WorkArea {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    /// Physical pixels per logical pixel on that monitor.
    scale: f64,
    /// The window frame (border + title bar) in physical pixels.
    frame_w: u32,
    frame_h: u32,
}

impl WorkArea {
    /// The logical space a window gets inside this work area.
    fn avail(&self) -> (f64, f64) {
        (
            self.width.saturating_sub(self.frame_w) as f64 / self.scale,
            self.height.saturating_sub(self.frame_h) as f64 / self.scale,
        )
    }
}

/// The app's shared state: the last capture, the workspace the shot
/// window belongs to (the work area of the monitor the capture came from,
/// remembered between the capture and the fit that follows it), and the
/// settings.
struct AppState {
    /// The last capture (no history).
    capture: CaptureState,
    /// Where the shot window belongs.
    work_area: Mutex<Option<WorkArea>>,
    /// Where the screenshot is saved.
    settings: Settings,
}

impl AppState {
    fn set_work_area(&self, area: Option<WorkArea>) {
        *self.work_area.lock().expect("work area poisoned") = area;
    }

    fn work_area(&self) -> Option<WorkArea> {
        *self.work_area.lock().expect("work area poisoned")
    }
}

/// The work area of a monitor, with the frame measured from the main window.
/// The frame is the same on every monitor of the same scale; a mixed-DPI
/// desktop can be a pixel or two off.
fn work_area_of(app: &AppHandle, monitor: Monitor) -> WorkArea {
    let (frame_w, frame_h) = main_window(app)
        .ok()
        .and_then(|w| {
            let outer = w.outer_size().ok()?;
            let inner = w.inner_size().ok()?;
            Some((
                outer.width.saturating_sub(inner.width),
                outer.height.saturating_sub(inner.height),
            ))
        })
        .unwrap_or((0, 0));
    let area = monitor.work_area();
    WorkArea {
        x: area.position.x,
        y: area.position.y,
        width: area.size.width,
        height: area.size.height,
        scale: monitor.scale_factor(),
        frame_w,
        frame_h,
    }
}

/// The work area of the monitor showing a physical point.
fn work_area_at(app: &AppHandle, px: i32, py: i32) -> Option<WorkArea> {
    let main = main_window(app).ok()?;
    let monitors = main.available_monitors().ok()?;
    let monitor = monitors
        .iter()
        .find(|m| {
            let p = m.position();
            let s = m.size();
            px >= p.x && px < p.x + s.width as i32 && py >= p.y && py < p.y + s.height as i32
        })
        .cloned()
        .or_else(|| main.current_monitor().ok().flatten())?;
    Some(work_area_of(app, monitor))
}

/// The work area of the monitor the main window is on.
fn work_area_of_window(app: &AppHandle) -> Option<WorkArea> {
    let monitor = main_window(app).ok()?.current_monitor().ok().flatten()?;
    Some(work_area_of(app, monitor))
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
        // A minimized window is frozen: its webview gets no events, and the
        // capture that follows would never reach the frontend. Bring it
        // back to life before hiding it.
        let _ = main.unminimize();
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

fn show_main_inner(app: &AppHandle, focus: bool) -> Result<(), String> {
    let main = main_window(app)?;
    // A minimized window stays minimized when it is shown: the shot has
    // to come up, so restore it first.
    let _ = main.unminimize();
    main.show().map_err(|e| e.to_string())?;
    if focus {
        main.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn hide_overlay(app: AppHandle) -> Result<(), String> {
    overlay_window(&app)?.hide().map_err(|e| e.to_string())
}

/// Show the main window. `focus` is false for the shot view: the user is
/// usually headed back to the terminal to paste, and a focus steal would
/// take them out of it.
#[tauri::command]
fn show_main(app: AppHandle, focus: Option<bool>) -> Result<(), String> {
    show_main_inner(&app, focus.unwrap_or(true))
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

/// Copy the last capture to the clipboard.
///
/// Async on purpose: the PNG is decoded and pushed into the clipboard,
/// which takes a moment for a big capture. The window must not freeze
/// while it happens.
#[tauri::command(async)]
fn copy_to_clipboard(state: tauri::State<'_, AppState>) -> Result<(), String> {
    // The lock is held while the pixels are decoded and copied out: no
    // clone of the capture needed.
    state.capture.with(|last| {
        let last = last.ok_or("no capture yet")?;
        let cap = capture::decode_png(&last.png)?;
        clipboard_img::copy_rgba_to_clipboard(&cap.rgba, cap.width, cap.height)
    })
}

/// The global shortcut could not be registered (another app has it). The
/// frontend asks for this at startup and warns the user.
#[tauri::command]
fn shortcut_error(status: tauri::State<'_, AppStatus>) -> Option<String> {
    status.shortcut_error()
}

/// The global shortcut the app registers, so the UI can show the real one.
#[tauri::command]
fn shortcut() -> &'static str {
    SHORTCUT
}

/// Where the screenshot is saved: the folder the user picked, or nothing
/// for the temp folder.
#[tauri::command]
fn get_settings(state: tauri::State<'_, AppState>) -> serde_json::Value {
    serde_json::json!({
        "saveDir": state.settings.save_dir().map(|p| p.to_string_lossy().to_string()),
    })
}

/// Save the screenshot in this folder (or in the temp folder when the
/// folder is empty). The choice is remembered between runs.
#[tauri::command]
fn set_save_dir(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    dir: Option<String>,
) -> Result<serde_json::Value, String> {
    let path = dir
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .map(PathBuf::from);
    state.settings.set_save_dir(path.clone())?;
    if let Some(path) = &path {
        // The frontend loads the shot through the asset protocol, so the
        // folder has to be readable there too.
        app.asset_protocol_scope()
            .allow_directory(path, false)
            .map_err(|e| e.to_string())?;
    }
    Ok(serde_json::json!({
        "saveDir": path.map(|p| p.to_string_lossy().to_string()),
    }))
}

/// Ask the user for a folder (the Windows folder picker).
#[tauri::command]
fn pick_save_dir() -> Option<String> {
    win::picker::pick_folder().map(|p| p.to_string_lossy().to_string())
}

/// Resize the main window to a content size (logical px) and place it in
/// the work area of the monitor the last capture came from.
///
/// The content size is what the webview needs; the window frame (border and
/// title bar) is added on top by Windows, so it is subtracted from the
/// workspace first and added back when the window is centred.
#[tauri::command]
fn fit_main(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let main = main_window(&app)?;
    let work = state
        .work_area()
        .or_else(|| work_area_of_window(&app))
        .ok_or_else(|| "no monitor found".to_string())?;
    // The client area in physical pixels: the webview's CSS px map onto it
    // through the monitor's scale factor.
    let max_w = work.width.saturating_sub(work.frame_w).max(1);
    let max_h = work.height.saturating_sub(work.frame_h).max(1);
    // Never smaller than the toolbar needs, never bigger than the workspace.
    let min_w = ((MIN_CLIENT_W * work.scale).round() as u32)
        .min(max_w)
        .max(1);
    let min_h = ((MIN_CLIENT_H * work.scale).round() as u32)
        .min(max_h)
        .max(1);
    let w = ((width * work.scale).round() as u32).clamp(min_w, max_w);
    let h = ((height * work.scale).round() as u32).clamp(min_h, max_h);
    main.set_size(tauri::Size::Physical(tauri::PhysicalSize {
        width: w,
        height: h,
    }))
    .map_err(|e| e.to_string())?;
    // Centre the whole window (frame included) in the work area, so it never
    // hangs under a taskbar or off the edge of the screen.
    let x = work.x + (work.width.saturating_sub(w + work.frame_w) / 2) as i32;
    let y = work.y + (work.height.saturating_sub(h + work.frame_h) / 2) as i32;
    main.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Hide the overlay and wait until the screen really shows without it.
///
/// The dim is a transparent window: the compositor has to take it off the
/// screen before the grab, or the shot comes out dimmed. Asking the
/// compositor to finish its frame is enough — no need to guess a delay.
/// shortcut: 40ms can miss on a slow GPU, raise it if a dim shows in the shot.
fn hide_overlay_and_wait(app: &AppHandle) {
    let (tx, rx) = std::sync::mpsc::channel();
    let owned = app.clone();
    if app
        .run_on_main_thread(move || {
            if let Ok(overlay) = overlay_window(&owned) {
                let _ = overlay.hide();
            }
            let _ = tx.send(());
        })
        .is_err()
    {
        return;
    }
    // The hide is queued on the main thread: give it a moment to land.
    let _ = rx.recv_timeout(std::time::Duration::from_secs(2));
    std::thread::sleep(std::time::Duration::from_millis(30));
    // Then let the compositor finish the frame without the overlay.
    win::screen::wait_for_compositor();
    std::thread::sleep(std::time::Duration::from_millis(10));
}

/// Take a region screenshot.
///
/// The work is blocking (a screen grab, a PNG encode), so it goes to a
/// blocking thread: the app stays responsive (tray, shortcut) while a 4K
/// capture is grabbed and encoded.
#[tauri::command(async)]
async fn capture_region(
    app: AppHandle,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    scale: f64,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || capture_region_now(&app, x, y, w, h, scale))
        .await
        .map_err(|e| format!("capture task failed: {e}"))?
}

/// The capture itself, on a blocking thread.
fn capture_region_now(
    app: &AppHandle,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    scale: f64,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    // Hide the overlay first so the dim layer is not part of the screenshot,
    // and wait for the compositor to actually take it off the screen.
    hide_overlay_and_wait(app);

    let (cap, phys) = match capture::capture_css_rect(x, y, w, h, scale) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[minshot] capture failed: {e}");
            // The user gets a short message, not the raw error.
            let _ = app.emit_to(
                "main",
                "capture-failed",
                serde_json::json!({ "message": "Capture failed. Try again." }),
            );
            return Err(e);
        }
    };
    let png = capture::encode_png(&cap.rgba, cap.width, cap.height)?;

    // Clipboard copy is the primary terminal workflow (Ctrl+V in OpenCode),
    // so it happens first: a folder that cannot be written must not cost
    // the user the picture. Best effort: a clipboard failure must not lose
    // the capture either.
    let clipboard_ok =
        clipboard_img::copy_rgba_to_clipboard(&cap.rgba, cap.width, cap.height).is_ok();

    // The file: the temp folder (ours, it goes with the next capture) or
    // the folder the user picked (theirs, it stays).
    let (path, temporary) = match state.settings.save_dir() {
        Some(dir) => (files::saved_png_path(&dir), false),
        None => {
            files::ensure_dir().map_err(|e| format!("temp dir failed: {e}"))?;
            (files::unique_png_path(), true)
        }
    };
    if let Err(e) = std::fs::write(&path, &png) {
        eprintln!("[minshot] could not write {path:?}: {e}");
        let _ = app.emit_to(
            "main",
            "capture-failed",
            serde_json::json!({
                "message": "Can't save the screenshot. Check the save folder.",
            }),
        );
        return Err(format!("write PNG failed: {e}"));
    }

    // Replace the previous capture (no history).
    state.capture.replace(last::LastCapture {
        png,
        path: path.clone(),
        temporary,
    });

    // The shot window belongs on the monitor the capture came from: that is
    // the workspace the image has to fit into.
    let cx = phys.0 + phys.2 as i32 / 2;
    let cy = phys.1 + phys.3 as i32 / 2;
    let work = work_area_at(app, cx, cy);
    state.set_work_area(work);
    let (scale, avail_width, avail_height) = match work {
        Some(w) => {
            let (aw, ah) = w.avail();
            (w.scale, aw, ah)
        }
        None => (1.0, 1280.0, 800.0),
    };

    // The frontend loads the shot from this path (asset protocol), so no
    // base64 copy of the PNG ever crosses IPC. It also sizes the window
    // and shows it, so the window never appears in the wrong shape.
    app.emit_to(
        "main",
        "capture-ready",
        serde_json::json!({
            "path": path.to_string_lossy(),
            "width": cap.width,
            "height": cap.height,
            "clipboard": clipboard_ok,
            "scale": scale,
            "availWidth": avail_width,
            "availHeight": avail_height,
        }),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn start_native_drag(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let path = state
        .capture
        .with(|last| last.ok_or("no capture yet").map(|l| l.path.clone()))?;
    if !path.is_file() {
        // Temp file lost after capture (external cleaners, AV): rebuild it
        // from the PNG we still hold.
        let png = state.capture.with(|last| {
            let last = last.ok_or("no capture yet")?;
            Ok::<_, String>(last.png.clone())
        })?;
        std::fs::write(&path, &png).map_err(|e| format!("rewrite PNG failed: {e}"))?;
    }
    win::drag::start_file_drag_main_thread(app, path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    files::cleanup_stale(24 * 3600);

    let mut builder = tauri::Builder::default().manage(AppStatus::default());

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
            // The settings need the app handle (they live in its config
            // folder), so the state is built here.
            let settings = Settings::load(app.handle());
            if let Some(dir) = settings.save_dir() {
                // The frontend loads the shot through the asset protocol.
                let _ = app.asset_protocol_scope().allow_directory(&dir, false);
            }
            app.manage(AppState {
                capture: CaptureState::default(),
                work_area: Mutex::new(None),
                settings,
            });

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
                        let _ = show_main_inner(app, true);
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
            shortcut,
            shortcut_error,
            fit_main,
            hide_overlay,
            capture_region,
            start_native_drag,
            get_settings,
            set_save_dir,
            pick_save_dir
        ]);

    let app = builder
        .build(tauri::generate_context!())
        .expect("failed to build minshot");

    // Register the global shortcut before the event loop starts. The
    // windows are created later (when the app is ready), and the launcher
    // asks for the shortcut state as soon as it loads — so the answer has
    // to be final by then. If the shortcut collides with another app,
    // capture is still available from the tray menu, but the user has to
    // know, so the error is shown in the window.
    #[cfg(desktop)]
    {
        use tauri_plugin_global_shortcut::GlobalShortcutExt;
        if let Err(e) = app.global_shortcut().register(SHORTCUT) {
            eprintln!("[minshot] shortcut register failed: {e}");
            app.state::<AppStatus>().set_shortcut_error(e.to_string());
        }
    }

    app.run(|_app, event| {
        if matches!(event, RunEvent::Exit) {
            files::cleanup_all();
        }
    });
}
