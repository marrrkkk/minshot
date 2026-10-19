import { invoke } from "@tauri-apps/api/core";
import { Selection, type Rect } from "./selection.ts";

const sel = document.getElementById("sel") as HTMLDivElement;
const selection = new Selection();

function showRect(r: Rect) {
  sel.style.display = "block";
  sel.style.left = `${r.x}px`;
  sel.style.top = `${r.y}px`;
  sel.style.width = `${r.w}px`;
  sel.style.height = `${r.h}px`;
}

window.addEventListener("mousedown", (e) => {
  if (e.button !== 0) return;
  selection.begin(e.clientX, e.clientY);
  const r = selection.update(e.clientX, e.clientY);
  if (r) showRect(r);
});

window.addEventListener("mousemove", (e) => {
  const r = selection.update(e.clientX, e.clientY);
  if (r) showRect(r);
});

window.addEventListener("mouseup", async (e) => {
  const r = selection.end(e.clientX, e.clientY);
  sel.style.display = "none";
  if (!r) {
    await cancelCapture();
    return;
  }
  try {
    await invoke("capture_region", {
      x: r.x,
      y: r.y,
      w: r.w,
      h: r.h,
      scale: window.devicePixelRatio || 1,
    });
  } catch (err) {
    console.error("capture failed", err);
    await cancelCapture();
  }
});

/// Dismiss the overlay and bring the main window back (it hides during
/// selection so it never covers the target area).
async function cancelCapture() {
  // Drop the drag: without this the mouseup that follows an Escape would
  // still capture the region the user just cancelled.
  selection.cancel();
  await invoke("hide_overlay");
  await invoke("show_main");
}

window.addEventListener("keydown", async (e) => {
  if (e.key === "Escape") await cancelCapture();
});
