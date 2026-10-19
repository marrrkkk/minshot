import { invoke } from "@tauri-apps/api/core";
import { Selection, type Rect } from "./selection.ts";

const sel = document.getElementById("sel") as HTMLDivElement;
const selection = new Selection();

/// The dim is a hole in the screen: the selection sits in it and the
/// shadow of the selection darkens everything around it. Nothing is
/// painted inside the hole, so the region that is about to be captured
/// is the screen itself.
function showHole(r: Rect) {
  sel.style.display = "block";
  sel.style.left = `${r.x}px`;
  sel.style.top = `${r.y}px`;
  sel.style.width = `${r.w}px`;
  sel.style.height = `${r.h}px`;
}

// The screen is dimmed as soon as the overlay appears, the way the
// Windows snipping tool shows it. The hole is empty until the drag
// starts, so the whole screen is dim.
showHole({ x: 0, y: 0, w: 0, h: 0 });

window.addEventListener("mousedown", (e) => {
  if (e.button !== 0) return;
  selection.begin(e.clientX, e.clientY);
  const r = selection.update(e.clientX, e.clientY);
  if (r) showHole(r);
});

window.addEventListener("mousemove", (e) => {
  const r = selection.update(e.clientX, e.clientY);
  if (r) showHole(r);
});

window.addEventListener("mouseup", async (e) => {
  const r = selection.end(e.clientX, e.clientY);
  // The hole stays where it is: the dim goes away with the overlay, and
  // the hole is what the camera sees.
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
