import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const card = document.getElementById("card") as HTMLDivElement;
const img = document.getElementById("shot") as HTMLImageElement;
const meta = document.getElementById("meta") as HTMLSpanElement;

listen<{ base64: string; width: number; height: number }>("capture-ready", (e) => {
  img.src = `data:image/png;base64,${e.payload.base64}`;
  meta.textContent = `minshot — ${e.payload.width}×${e.payload.height} — drag into terminal or Ctrl+V`;
});

// Size the window to exactly fit the rendered card (no dead space at the
// bottom): measure after the image decodes, then ask Rust to fit the window.
img.addEventListener("load", () => {
  requestAnimationFrame(() => {
    if (img.naturalWidth === 0) return;
    void invoke("fit_preview", {
      width: card.offsetWidth,
      height: card.offsetHeight,
    }).catch((err) => console.error("fit preview failed", err));
  });
});

document.getElementById("close")?.addEventListener("click", () => invoke("hide_preview"));

window.addEventListener("keydown", async (e) => {
  if (e.key === "Escape") await invoke("hide_preview");
});

// Dragging the image starts a native OS file-drag (CF_HDROP) of the temp PNG.
// Terminals receive it as a file path, which OpenCode attaches as an image.
// The `.dragging` class gives press-and-hold feedback until the drop finishes.
let dragArmed = true;
async function beginNativeDrag(e: Event) {
  e.preventDefault();
  if (!dragArmed) return;
  dragArmed = false;
  card.classList.add("dragging");
  try {
    await invoke("start_native_drag");
  } catch (err) {
    console.error("native drag failed", err);
  } finally {
    card.classList.remove("dragging");
    dragArmed = true;
  }
}
img.addEventListener("pointerdown", (e) => {
  if (e.button === 0) void beginNativeDrag(e);
});
