import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

const status = document.getElementById("status") as HTMLDivElement;

// Closing the window hides it to the tray instead of quitting (tray: Open minshot).
getCurrentWindow().onCloseRequested(async (e) => {
  e.preventDefault();
  await getCurrentWindow().hide();
});

document.getElementById("newBtn")?.addEventListener("click", () => invoke("show_overlay"));
document.getElementById("quit")?.addEventListener("click", () => invoke("quit_app"));

// Show the latest capture result, like Snipping Tool's status line.
listen<{ width: number; height: number; clipboard: boolean }>("capture-ready", (e) => {
  const clip = e.payload.clipboard ? "copied to clipboard ✓" : "clipboard copy failed";
  status.textContent = `${e.payload.width}×${e.payload.height} captured — ${clip}`;
});
