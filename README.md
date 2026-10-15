# minshot

Minimal Windows 11 screenshot-to-terminal utility. Rust + Tauri 2, vanilla TypeScript, no frameworks.

**The whole UX is three things:**

1. Press `Ctrl+Shift+4` → fullscreen region overlay (dim + drag a rectangle, `Esc` cancels).
2. A small always-on-top thumbnail appears with your screenshot.
3. Drag the thumbnail into a terminal, or press `Ctrl+V` — the image is already on the clipboard. A new capture replaces the thumbnail (no history, no save dialog).

## Terminal image input: what actually works

Researched against Windows Terminal, conhost, and OpenCode docs/behavior before implementing:

- **Terminals do not accept bitmap drops.** Dragging *pixels* (`CF_DIB`/`CF_BITMAP`) into Windows Terminal is impossible — a file drop lands as **text: the file path**. This is a terminal limitation, not something an app can work around.
- **OpenCode TUI turns a dropped PNG path into an image attachment.** Per OpenCode docs: *"You can also drag a supported image into a terminal that exposes the dropped file path to the TUI. The TUI reads PNG, JPEG, GIF, and WebP as image attachments."* So minshot drags a **temp PNG file** (`CF_HDROP`, native `DoDragDrop` via the `drag` crate). Dropping it on Windows Terminal inserts the quoted path, and OpenCode attaches it as an image.
- **Clipboard paste is the pixel path.** minshot auto-copies every capture as PNG/DIB to the Windows clipboard (`arboard`). `Ctrl+V` in OpenCode TUI attaches it as an image (OpenCode reads the clipboard itself; Windows Terminal forwards the paste keypress to the app). If a target only accepts text, paste does nothing — use the drag instead, or vice versa.

Honest summary: **drag = file path that OpenCode upgrades to an image; paste = actual image bytes.** Both end with the model seeing the screenshot. Neither inserts a path you must handle manually.

## Install dependencies

- Node 20+ and Rust stable (MSVC): `winget install Rustlang.Rustup`, then `rustup toolchain install stable-x86_64-pc-windows-msvc`
- MSVC linker + Windows SDK (for Tauri): Visual Studio 2022 BuildTools with the **VCTools** workload (note: on BuildTools the workload ID is `Microsoft.VisualStudio.Workload.VCTools`, not `NativeDesktop`):
  ```powershell
  winget install --id Microsoft.VisualStudio.2022.BuildTools -e
  # then, elevated:
  setup.exe modify --installPath "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools" `
    --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended `
    --add Microsoft.VisualStudio.Component.Windows11SDK.26100 --quiet --norestart
  ```
- WebView2 (preinstalled on Windows 11).

## Build, run, package

```powershell
npm install
npm run tauri dev      # run with hot reload
npm run tauri build    # → src-tauri/target/release/bundle/ (NSIS installer + portable exe)
cargo test --manifest-path src-tauri/Cargo.toml   # unit tests (rect math, temp names)
```

First `dev`/`build` compiles all Rust deps (one-time, several minutes).

## Use

- `Ctrl+Shift+4` anywhere → overlay. Drag a rectangle → thumbnail pops up. `Esc` cancels.
- **Drag the image** from the thumbnail into Windows Terminal / OpenCode → path is inserted and attached as an image.
- **`Ctrl+V`** where images are accepted → pastes the screenshot (no save step; PNG, auto-copied on capture).
- Drag the thumbnail's **title bar** to move it; **✕** (or `Esc` when focused) dismisses it.
- Tray icon: Capture region / Quit. Temp files live in `%TEMP%\minshot\` with unique names, replaced per capture, stale ones (>24h) pruned at startup, all removed on exit.

## Limitations (verified or by design)

- Multi-monitor works via one overlay spanning the virtual screen; capture is clamped to the monitor holding the selection center. Mixed-DPI setups: mapping uses the overlay window's scale, so a selection on a differently-scaled monitor can be off by the scale ratio. (Exact mixed-DPI would need per-monitor overlays — out of scope.)
- Drag into a terminal inserts a **file path**, per terminal design; only image-aware TUIs (OpenCode, etc.) upgrade it to an attachment. Plain shells will just show the path text.
- Clipboard holds one image; copying anything else replaces it.
- Single global shortcut, no configuration UI. If `Ctrl+Shift+4` collides with another app, use the tray menu.
