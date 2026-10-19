/// The launcher's "save to" row: where the screenshot is saved.
///
/// Kept apart from the DOM so the label rules can be tested without a
/// browser: the row is the only place the user can change where
/// minshot puts the file.

/// The elements the row touches.
export interface SaveDirEls {
  dir: { textContent: string; title: string };
  reset: { hidden: boolean };
  change: { disabled: boolean };
}

/// What the row shows for a folder. `null` means the temp folder: the
/// file is ours and goes away with the next capture.
export function saveDirLabel(dir: string | null): string {
  if (!dir) return "Temp folder";
  const parts = dir.split(/[\\/]/).filter(Boolean);
  return parts.length > 0 ? parts[parts.length - 1] : dir;
}

/// The row for the current choice.
export function renderSaveDir(els: SaveDirEls, dir: string | null): void {
  els.dir.textContent = saveDirLabel(dir);
  els.dir.title = dir
    ? `Screenshots are saved in ${dir}`
    : "The temp folder — the file is deleted with the next capture";
  // The reset only makes sense when there is something to reset.
  els.reset.hidden = !dir;
}
