import { invoke } from "@tauri-apps/api/core";
import { Selection, type Rect } from "./selection.ts";

const sel = document.getElementById("sel") as HTMLDivElement;
const selection = new Selection();

declare global {
  interface Window {
    /// Called by the app when the overlay comes up, to clear the
    /// selection of the last shot.
    __minshotReset?: () => void;
  }
}

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
  // The edge is only drawn around a real selection. An empty hole would
  // leave a mark in the corner of the screen.
  sel.classList.toggle("picked", r.w > 0 && r.h > 0);
}

/// The screen is dimmed as soon as the overlay appears, the way the
/// Windows snipping tool shows it. The hole is empty until the drag
/// starts, so the whole screen is dim.
const noSelection: Rect = { x: 0, y: 0, w: 0, h: 0 };
showHole(noSelection);

// The overlay is shown again after every capture. Its webview keeps the
// last selection, so it has to be cleared: otherwise the rectangle of the
// previous shot is still on the screen when the overlay comes up. The app
// calls this when it shows the overlay; the focus is the backup for the
// case where the page is not ready yet.
function reset() {
  selection.cancel();
  showHole(noSelection);
}
window.__minshotReset = reset;
window.addEventListener("focus", reset);

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
  if (!r) {
    await cancelCapture();
    return;
  }
  // The hole has to be exactly the region that is captured: the mouseup
  // can land where the last mousemove did not, and then the picture
  // would show the dim and the edge of the hole.
  showHole(r);
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
