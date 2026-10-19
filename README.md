# minshot

Minimal Windows 11 screenshot-to-terminal utility. Rust + Tauri 2, vanilla TypeScript, no frameworks.

Capture a region of the screen, and it is on your clipboard and in your hand —
drag it into a terminal, or paste it. No history, no save dialog, no cloud.

## What it does

1. **Launch.** A small window with one action, **New capture** — or press `Ctrl+Shift+4` anywhere. The window hides itself while you select, so it never covers the target area.
2. **Select.** Drag a rectangle on the fullscreen overlay. `Esc` or a click cancels and brings the window back.
3. **Use it.** The same window reappears as the **shot view**: the capture drawn onto the app surface, with a compact toolbar (dimensions, `Copy`, `New capture`). The window takes the shape of the shot — a portrait shot gets a portrait window, a wide shot a wide one, a big capture is scaled down to the work area of the monitor it came from. Never cropped, never stretched.

The shot can be **dragged into a terminal** (it shrinks and shows a drop hint while held) or pasted with `Ctrl+V` — the image is already on the clipboard.

Closing the window quits the app. The tray holds Open / Capture / Quit.

## Install

Grab the installer from the [releases page](https://github.com/marrrkkk/minshot/releases) and run it. It installs for the current user, no admin rights, and WebView2 ships with Windows 11.

## Use

- **Capture:** click **New capture**, or press `Ctrl+Shift+4` anywhere.
- **Paste:** `Ctrl+V` where images are accepted (OpenCode TUI, Paint, an image box in a chat app). The PNG is auto-copied on capture; **Copy** re-copies it if the clipboard moved on.
- **Drag:** drag the screenshot into Windows Terminal / OpenCode. See below for what that actually does.
- **Save to:** by default the screenshot goes to `%TEMP%\minshot\` and the next capture replaces it; quitting removes it. Click **Change** on the launcher to pick a folder instead — every capture then keeps its own file (`minshot-2026-10-20-15-34-22.png`) and nothing is ever deleted. The choice is remembered.
- If `Ctrl+Shift+4` is already taken by another app, the window says so and you capture from the tray menu instead.
- The UI follows the OS light/dark theme.

## What the terminal integration really does

Terminals do not accept bitmap drops. Dragging *pixels* (`CF_DIB`/`CF_BITMAP`) into Windows Terminal is impossible — a file drop lands as **text: the file path**. This is a terminal limitation, not something an app can work around.

- **Drag** hands over a PNG file (native `CF_HDROP` `DoDragDrop`, hand-rolled `IDataObject` in `src-tauri/src/win/drag.rs`). Dropping it on Windows Terminal inserts the quoted path. OpenCode's TUI reads a dropped PNG path as an image attachment, so it ends up as an image there.
- **Paste** is the pixel path: minshot puts the actual image bytes on the clipboard, and `Ctrl+V` pastes the image.

In short: **drag = a file path that OpenCode upgrades to an image; paste = the image itself.** Both end with the model seeing your screenshot. In a plain shell, a drag only inserts the path text.

## Build from source

You need Windows 11, [Node 22.18+](https://nodejs.org), [Rust stable (MSVC)](https://rustup.rs) and the Visual Studio 2022 Build Tools with the **VCTools** workload:

```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools -e
# then, elevated:
setup.exe modify --installPath "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools" `
  --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended `
  --add Microsoft.VisualStudio.Component.Windows11SDK.26100 --quiet --norestart
```

```powershell
npm install
npm run tauri dev      # run with hot reload
npm run tauri build    # → src-tauri/target/release/bundle/nsis/
npm test               # frontend tests (node --test, no dependencies)
cargo test --manifest-path src-tauri/Cargo.toml   # 15 Rust unit tests
```

The first build compiles all Rust dependencies (one-time, several minutes).

The Rust tests that need a real desktop (screen capture, the clipboard, the folder picker) are marked `#[ignore]` and skipped in CI. Run them with:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -- --include-ignored
```

`scripts/drive.ps1` drives the running app from the command line (trigger the shortcut, drag a region, screenshot the window) — the manual test harness for the behaviours CI cannot check.

## Known limits

- Mixed-DPI setups: the selection maps through the overlay window's scale factor, so a selection on a differently-scaled monitor can be off by the scale ratio.
- The clipboard holds one image; copying anything else replaces it.
- One global shortcut, no configuration UI.
- The shot is never enlarged: a capture is shown one device pixel per captured pixel.
- The overlay spans the whole virtual screen and clamps the capture to the monitor holding the selection, but only a single-display setup has been verified.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE) © 2026 marrrkkk
