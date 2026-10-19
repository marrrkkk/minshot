import { test } from "node:test";
import assert from "node:assert/strict";
import { Selection } from "./selection.ts";

test("a drag reports the rectangle it covers", () => {
  const s = new Selection();
  s.begin(100, 50);
  assert.deepEqual(s.update(300, 250), { x: 100, y: 50, w: 200, h: 200 });
});

test("a drag up and to the left is normalized", () => {
  const s = new Selection();
  s.begin(300, 250);
  assert.deepEqual(s.update(100, 50), { x: 100, y: 50, w: 200, h: 200 });
});

test("a click is not a selection", () => {
  const s = new Selection();
  s.begin(100, 50);
  assert.equal(s.end(101, 51), null);
});

test("a cancelled drag is not a selection", () => {
  const s = new Selection();
  s.begin(10, 10);
  s.cancel();
  assert.equal(s.end(400, 400), null);
});

test("moving without a drag reports nothing", () => {
  const s = new Selection();
  assert.equal(s.update(10, 10), null);
});

test("a drag that ends after the mouseup reports nothing twice", () => {
  const s = new Selection();
  s.begin(10, 10);
  assert.notEqual(s.end(400, 400), null);
  assert.equal(s.end(400, 400), null);
});
