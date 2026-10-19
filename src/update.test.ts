import { test } from "node:test";
import assert from "node:assert/strict";
import { updateNotice, updateProgress } from "./update.ts";

test("the notice names the version that is waiting", () => {
  assert.equal(updateNotice("0.2.0"), "Version 0.2.0 is ready to install.");
});

test("the progress shows how far the download is", () => {
  assert.equal(updateProgress(50, 200), "Downloading the update… 25%");
  assert.equal(updateProgress(200, 200), "Downloading the update… 100%");
});

test("a download of unknown size still says something", () => {
  assert.equal(updateProgress(0, 0), "Downloading the update…");
});
