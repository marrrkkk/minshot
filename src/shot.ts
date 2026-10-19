/// The shot view: how a capture is drawn and sized in the main window.
///
/// The window is sized to the capture, so the image is never bigger than the
/// space the desktop gives it: a 4K capture is scaled down to fit the work
/// area, a small one is left at its own size.

export interface ShotPayload {
  /// Image URL of the captured PNG (asset protocol).
  src: string;
  /// The capture's real pixel size.
  width: number;
  height: number;
  /// Whether the PNG made it onto the clipboard.
  clipboard: boolean;
  /// Physical pixels per CSS pixel on the monitor the capture came from.
  scale: number;
  /// The logical space the window may use: the work area minus its frame.
  availWidth: number;
  availHeight: number;
}

/// The elements renderShot touches. Structural, so the DOM nodes fit.
export interface ShotEls {
  img: { src: string; style: { width: string; height: string } };
  dims: { textContent: string; title: string };
  dot: {
    title: string;
    classList: { add(c: string): void; remove(c: string): void };
  };
  home: { classList: { add(c: string): void; remove(c: string): void } };
  shot: { classList: { add(c: string): void; remove(c: string): void } };
}

/// The size the image is drawn at, and the window size that holds it.
export interface ShotPlan {
  /// The image's display size in CSS px.
  width: number;
  height: number;
  /// The window's content size in CSS px.
  windowWidth: number;
  windowHeight: number;
}

/// Scale a capture to fit the box. Never enlarges: a screenshot has no
/// detail to show at 2x.
export function fitShot(
  naturalW: number,
  naturalH: number,
  maxW: number,
  maxH: number,
): { width: number; height: number } {
  const w = Math.max(1, maxW);
  const h = Math.max(1, maxH);
  const scale = Math.min(1, w / naturalW, h / naturalH);
  return {
    width: Math.round(naturalW * scale),
    height: Math.round(naturalH * scale),
  };
}

/// Plan the shot view for a capture.
///
/// The image is shown one device pixel per captured pixel (a 1920px region
/// on a 150% screen is 1280 CSS px wide, exactly as wide as it was on the
/// screen), scaled down only when the workspace cannot hold it. `chrome` is
/// the toolbar, `pad` the breathing room around the image.
export function planShot(
  physW: number,
  physH: number,
  scale: number,
  availW: number,
  availH: number,
  chrome: number,
  pad: number,
): ShotPlan {
  const s = scale > 0 ? scale : 1;
  const box = fitShot(
    physW / s,
    physH / s,
    availW - pad * 2,
    availH - chrome - pad * 2,
  );
  return {
    width: box.width,
    height: box.height,
    windowWidth: box.width + pad * 2,
    windowHeight: box.height + chrome + pad * 2,
  };
}

/// Show a capture: swap the launcher out, label it, note the clipboard state.
export function renderShot(els: ShotEls, payload: ShotPayload): void {
  els.img.src = payload.src;
  els.dims.textContent = `${payload.width} × ${payload.height}`;
  els.dims.title = `PNG · ${payload.width} × ${payload.height}`;
  els.dot.title = payload.clipboard
    ? "On the clipboard"
    : "Clipboard copy failed — press Copy";
  els.dot.classList.remove("warn");
  if (!payload.clipboard) els.dot.classList.add("warn");
  els.home.classList.remove("active");
  els.shot.classList.add("active");
}
