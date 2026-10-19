import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const homeView = document.getElementById("homeView") as HTMLDivElement;
const shotView = document.getElementById("shotView") as HTMLDivElement;
const shotCard = document.getElementById("shotCard") as HTMLDivElement;
const img = document.getElementById("shot") as HTMLImageElement;
const dims = document.getElementById("dims") as HTMLSpanElement;
const foot = document.getElementById("shotFoot") as HTMLDivElement;
const homeInner = document.getElementById("homeInner") as HTMLDivElement;

// Size the window to exactly fit the home content on startup, so the footer
// (Quit) can never end up clipped below the fold. Same mechanism as shots.
function fitHome() {
  requestAnimationFrame(() => {
    void invoke("fit_main", {
      width: document.documentElement.clientWidth,
      height: homeInner.offsetHeight + 32,
    }).catch((err) => console.error("fit window failed", err));
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

// A capture turns the main window into the screenshot view.
listen<{ path: string; width: number; height: number; clipboard: boolean }>(
  "capture-ready",
  (e) => {
    img.src = convertFileSrc(e.payload.path);
    dims.textContent = `${e.payload.width}×${e.payload.height}`;
    foot.textContent += e.payload.clipboard ? " ✓ Copied" : " Copy failed — press ⧉ Copy.";
    homeView.classList.add("hidden");
    shotView.classList.remove("hidden");
  },
);

// Size the window to exactly fit the rendered shot view (no dead space):
// measure after the image decodes, then ask Rust to fit the window.
img.addEventListener("load", () => {
  requestAnimationFrame(() => {
    if (img.naturalWidth === 0) return;
    void invoke("fit_main", {
      width: document.documentElement.clientWidth,
      height: shotView.offsetHeight,
    }).catch((err) => console.error("fit window failed", err));
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
