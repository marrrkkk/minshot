import { invoke } from "@tauri-apps/api/core";
window.addEventListener("DOMContentLoaded", () => {
  document.getElementById("capture-btn")?.addEventListener("click", () => invoke("show_overlay"));
});
