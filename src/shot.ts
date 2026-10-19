/// The shot view: how a capture is drawn in the main window.
///
/// The window is resized to the shot, so the image may not be bigger than
/// this box — a 4K capture has to be scaled down to fit, a small one is
/// left at its own size.

export const MAX_SHOT_W = 840;
export const MAX_SHOT_H = 600;

export interface ShotPayload {
  /// Image URL of the captured PNG (asset protocol).
  src: string;
  /// The capture's real pixel size.
  width: number;
  height: number;
  /// Whether the PNG made it onto the clipboard.
  clipboard: boolean;
}

/// The elements renderShot touches. Structural, so the DOM nodes fit.
export interface ShotEls {
  img: { src: string; style: { width: string; height: string } };
  dims: { textContent: string };
  foot: { textContent: string };
  homeView: { classList: { add(c: string): void; remove(c: string): void } };
  shotView: { classList: { add(c: string): void; remove(c: string): void } };
}

const HINT =
  "Drag the image into Windows Terminal / OpenCode, or paste with Ctrl+V.";

export function shotFooter(clipboardOk: boolean): string {
  return clipboardOk
    ? `${HINT} ✓ Copied`
    : `${HINT} Copy failed — press ⧉ Copy.`;
}

/// Scale a capture to fit the box. Never enlarges: a screenshot has no
/// detail to show at 2x.
export function fitShot(
  naturalW: number,
  naturalH: number,
  maxW: number = MAX_SHOT_W,
  maxH: number = MAX_SHOT_H,
): { width: number; height: number } {
  const scale = Math.min(1, maxW / naturalW, maxH / naturalH);
  return {
    width: Math.round(naturalW * scale),
    height: Math.round(naturalH * scale),
  };
}

/// Show a capture: size the image, label it, swap the home view out.
export function renderShot(els: ShotEls, payload: ShotPayload): void {
  const box = fitShot(payload.width, payload.height);
  els.img.src = payload.src;
  els.img.style.width = `${box.width}px`;
  els.img.style.height = `${box.height}px`;
  els.dims.textContent = `${payload.width}×${payload.height}`;
  els.foot.textContent = shotFooter(payload.clipboard);
  els.homeView.classList.add("hidden");
  els.shotView.classList.remove("hidden");
}
