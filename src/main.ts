import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { renderShot, type ShotEls } from "./shot.ts";

const homeView = document.getElementById("homeView") as HTMLDivElement;
const shotView = document.getElementById("shotView") as HTMLDivElement;
const shotCard = document.getElementById("shotCard") as HTMLDivElement;
const img = document.getElementById("shot") as HTMLImageElement;
const dims = document.getElementById("dims") as HTMLSpanElement;
const foot = document.getElementById("shotFoot") as HTMLDivElement;
const homeInner = document.getElementById("homeInner") as HTMLDivElement;

const els: ShotEls = { img, dims, foot, homeView, shotView };

// The Windows title bar is not part of the webview, so the window has to
// be that much taller than the content it shows.
const TITLE_BAR = 32;

function fitWindow(width: number, height: number) {
  void invoke("fit_main", { width, height }).catch((err) =>
    console.error("fit window failed", err),
  );
}

// Size the window to exactly fit the home content on startup, so the footer
// (Quit) can never end up clipped below the fold. Same mechanism as shots.
function fitHome() {
  requestAnimationFrame(() => {
    fitWindow(
      document.documentElement.clientWidth,
      homeInner.offsetHeight + TITLE_BAR,
    );
  });
}
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", fitHome);
} else {
  fitHome();
}

// Closing the main window quits the app (Snipping-Tool-style: no hidden
// background mode to get confused by; Quit buttons and the tray do the same).
// The Rust side exits on the window-close event, so temp files are cleaned.

document.querySelectorAll(".newBtn").forEach((b) =>
  b.addEventListener("click", () => invoke("show_overlay")),
);
document.querySelectorAll(".quit").forEach((b) =>
  b.addEventListener("click", () => invoke("quit_app")),
);
document.querySelectorAll(".copyBtn").forEach((b) =>
  b.addEventListener("click", async () => {
    const btn = b as HTMLButtonElement;
    try {
      await invoke("copy_to_clipboard");
      const prev = btn.textContent;
      btn.textContent = "✓ Copied";
      setTimeout(() => (btn.textContent = prev), 1200);
    } catch (err) {
      console.error("copy failed", err);
    }
  }),
);

// The global shortcut may be taken by another app. Say so instead of
// leaving the user wondering why nothing happens.
void invoke<string | null>("shortcut_error").then((err) => {
  if (!err) return;
  const warn = document.getElementById("warn") as HTMLDivElement;
  warn.textContent =
    "Ctrl+Shift+4 is taken by another app — capture from the tray menu instead.";
  warn.classList.remove("hidden");
});

// A capture turns the main window into the screenshot view.
listen<{ path: string; width: number; height: number; clipboard: boolean }>(
  "capture-ready",
  (e) => {
    renderShot(els, {
      src: convertFileSrc(e.payload.path),
      width: e.payload.width,
      height: e.payload.height,
      clipboard: e.payload.clipboard,
    });
  },
);

// Size the window to exactly fit the rendered shot view (no dead space):
// measure after the image decodes, then ask Rust to fit the window.
img.addEventListener("load", () => {
  requestAnimationFrame(() => {
    if (img.naturalWidth === 0) return;
    const pad = getComputedStyle(shotView);
    const chrome = parseFloat(pad.paddingLeft) + parseFloat(pad.paddingRight);
    fitWindow(shotCard.offsetWidth + chrome, shotView.offsetHeight + TITLE_BAR);
  });
});

// Dragging the image starts a native OS file-drag (CF_HDROP) of the temp PNG.
// Terminals receive it as a file path, which OpenCode attaches as an image.
// The `.dragging` class gives press-and-hold feedback until the drop finishes.
let dragArmed = true;
async function beginNativeDrag(e: Event) {
  e.preventDefault();
  if (!dragArmed) return;
  dragArmed = false;
  shotCard.classList.add("dragging");
  try {
    await invoke("start_native_drag");
  } catch (err) {
    console.error("native drag failed", err);
  } finally {
    shotCard.classList.remove("dragging");
    dragArmed = true;
  }
}
img.addEventListener("pointerdown", (e) => {
  if (e.button === 0) void beginNativeDrag(e);
});
