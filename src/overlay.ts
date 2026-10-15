import { invoke } from "@tauri-apps/api/core";

const sel = document.getElementById("sel") as HTMLDivElement;
let startX = 0, startY = 0, dragging = false;

function showRect(x0: number, y0: number, x1: number, y1: number) {
  const x = Math.min(x0, x1), y = Math.min(y0, y1);
  const w = Math.abs(x1 - x0), h = Math.abs(y1 - y0);
  sel.style.display = "block";
  sel.style.left = `${x}px`;
  sel.style.top = `${y}px`;
  sel.style.width = `${w}px`;
  sel.style.height = `${h}px`;
}

window.addEventListener("mousedown", (e) => {
  if (e.button !== 0) return;
  dragging = true;
  startX = e.clientX; startY = e.clientY;
  showRect(startX, startY, startX, startY);
});

window.addEventListener("mousemove", (e) => {
  if (!dragging) return;
  showRect(startX, startY, e.clientX, e.clientY);
});

window.addEventListener("mouseup", async (e) => {
  if (!dragging) return;
  dragging = false;
  sel.style.display = "none";
  const x = Math.min(startX, e.clientX);
  const y = Math.min(startY, e.clientY);
  const w = Math.abs(e.clientX - startX);
  const h = Math.abs(e.clientY - startY);
  if (w < 4 || h < 4) {
    await cancelCapture();
    return;
  }
  try {
    await invoke("capture_region", {
      x, y, w, h,
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
  await invoke("hide_overlay");
  await invoke("show_main");
}

window.addEventListener("keydown", async (e) => {
  if (e.key === "Escape") await cancelCapture();
});
