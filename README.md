# minshot

Minimal Windows 11 screenshot-to-terminal utility. Rust + Tauri 2, vanilla TypeScript, no frameworks.

**The whole UX is one window:**

1. The **launcher**: a compact window with one action, **New capture** — or press `Ctrl+Shift+4` anywhere. The window hides itself while you select, so it never covers the target area. Below the button: where the screenshot is saved (the temp folder by default, or a folder you pick — the choice is remembered).
2. Drag a rectangle on the fullscreen overlay (`Esc`/click cancels and brings the window back) → the same window reappears as the **shot view**: the capture drawn straight onto the app surface, with a compact toolbar (dimensions, `Copy`, `New capture`).
3. **Drag the screenshot** into a terminal (it shrinks and shows a "Release to drop" hint while held), or press `Ctrl+V` — the image is already on the clipboard. A new capture replaces the shot (no history in the temp folder; a saved folder keeps every screenshot).

The shot window takes the shape of the capture: a portrait shot gets a portrait window, a wide shot a wide one, and a big capture is scaled down to the work area of the monitor it came from — never cropped, never stretched. Resizing the window by hand keeps working; the image follows.

Closing the window quits the app (the tray has Open / Capture / Quit).

## Terminal image input: what actually works

Researched against Windows Terminal, conhost, and OpenCode docs/behavior before implementing:

- **Terminals do not accept bitmap drops.** Dragging *pixels* (`CF_DIB`/`CF_BITMAP`) into Windows Terminal is impossible — a file drop lands as **text: the file path**. This is a terminal limitation, not something an app can work around.
- **OpenCode TUI turns a dropped PNG path into an image attachment.** Per OpenCode docs: *"You can also drag a supported image into a terminal that exposes the dropped file path to the TUI. The TUI reads PNG, JPEG, GIF, and WebP as image attachments."* So minshot drags a **PNG file** (native `CF_HDROP` `DoDragDrop`, hand-rolled `IDataObject` in `src-tauri/src/win/drag.rs`). Dropping it on Windows Terminal inserts the quoted path, and OpenCode attaches it as an image.
- **Clipboard paste is the pixel path and the primary workflow.** minshot auto-copies every capture as PNG/DIB to the Windows clipboard. `Ctrl+V` in OpenCode TUI attaches it as an image (OpenCode reads the clipboard itself; Windows Terminal forwards the paste keypress to the app).

Honest summary: **drag = file path that OpenCode upgrades to an image; paste = actual image bytes.** Both end with the model seeing the screenshot.

## Verification status (tested on Windows 11, single 1920×1080 display)

| Behavior | Status | Evidence |
|---|---|---|
| `Ctrl+Shift+4` shows fullscreen overlay, main hides / `Esc` cancels and restores main, no stray files | ✅ verified | Drove the running app via shortcut + key events; overlay fullscreen, main hidden during selection, main back after `Esc`, temp dir untouched |
| `Esc` **during** a drag cancels it (no capture) | ✅ verified | Drag → `Esc` → release: no PNG written, main window restored, overlay hidden |
| Pixel-accurate region capture | ✅ verified | 400×300 drag → 400×300 PNG, valid PNG magic; real-screen unit test passes |
| Single window becomes the shot view, sized to the capture | ✅ verified | The image is drawn 1:1 in device pixels and the window takes its shape: a 900×100 shot → a 940×203 window, a 400×800 shot → 440×903, a 120×70 shot → 240×173 (the toolbar's minimum), a 1919×1038 shot → scaled to fit the work area (1769×1038) |
| The window stays inside the workspace | ✅ verified | The shot window is centred in the work area of the monitor the capture came from: a 540 px window at x=690 on a 1920 px work area (exactly centred), never under the taskbar |
| Manual resize keeps working | ✅ verified | A 500×500 shot in a window resized to 900×400 → the image scales down to 297×297 (aspect kept), centred, the toolbar intact; the app does not resize the window back |
| The shot view has no nested window | ✅ verified | The capture is drawn straight onto the app surface — no card, no frame, no second title bar around the image |
| The launcher shows one action and the real shortcut | ✅ verified | The shortcut is read back from the app (`shortcut` command), so the hint cannot drift from the binding |
| The shortcut-collision warning appears | ✅ verified | A second instance cannot register `Ctrl+Shift+4`; its launcher shows the warning. (Fixed: the shortcut is now registered before the event loop starts — the frontend used to ask before the answer existed, so the warning never appeared) |
| A failed capture says so | ✅ verified | The capture command reports a short message in the launcher ("Capture failed. Try again."); the raw error stays in the log |
| No console errors | ✅ verified | The app's CSP was missing `connect-src`, which blocked the IPC and filled the console with CSP violations; fixed, the console is clean |
| The footer does not grow across captures | ✅ verified | Three captures in a row → one `✓ Copied`, not three |
| Main-window New capture button (launcher + shot view) | ✅ verified | Real clicks on both buttons opened the overlay |
| New capture replaces old (no history) | ✅ verified | Temp dir holds exactly 1 PNG after consecutive captures; old file deleted (unit test + live run) |
| A chosen save folder keeps every screenshot | ✅ verified | With a folder picked, two captures left two files (`minshot-2026-10-20-03-57-33.png`, `…-03-58-11.png`); the first was not deleted. Without a folder the temp file is still replaced (unit test + live run) |
| The save folder is remembered | ✅ verified | The choice is written to `%APPDATA%\com.minshot.app\save-dir.txt` and read back at startup (unit test + live run: the setting file alone was enough to change where the file landed) |
| The folder picker opens | ✅ verified | The Windows folder dialog ("Choose where minshot saves screenshots") appears; `Esc` cancels and nothing changes (hardware test `the_picker_opens_a_dialog` closes it by itself) |
| The shot is loaded from the chosen folder | ✅ verified | The shot view draws the PNG through the asset protocol; the folder is added to the protocol scope when it is chosen and at startup |
| Clipboard auto-copy (`Ctrl+V` source) | ✅ verified | The clipboard image is pixel-identical to the PNG (same top-left, bottom-right and band means, checked with an independent reader). The DIB is written by hand now (no `arboard`) and round-trips through a hardware test that reads the clipboard back |
| The window shows up already in the right shape | ✅ verified | The window is sized before it is shown: the shot view never appears at the launcher's size first. A capture of 295×45 → a 335×149 window, in one step |
| A minimized window still gets the shot | ✅ verified | The window was minimized, then a capture was made: the window came back, un-minimized and in the shot's shape. (Fixed: a hidden window gets no animation frames, so the sizing used to stall on a minimized window) |
| The dim never lands in the shot | ✅ verified | The capture waits for the compositor instead of sleeping 180 ms; the captured pixels reach 255 (no dim) — checked on a real capture |
| The release build is small and quick | ✅ verified | The exe is 4.8 MB (was 9.1 MB): LTO, one codegen unit, `panic = "abort"`, symbols stripped. The PNG encoder was measured at 24 ms for a 1080p screenshot and 114 ms for 4K — the wait for the compositor, not the encode, is what a capture spends time on |
| The app stays responsive while capturing | ✅ verified | The capture runs on a runtime thread, not the UI thread (the command is `async`); a 4K capture no longer freezes the tray and the shortcut. Copying to the clipboard runs off the UI thread too |
| Multi-monitor virtual-screen overlay | ⚠️ implemented, single-monitor verified | Overlay spans `GetSystemMetrics` virtual screen; capture clamps to the monitor holding the selection; only one display available to test |
| Native file drag (CF_HDROP `DoDragDrop`) | ⚠️ implemented, initiation + data verified; drop arrival unverified here | `DoDragDrop` runs, targets probe `CF_HDROP` (`QueryGetData`/`GetData` served live), HDROP payload layout unit-tested; but synthetic mouse input does not steer OLE drop targeting in this sandbox (even native Notepad→test-target drags reach nothing), so no target-side arrival could be observed. On a real desktop the OS routes the drop normally. Fast-click guard prevents starting a drag after button release, so the app cannot wedge. |
| Drop into Windows Terminal / OpenCode TUI | ❌ not tested here | No terminal automation available; relies on OpenCode's documented dropped-path→attachment behavior. If your terminal only inserts the path as text, use `Ctrl+V` (verified path). |
| Temp cleanup on graceful exit | ✅ verified | Quit button exits the app and `%TEMP%\minshot` is removed; stale (>24h) files pruned at startup. Files in a folder the user chose are never touched |

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
npm test               # frontend tests (node --test, no dependencies)
cargo test --manifest-path src-tauri/Cargo.toml   # 15 unit tests (rect math, temp names, saved names, HDROP layout, capture state, settings)
cargo test --manifest-path src-tauri/Cargo.toml -- --include-ignored   # + the 4 hardware tests (screen, clipboard, off-thread capture, folder picker)
```

The frontend tests run on plain Node (`node --test`, no test framework, no
dependencies). The 3 hardware tests need a desktop session and are skipped
by default so CI stays green — run them with `--include-ignored`.

First `dev`/`build` compiles all Rust deps (one-time, several minutes).

## Use

- Open the **minshot window** and click **New capture**, or press `Ctrl+Shift+4` anywhere (the window hides during selection). Drag a rectangle → the window returns showing your screenshot, in a window shaped like the shot. `Esc`/click cancels and brings the window back.
- **Drag the screenshot** into Windows Terminal / OpenCode → path is inserted and attached as an image. While held, it shrinks with a drop hint.
- **`Ctrl+V`** where images are accepted → pastes the screenshot (no save step; PNG, auto-copied on capture). **Copy** re-copies it if the clipboard moved on; the dot in the toolbar says whether the capture is on the clipboard.
- UI follows the OS light/dark theme (zinc tokens, one accent, no frameworks or extra deps).
- Drag the window by its title bar or the shot toolbar. **Closing the window (×) or tray Quit quits the app** and removes temp files.
- **Save to**: by default the screenshot goes to `%TEMP%\minshot\` with a unique name; the next capture replaces it and quitting removes it. Click **Change** on the launcher to pick a folder instead: the file is then *saved* there (`minshot-2026-10-20-15-34-22.png`), every capture keeps its own file, and nothing is deleted. The choice is remembered in the app's config folder. Click **×** to go back to the temp folder.
- If `Ctrl+Shift+4` is already taken by another app, the window says so and you capture from the tray menu instead.

## Limitations (verified or by design)

- Mixed-DPI setups: mapping uses the overlay window's scale, so a selection on a differently-scaled monitor can be off by the scale ratio. (Exact mixed-DPI would need per-monitor overlays — out of scope.)
- Drag into a terminal inserts a **file path**, per terminal design; only image-aware TUIs (OpenCode, etc.) upgrade it to an attachment. Plain shells will just show the path text.
- Clipboard holds one image; copying anything else replaces it.
- Single global shortcut, no configuration UI. If `Ctrl+Shift+4` collides with another app, use the tray menu.
- The shot is never enlarged: a capture is shown one device pixel per captured pixel, so zooming the window in only adds space around it.
- A full-screen capture takes a moment in a debug build (the PNG encode is unoptimised); the release build is the fast one.
