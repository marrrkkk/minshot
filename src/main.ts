import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { fitShot, planShot, renderShot, type ShotEls } from "./shot.ts";
import { renderSaveDir, type SaveDirEls } from "./settings.ts";

const home = document.getElementById("home") as HTMLDivElement;
const shotView = document.getElementById("shot") as HTMLDivElement;
const img = document.getElementById("shotImg") as HTMLImageElement;
const dims = document.getElementById("dims") as HTMLSpanElement;
const dot = document.getElementById("clipDot") as HTMLSpanElement;
const toolbar = document.querySelector(".toolbar") as HTMLElement;
const preview = document.getElementById("preview") as HTMLDivElement;
const notice = document.getElementById("notice") as HTMLParagraphElement;
const copyBtn = document.getElementById("copyBtn") as HTMLButtonElement;
const copyLabel = document.getElementById("copyLabel") as HTMLSpanElement;
const keys = document.getElementById("keys") as HTMLSpanElement;
const saveDir = document.getElementById("saveDir") as HTMLSpanElement;
const saveDirBtn = document.getElementById("saveDirBtn") as HTMLButtonElement;
const saveDirReset = document.getElementById("saveDirReset") as HTMLButtonElement;

const els: ShotEls = { img, dims, dot, home, shot: shotView };
const saveEls: SaveDirEls = {
  dir: saveDir,
  reset: saveDirReset,
  change: saveDirBtn,
};

/// What the capture-ready event carries.
interface CapturePayload {
  path: string;
  width: number;
  height: number;
  clipboard: boolean;
  scale: number;
  availWidth: number;
  availHeight: number;
}

/// What the settings commands carry.
interface SettingsPayload {
  saveDir: string | null;
}

function fitWindow(width: number, height: number): Promise<void> {
  return invoke("fit_main", { width, height })
    .then(() => undefined)
    .catch((err) => {
      console.error("fit window failed", err);
    });
}

/// The launcher is sized to its content: the window is exactly as big as the
/// app needs, never more.
function fitHome() {
  requestAnimationFrame(() => {
    const r = home.getBoundingClientRect();
    if (r.width > 0) void fitWindow(r.width, r.height);
  });
}

function showHome() {
  shotView.classList.remove("active");
  home.classList.add("active");
}

/// A notice is only there when it has something to say.
function showNotice(text: string, kind: "warn" | "error") {
  notice.textContent = text;
  notice.className = `notice show ${kind}`;
  fitHome();
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", fitHome);
} else {
  fitHome();
}

// There is no Quit button: the native close button quits the app (the Rust
// side exits on the window-close event) and the tray has Open / Capture /
// Quit. A second quit control would only duplicate the native one.

// ---------------------------------------------------------------- capture

document.querySelectorAll<HTMLButtonElement>(".newBtn").forEach((b) => {
  b.addEventListener("click", async () => {
    // One capture at a time: a double click must not open two overlays.
    b.disabled = true;
    try {
      await invoke("show_overlay");
    } catch (err) {
      console.error("show overlay failed", err);
    } finally {
      b.disabled = false;
    }
  });
});

/// The image at its natural size (CSS px), so a later resize can refit it.
let natural = { w: 0, h: 0 };

/// The breathing room around the image. Read when a capture arrives: the
/// stylesheet decides it, the theme can change it.
let pad = 0;

function applySize(width: number, height: number) {
  img.style.width = `${width}px`;
  img.style.height = `${height}px`;
}

/// The toolbar's height, measured from the laid-out DOM.
function chromePx(): number {
  return toolbar.offsetHeight;
}

// A capture turns the launcher into the shot view.
listen<CapturePayload>("capture-ready", (e) => {
  const p = e.payload;
  renderShot(els, {
    src: convertFileSrc(p.path),
    width: p.width,
    height: p.height,
    clipboard: p.clipboard,
    scale: p.scale,
    availWidth: p.availWidth,
    availHeight: p.availHeight,
  });
  natural = { w: p.width / p.scale, h: p.height / p.scale };
  // Measure right away: the window is still hidden, and a hidden window
  // gets no animation frames — waiting for one would stall the shot.
  const plan = planShot(
    p.width,
    p.height,
    p.scale,
    p.availWidth,
    p.availHeight,
    chromePx(),
    padPx(),
  );
  applySize(plan.width, plan.height);
  // The window is sized before it shows up, so it never appears in the
  // wrong shape for a frame. No focus: the user is usually headed back
  // to the terminal to paste.
  void fitWindow(plan.windowWidth, plan.windowHeight).then(() =>
    invoke("show_main", { focus: false }).catch((err) => {
      console.error("show main failed", err);
    }),
  );
});

// A failed capture says so and offers the same action again.
listen<{ message: string }>("capture-failed", (e) => {
  showHome();
  showNotice(e.payload.message, "error");
});

// The preview follows the window: it shrinks with the window, never
// stretches past its own pixels. One paint per frame, not one per event.
let resizeFrame = 0;
window.addEventListener("resize", () => {
  if (!shotView.classList.contains("active")) return;
  if (resizeFrame) return;
  resizeFrame = requestAnimationFrame(() => {
    resizeFrame = 0;
    const box = fitShot(
      natural.w,
      natural.h,
      preview.clientWidth - pad * 2,
      preview.clientHeight - pad * 2,
    );
    applySize(box.width, box.height);
  });
});

/// The breathing room around the image, read from the stylesheet.
function padPx(): number {
  if (!pad) {
    pad = parseFloat(getComputedStyle(preview).paddingLeft) || 0;
  }
  return pad;
}

// -------------------------------------------------------------- clipboard

let copyTimer = 0;
function setCopyState(label: string, failed: boolean) {
  copyLabel.textContent = label;
  dot.classList.toggle("warn", failed);
  dot.title = failed ? "Clipboard copy failed" : "On the clipboard";
  window.clearTimeout(copyTimer);
  copyTimer = window.setTimeout(() => {
    copyLabel.textContent = "Copy";
    dot.classList.remove("warn");
    dot.title = "On the clipboard";
  }, 1400);
}

copyBtn.addEventListener("click", async () => {
  copyBtn.disabled = true;
  try {
    await invoke("copy_to_clipboard");
    setCopyState("Copied", false);
  } catch (err) {
    console.error("copy failed", err);
    setCopyState("Copy failed", true);
  } finally {
    copyBtn.disabled = false;
  }
});

// ----------------------------------------------------------------- chrome

// The global shortcut may be taken by another app. Say so instead of
// leaving the user wondering why nothing happens.
void invoke<string | null>("shortcut_error").then((err) => {
  if (!err) return;
  showNotice(
    "Ctrl+Shift+4 is taken by another app — capture from the tray menu instead.",
    "warn",
  );
});

// The shortcut on the launcher is read back from the app, so the hint can
// never drift from the real binding.
void invoke<string>("shortcut")
  .then((sc) => {
    for (const part of sc.split("+")) {
      const k = document.createElement("span");
      k.className = "kbd";
      k.textContent = part;
      keys.appendChild(k);
    }
  })
  .catch((err) => console.error("shortcut lookup failed", err));

// --------------------------------------------------------------- settings

// Where the screenshot is saved: the temp folder, or a folder the user
// picked. The choice is remembered by the app.
void invoke<SettingsPayload>("get_settings")
  .then((s) => renderSaveDir(saveEls, s.saveDir))
  .catch((err) => console.error("settings lookup failed", err));

saveDirBtn.addEventListener("click", async () => {
  saveDirBtn.disabled = true;
  try {
    // The folder picker: nothing chosen means "never mind".
    const dir = await invoke<string | null>("pick_save_dir");
    if (!dir) return;
    const s = await invoke<SettingsPayload>("set_save_dir", { dir });
    renderSaveDir(saveEls, s.saveDir);
  } catch (err) {
    console.error("save folder failed", err);
  } finally {
    saveDirBtn.disabled = false;
  }
});

saveDirReset.addEventListener("click", async () => {
  try {
    const s = await invoke<SettingsPayload>("set_save_dir", { dir: null });
    renderSaveDir(saveEls, s.saveDir);
  } catch (err) {
    console.error("save folder reset failed", err);
  }
});

// ------------------------------------------------------------------- drag

// Dragging the image starts a native OS file-drag (CF_HDROP) of the temp
// PNG. Terminals receive it as a file path, which OpenCode attaches as an
// image. The `dragging` class gives press-and-hold feedback until the drop
// finishes.
let dragArmed = true;
async function beginNativeDrag(e: Event) {
  e.preventDefault();
  if (!dragArmed) return;
  dragArmed = false;
  preview.classList.add("dragging");
  try {
    await invoke("start_native_drag");
  } catch (err) {
    console.error("native drag failed", err);
  } finally {
    preview.classList.remove("dragging");
    dragArmed = true;
  }
}
img.addEventListener("pointerdown", (e) => {
  if (e.button === 0) void beginNativeDrag(e);
});
