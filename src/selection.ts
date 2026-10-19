/// The region the user drags out on the overlay.
///
/// Kept apart from the DOM so the cancel/click rules can be tested: the
/// overlay is a fullscreen transparent window and a mis-handled mouseup
/// there is invisible until it costs the user a screenshot.

export interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

/// Below this many CSS pixels the drag is a click, not a selection.
/// The Rust side keeps its own floor (a rect smaller than a physical
/// pixel cannot be a real region).
export const MIN_DRAG_PX = 4;

export class Selection {
  private dragging = false;
  private x0 = 0;
  private y0 = 0;

  begin(x: number, y: number): void {
    this.dragging = true;
    this.x0 = x;
    this.y0 = y;
  }

  /// The rectangle under the cursor, or null when no drag is running.
  update(x: number, y: number): Rect | null {
    if (!this.dragging) return null;
    return rect(this.x0, this.y0, x, y);
  }

  /// The finished selection, or null when the drag was too small or was
  /// cancelled.
  end(x: number, y: number): Rect | null {
    if (!this.dragging) return null;
    this.dragging = false;
    const r = rect(this.x0, this.y0, x, y);
    return r.w < MIN_DRAG_PX || r.h < MIN_DRAG_PX ? null : r;
  }

  /// Drop the drag: Escape, or anything else that aborts the capture.
  cancel(): void {
    this.dragging = false;
  }

  get active(): boolean {
    return this.dragging;
  }
}

function rect(x0: number, y0: number, x1: number, y1: number): Rect {
  return {
    x: Math.min(x0, x1),
    y: Math.min(y0, y1),
    w: Math.abs(x1 - x0),
    h: Math.abs(y1 - y0),
  };
}
