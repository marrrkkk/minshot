import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const img = document.getElementById("shot") as HTMLImageElement;
const meta = document.getElementById("meta") as HTMLSpanElement;

listen<{ base64: string; width: number; height: number }>("capture-ready", (e) => {
  img.src = `data:image/png;base64,${e.payload.base64}`;
  meta.textContent = `minshot — ${e.payload.width}×${e.payload.height} — drag into terminal or Ctrl+V`;
});

document.getElementById("close")?.addEventListener("click", () => invoke("hide_preview"));

window.addEventListener("keydown", async (e) => {
  if (e.key === "Escape") await invoke("hide_preview");
});

// Dragging the image starts a native OS file-drag (CF_HDROP) of the temp PNG.
// Terminals receive it as a file path, which OpenCode attaches as an image.
// If a target only inserts the path as text, use Ctrl+V instead (image is on the clipboard).
// Guard against re-entry while a drag is already running.
let dragArmed = true;
async function beginNativeDrag(e: Event) {
  e.preventDefault();
  if (!dragArmed) return;
  dragArmed = false;
  try {
    await invoke("start_native_drag");
  } catch (err) {
    console.error("native drag failed", err);
  } finally {
    dragArmed = true;
  }
}
img.addEventListener("pointerdown", (e) => {
  if (e.button === 0) void beginNativeDrag(e);
});
