# minshot

Minimal Windows 11 screenshot-to-terminal utility. Rust + Tauri 2, vanilla TypeScript, no frameworks.

**The whole UX is one window:**

1. The **minshot window** (Snipping-Tool-style) with a **✂ New capture** button — or press `Ctrl+Shift+4` anywhere. The window hides itself while you select, so it never covers the target area.
2. Drag a rectangle on the fullscreen overlay (`Esc`/click cancels and brings the window back) → the same window reappears as the **screenshot view**, auto-fitted to the shot.
3. **Drag the screenshot** into a terminal (it shrinks and shows a "Release to drop" hint while held), or press `Ctrl+V` — the image is already on the clipboard. A new capture replaces the shot (no history, no save dialog).

Closing the window hides it to the tray (Open / Capture / Quit there).

## Terminal image input: what actually works

Researched against Windows Terminal, conhost, and OpenCode docs/behavior before implementing:

- **Terminals do not accept bitmap drops.** Dragging *pixels* (`CF_DIB`/`CF_BITMAP`) into Windows Terminal is impossible — a file drop lands as **text: the file path**. This is a terminal limitation, not something an app can work around.
- **OpenCode TUI turns a dropped PNG path into an image attachment.** Per OpenCode docs: *"You can also drag a supported image into a terminal that exposes the dropped file path to the TUI. The TUI reads PNG, JPEG, GIF, and WebP as image attachments."* So minshot drags a **temp PNG file** (native `CF_HDROP` `DoDragDrop`, hand-rolled `IDataObject` in `src-tauri/src/win/drag.rs`). Dropping it on Windows Terminal inserts the quoted path, and OpenCode attaches it as an image.
- **Clipboard paste is the pixel path and the primary workflow.** minshot auto-copies every capture as PNG/DIB to the Windows clipboard (`arboard`). `Ctrl+V` in OpenCode TUI attaches it as an image (OpenCode reads the clipboard itself; Windows Terminal forwards the paste keypress to the app).

Honest summary: **drag = file path that OpenCode upgrades to an image; paste = actual image bytes.** Both end with the model seeing the screenshot.

## Verification status (tested on Windows 11, single 1920×1080 display)

| Behavior | Status | Evidence |
|---|---|---|
| `Ctrl+Shift+4` shows fullscreen overlay, main hides / `Esc` cancels and restores main, no stray files | ✅ verified | Drove the running app via shortcut + key events; overlay fullscreen, main hidden during selection, main back after `Esc`, temp dir untouched |
| Pixel-accurate region capture | ✅ verified | 400×300 drag → 400×300 PNG, valid PNG magic; real-screen unit test passes |
| Single window becomes the shot view, sized to fit | ✅ verified | Main window reappears at captured aspect after every capture; exact-fit resize confirmed numerically |
| Main-window ✂ New capture button (home + shot views) | ✅ verified | Real clicks on both buttons opened the overlay |
| New capture replaces old (no history) | ✅ verified | Temp dir holds exactly 1 PNG after consecutive captures; old file deleted |
| Clipboard auto-copy (`Ctrl+V` source) | ✅ verified | `Clipboard.ContainsImage()` true after capture; clipboard round-trip unit test passes |
| Multi-monitor virtual-screen overlay | ⚠️ implemented, single-monitor verified | Overlay spans `GetSystemMetrics` virtual screen; capture clamps to the monitor holding the selection; only one display available to test |
| Native file drag (CF_HDROP `DoDragDrop`) | ⚠️ implemented, initiation + data verified; drop arrival unverified here | `DoDragDrop` runs, targets probe `CF_HDROP` (`QueryGetData`/`GetData` served live), HDROP payload layout unit-tested; but synthetic mouse input does not steer OLE drop targeting in this sandbox (even native Notepad→test-target drags reach nothing), so no target-side arrival could be observed. On a real desktop the OS routes the drop normally. Fast-click guard prevents starting a drag after button release, so the app cannot wedge. |
| Drop into Windows Terminal / OpenCode TUI | ❌ not tested here | No terminal automation available; relies on OpenCode's documented dropped-path→attachment behavior. If your terminal only inserts the path as text, use `Ctrl+V` (verified path). |
| Temp cleanup on graceful exit | ✅ verified | Quit button exits the app and `%TEMP%\minshot` is removed; stale (>24h) files pruned at startup |

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
cargo test --manifest-path src-tauri/Cargo.toml   # 8 unit tests (rect math, temp names, HDROP layout, clipboard + screen smoke)
```

First `dev`/`build` compiles all Rust deps (one-time, several minutes).

## Use

- Open the **minshot window** and click **＋ New capture**, or press `Ctrl+Shift+4` anywhere (the window hides during selection). Drag a rectangle → the window returns showing your screenshot, auto-fitted. `Esc`/click cancels and brings the window back.
- **Drag the screenshot** into Windows Terminal / OpenCode → path is inserted and attached as an image. While held, it shrinks with a drop hint.
- **`Ctrl+V`** where images are accepted → pastes the screenshot (no save step; PNG, auto-copied on capture). **⧉ Copy** re-copies it if the clipboard moved on.
- UI follows the OS light/dark theme (shadcn zinc tokens, no frameworks or extra deps).
- Drag the window by its header bars. **Closing the window (×), Quit buttons, or tray Quit all quit the app** and remove temp files. Temp files live in `%TEMP%\minshot\` with unique names, replaced per capture, stale ones (>24h) pruned at startup, all removed on exit.

## Limitations (verified or by design)

- Mixed-DPI setups: mapping uses the overlay window's scale, so a selection on a differently-scaled monitor can be off by the scale ratio. (Exact mixed-DPI would need per-monitor overlays — out of scope.)
- Drag into a terminal inserts a **file path**, per terminal design; only image-aware TUIs (OpenCode, etc.) upgrade it to an attachment. Plain shells will just show the path text.
- Clipboard holds one image; copying anything else replaces it.
- Single global shortcut, no configuration UI. If `Ctrl+Shift+4` collides with another app, use the tray menu.
