import { test } from "node:test";
import assert from "node:assert/strict";
import { renderSaveDir, saveDirLabel, type SaveDirEls } from "./settings.ts";

function fakeEls() {
  const els: SaveDirEls = {
    dir: { textContent: "", title: "" },
    reset: { hidden: false },
    change: { disabled: false },
  };
  return { els, state: els };
}

test("no folder means the temp folder", () => {
  assert.equal(saveDirLabel(null), "Temp folder");
});

test("the row shows the name of the folder, not the whole path", () => {
  assert.equal(saveDirLabel("C:\\Users\\Mark\\Pictures"), "Pictures");
  assert.equal(saveDirLabel("C:/Users/Mark/Pictures"), "Pictures");
});

test("a folder that is only a drive letter still shows something", () => {
  assert.equal(saveDirLabel("C:\\"), "C:");
});

test("the temp folder has no reset button", () => {
  const { els } = fakeEls();
  renderSaveDir(els, null);
  assert.equal(els.dir.textContent, "Temp folder");
  assert.equal(els.reset.hidden, true);
  assert.match(els.dir.title, /temp folder/);
});

test("a chosen folder shows its name and can be reset", () => {
  const { els } = fakeEls();
  renderSaveDir(els, "C:\\Users\\Mark\\Pictures");
  assert.equal(els.dir.textContent, "Pictures");
  assert.equal(els.reset.hidden, false);
  assert.match(els.dir.title, /C:\\Users\\Mark\\Pictures/);
});
