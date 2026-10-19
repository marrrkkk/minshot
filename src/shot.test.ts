import { test } from "node:test";
import assert from "node:assert/strict";
import { fitShot, renderShot, shotFooter } from "./shot.ts";
import type { ShotEls, ShotPayload } from "./shot.ts";

// Minimal stand-ins for the elements renderShot touches. The point is the
// real function's effect on them, not the stand-ins themselves.
function fakeEls() {
  const homeAdded: string[] = [];
  const shotRemoved: string[] = [];
  const els: ShotEls = {
    img: { src: "", style: { width: "", height: "" } },
    dims: { textContent: "" },
    foot: { textContent: "" },
    homeView: {
      classList: {
        add: (c: string) => homeAdded.push(c),
        remove: () => {},
      },
    },
    shotView: {
      classList: {
        add: () => {},
        remove: (c: string) => shotRemoved.push(c),
      },
    },
  };
  return { els, homeAdded, shotRemoved };
}

const shot: ShotPayload = {
  src: "http://asset.localhost/one.png",
  width: 1920,
  height: 1080,
  clipboard: true,
};

test("a capture wider than the box is scaled down to fit it", () => {
  assert.deepEqual(fitShot(1920, 1080, 840, 600), { width: 840, height: 473 });
});

test("a tall capture is scaled to the box height so no part is cut off", () => {
  assert.deepEqual(fitShot(380, 2000, 840, 600), { width: 114, height: 600 });
});

test("a small capture is shown at its own size, not blown up", () => {
  assert.deepEqual(fitShot(100, 50, 840, 600), { width: 100, height: 50 });
});

test("the footer says the shot is on the clipboard", () => {
  assert.match(shotFooter(true), /Copied/);
});

test("the footer says the clipboard copy failed", () => {
  assert.match(shotFooter(false), /Copy failed/);
});

test("a second capture does not make the footer longer", () => {
  const { els } = fakeEls();
  renderShot(els, shot);
  const afterFirst = els.foot.textContent;
  renderShot(els, shot);
  assert.equal(els.foot.textContent, afterFirst);
});

test("the shot view replaces the home view", () => {
  const { els, homeAdded, shotRemoved } = fakeEls();
  renderShot(els, shot);
  assert.ok(homeAdded.includes("hidden"));
  assert.ok(shotRemoved.includes("hidden"));
  assert.equal(els.img.src, shot.src);
  assert.equal(els.dims.textContent, "1920×1080");
});

test("a 4K capture is scaled to fit the preview box", () => {
  const { els } = fakeEls();
  renderShot(els, { ...shot, width: 3840, height: 2160 });
  const w = parseInt(els.img.style.width, 10);
  const h = parseInt(els.img.style.height, 10);
  assert.ok(w <= 840, `width ${w} must fit the box`);
  assert.ok(h <= 600, `height ${h} must fit the box`);
  assert.ok(
    Math.abs(w / h - 3840 / 2160) < 0.01,
    "the aspect ratio must survive the scaling",
  );
});
