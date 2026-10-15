# Minshot Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a minimal Windows 11 screenshot-to-terminal utility (Rust + Tauri 2) with global shortcut, region overlay, floating thumbnail, clipboard copy, and file-drag to terminal.

**Architecture:** Tauri 2 app with 3 windows (hidden main, fullscreen overlay, floating preview). Rust backend owns capture (xcap), clipboard (arboard), temp PNG files, global shortcut (tauri-plugin-global-shortcut), and native CF_HDROP drag source (windows crate). Vanilla TS frontend only renders overlay selection and preview thumbnail; all pixels stay in Rust until served as data-URL to preview.

**Tech Stack:** Rust (stable MSVC), Tauri 2, tauri-plugin-global-shortcut 2, xcap (capture), arboard (clipboard PNG), image (PNG encode), windows 0.58 (DoDragDrop CF_HDROP), uuid (temp names), vanilla TypeScript + Vite (no UI framework).

**Spec:** User request in session (minshot: region capture Ctrl+Shift+4, floating thumbnail, drag into terminal, clipboard PNG, temp files, no save dialog, no extra features).

## Global Constraints

- Backend: Rust. Desktop framework: Tauri 2. Frontend: TypeScript, minimal deps, no Electron.
- Keep Windows-specific behavior isolated in `src-tauri/src/win/` Rust module.
- Minimize Tauri capabilities/permissions; low idle CPU/memory.
- No editing, annotation, OCR, AI, cloud, accounts, gallery, history, multiple modes, extensive settings.
- New capture replaces existing preview; unique temp filenames under `%TEMP%/minshot/`; cleanup on replace/exit.
- Never require Save As; never claim untested drag behavior works.

## Review Focus

- Mixed-DPI multi-monitor selection maps to wrong pixels (overlay CSS px vs physical px) — expect pixel-accurate capture on 100%/125%/150% mixed setup.
- Zero-size or 1px drag selection should cancel, not produce corrupt PNG.
- Escape during overlay must cancel without leaving overlay visible or shortcut dead.
- Dragging thumbnail into Windows Terminal inserts a quoted file path, not pixels — expect OpenCode TUI to attach it as image only if PNG path is readable.
- Pasting (Ctrl+V) into non-image app does nothing — expect no crash and clipboard still holds PNG.

---

### Task 1: Toolchain + Tauri scaffold

**Files:**
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/build.rs`, `src-tauri/capabilities/default.json`, `src-tauri/src/main.rs`, `package.json`, `src/index.html` (main hidden), `src/overlay.html`, `src/preview.html`, `src/overlay.ts`, `src/preview.ts`
- Test: `scripts/verify-scaffold.ps1` (manual checklist, no unit test — scaffold has no logic)

**Interfaces:**
- Consumes: none.
- Produces: `cargo tauri dev` launches; windows `main` (hidden), `overlay` (hidden fullscreen transparent), `preview` (hidden 340px thumbnail).

- [ ] **Step 1: Install toolchain** — `winget install Rustlang.Rustup`, `rustup toolchain install stable-x86_64-pc-windows-msvc`, `winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.NativeDesktop --includeRecommended"`, verify `rustc --version`, `cargo --version`, `rustup show`.
- [ ] **Step 2: Scaffold npm+Tauri 2** — `npm create tauri-app@latest` equivalent manually (or `cargo install tauri-cli@2`); keep `tauri.conf.json` with `app.windows`: main hidden, overlay fullscreen transparent decorations:false, preview decorations:false alwaysOnTop:true resizable:false.
- [ ] **Step 3: Verify scaffold builds** — Run: `npm install; npm run tauri build` Expected: PASS (bundle produced, no Rust errors).
- [ ] **Step 4: Commit** — `git add -A; git commit -m "chore: tauri 2 scaffold with overlay+preview windows"`

### Task 2: Global shortcut + overlay show/hide/cancel

**Files:**
- Modify: `src-tauri/src/main.rs`, `src-tauri/capabilities/default.json`, `src/overlay.ts`, `src/overlay.html`
- Test: manual — press Ctrl+Shift+4 shows overlay on all monitors; Esc hides.

**Interfaces:**
- Consumes: Task 1 windows.
- Produces: `show_overlay()`, `hide_overlay()` (Rust commands); frontend `overlay.ts` reports `cancel_capture()` on Escape.

- [ ] **Step 1: Register Ctrl+Shift+4** using `tauri-plugin-global-shortcut` in Rust; on trigger, show `overlay` window fullscreen, focus it.
- [ ] **Step 2: Overlay dim + Esc** — `overlay.html` fullscreen rgba(0,0,0,0.5), `overlay.ts` keydown Escape → invoke `hide_overlay`.
- [ ] **Step 3: Verify manually** — Run `npm run tauri dev`, press shortcut, Esc. Expected: overlay appears/dismisses, app idles <1% CPU.
- [ ] **Step 4: Commit**

### Task 3: Region selection + pixel-accurate capture (multi-monitor/DPI)

**Files:**
- Create: `src-tauri/src/capture.rs`, `src-tauri/src/win/mod.rs` (monitor enum via windows crate)
- Modify: `src-tauri/src/main.rs`, `src/overlay.ts`
- Test: `src-tauri` unit test `capture::tests::rect_normalize` (zero-size rejected, inverted drag normalized).

**Interfaces:**
- Consumes: Task 2 overlay visibility.
- Produces: `capture_region(x: f64, y: f64, w: f64, h: f64, scale: f64) -> { png_base64, width, height, temp_path }`; `win::virtual_screen()` returns monitor list with scale.

- [ ] **Step 1: Write failing test** `rect_normalize` — inverted coords normalize, zero-size returns Err.
- [ ] **Step 2: Implement `capture.rs`** with `xcap::Monitor::all()`, map overlay CSS px × scale → physical px, crop, PNG encode via `image` crate, write `%TEMP%/minshot/minshot-<uuid>.png`, return base64.
- [ ] **Step 3: Wire overlay drag** — `overlay.ts` mousedown/mousemove/mouseup draws rect div, on mouseup invokes `capture_region` with `window.devicePixelRatio`.
- [ ] **Step 4: Verify** — `cargo test`, plus manual capture on primary monitor; check PNG exists and dimensions match selection.
- [ ] **Step 5: Commit**

### Task 4: Floating preview thumbnail (replace, drag-handle, dismiss)

**Files:**
- Modify: `src-tauri/src/main.rs`, `src/preview.html`, `src/preview.ts`
- Test: manual — second capture replaces image, no history growth; close button hides.

**Interfaces:**
- Consumes: Task 3 `capture_region` result.
- Produces: `get_last_capture() -> { png_base64, temp_path } | null`; preview window shows `<img>`, draggable via `data-tauri-drag-region`, close via `hide_preview`.

- [ ] **Step 1: Show preview after capture** — Rust emits `capture-ready` event with base64+path; `preview.ts` renders `<img>`, auto-sizes window (~320px wide, aspect preserved).
- [ ] **Step 2: Replace + dismiss** — new capture overwrites `LAST_CAPTURE` + deletes old temp file; `X` button and `Esc` in preview invoke `hide_preview`.
- [ ] **Step 3: Verify manually** — capture twice, confirm `%TEMP%/minshot` holds exactly 1 PNG; drag titlebar moves window; X dismisses.
- [ ] **Step 4: Commit**

### Task 5: Clipboard PNG copy (auto) + temp-file lifecycle

**Files:**
- Create: `src-tauri/src/clipboard.rs`, `src-tauri/src/tempfiles.rs`
- Modify: `src-tauri/src/main.rs`, `src-tauri/src/capture.rs`
- Test: `tempfiles::tests::unique_names_differ`; manual paste into Paint + OpenCode TUI.

**Interfaces:**
- Consumes: Task 3 PNG bytes.
- Produces: `copy_last_to_clipboard()` (auto-called after capture); temp dir `%TEMP%/minshot`, cleanup on replace/exit.

- [ ] **Step 1: Implement clipboard** with `arboard::Clipboard::set_image(arboard::ImageData)` from decoded PNG RGBA.
- [ ] **Step 2: Implement tempfiles** — `unique_path()`, `cleanup_old()`, `cleanup_all_on_exit()`.
- [ ] **Step 3: Verify** — `cargo test`; manual: capture → open Paint Ctrl+V shows image; `Ctrl+V` in OpenCode TUI attaches image (if supported).
- [ ] **Step 4: Commit**

### Task 6: Native file-drag from thumbnail (CF_HDROP) + docs

**Files:**
- Create: `src-tauri/src/win/drag.rs` (IDataObject + DoDragDrop with CF_HDROP to temp PNG)
- Modify: `src-tauri/src/main.rs`, `src/preview.ts`, `README.md`, `src-tauri/tauri.conf.json` (if needed for fileDrop)
- Test: manual drag into Windows Terminal + OpenCode TUI + Explorer.

**Interfaces:**
- Consumes: Task 4 preview + Task 5 temp PNG.
- Produces: `start_native_drag()` Rust command (blocking OLE drag); `preview.ts` calls it on thumbnail dragstart; README documents actual behavior.

- [ ] **Step 1: Implement `win/drag.rs`** — `ShellIds`: `CF_HDROP` via `windows::Win32::System::Ole::DoDragDrop`, `DROPEFFECT_COPY`, runs on dedicated thread (OLE requires STA).
- [ ] **Step 2: Wire preview drag** — thumbnail `draggable=true`, on `dragstart` preventDefault + invoke `start_native_drag`; cursor feedback; fallback hint "If drop inserts path, press Ctrl+V in OpenCode".
- [ ] **Step 3: Verify honestly** — drag into Windows Terminal running OpenCode: record whether path text inserted vs image attached; drag into Explorer (should copy file); paste Ctrl+V. Write results in README + final report. Never claim pixels transferred if only path observed.
- [ ] **Step 4: Verify distinguishes impl vs verified** — if headless/CI cannot test, label as implemented-not-verified.
- [ ] **Step 5: Commit**
