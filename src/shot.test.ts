import { test } from "node:test";
import assert from "node:assert/strict";
import { fitShot, planShot, renderShot } from "./shot.ts";
import type { ShotEls, ShotPayload } from "./shot.ts";

// Minimal stand-ins for the elements renderShot touches. The point is the
// real function's effect on them, not the stand-ins themselves.
function fakeEls() {
  const homeAdded: string[] = [];
  const homeRemoved: string[] = [];
  const shotAdded: string[] = [];
  const copyAdded: string[] = [];
  const els: ShotEls = {
    img: { src: "", style: { width: "", height: "" } },
    dims: { textContent: "", title: "" },
    copy: {
      title: "",
      classList: {
        add: (c: string) => copyAdded.push(c),
        remove: () => {},
      },
    },
    home: {
      classList: {
        add: (c: string) => homeAdded.push(c),
        remove: (c: string) => homeRemoved.push(c),
      },
    },
    shot: {
      classList: {
        add: (c: string) => shotAdded.push(c),
        remove: () => {},
      },
    },
  };
  return { els, homeAdded, homeRemoved, shotAdded, copyAdded };
}

const shot: ShotPayload = {
  src: "http://asset.localhost/one.png",
  width: 1920,
  height: 1080,
  clipboard: true,
  scale: 1,
  availWidth: 1200,
  availHeight: 900,
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

test("the shot view replaces the launcher", () => {
  const { els, homeRemoved, shotAdded } = fakeEls();
  renderShot(els, shot);
  assert.ok(homeRemoved.includes("active"));
  assert.ok(shotAdded.includes("active"));
  assert.equal(els.img.src, shot.src);
  assert.equal(els.dims.textContent, "1920 × 1080");
});

test("a capture that did not reach the clipboard is marked", () => {
  const { els, copyAdded } = fakeEls();
  renderShot(els, { ...shot, clipboard: false });
  assert.ok(copyAdded.includes("warn"));
  assert.match(els.copy.title, /failed/);
});

// --- the plan: the image size and the window size that holds it ---

const CHROME = 40; // the toolbar
const PAD = 12; // the space around the image

test("a capture is shown one device pixel per captured pixel", () => {
  const plan = planShot(1920, 1080, 1, 4000, 3000, CHROME, PAD);
  assert.deepEqual(
    { width: plan.width, height: plan.height },
    { width: 1920, height: 1080 },
  );
});

test("on a scaled screen the image is shown at its true on-screen size", () => {
  // A 1920px wide region on a 150% screen was 1280 CSS px wide.
  const plan = planShot(1920, 1080, 1.5, 4000, 3000, CHROME, PAD);
  assert.deepEqual(
    { width: plan.width, height: plan.height },
    { width: 1280, height: 720 },
  );
});

test("a portrait capture keeps its shape", () => {
  const plan = planShot(458, 794, 1, 4000, 3000, CHROME, PAD);
  assert.equal(plan.width, 458);
  assert.equal(plan.height, 794);
});

test("a square capture does not turn into a wide window", () => {
  const plan = planShot(600, 600, 1, 4000, 3000, CHROME, PAD);
  assert.equal(plan.width, plan.height);
  // The window is as wide as the image plus the padding, and the toolbar
  // only makes it taller.
  assert.equal(plan.windowHeight, plan.windowWidth + CHROME);
});

test("a capture too big for the workspace is scaled down, not cropped", () => {
  const plan = planShot(3840, 2160, 1, 1000, 700, CHROME, PAD);
  assert.ok(plan.width <= 1000 - PAD * 2);
  assert.ok(plan.height <= 700 - CHROME - PAD * 2);
  assert.ok(
    Math.abs(plan.width / plan.height - 3840 / 2160) < 0.01,
    "the aspect ratio must survive the scaling",
  );
});

test("the window holds the image, the toolbar and the space around it", () => {
  const plan = planShot(800, 600, 1, 4000, 3000, CHROME, PAD);
  assert.equal(plan.windowWidth, plan.width + PAD * 2);
  assert.equal(plan.windowHeight, plan.height + CHROME + PAD * 2);
});

test("a small capture gets a small window", () => {
  const plan = planShot(120, 80, 1, 4000, 3000, CHROME, PAD);
  assert.equal(plan.width, 120);
  assert.equal(plan.windowWidth, 120 + PAD * 2);
});

test("a capture never grows past the space the workspace allows", () => {
  const plan = planShot(4000, 4000, 1, 900, 700, CHROME, PAD);
  assert.ok(plan.windowWidth <= 900);
  assert.ok(plan.windowHeight <= 700);
});
