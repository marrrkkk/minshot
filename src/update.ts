/// The updater: a newer version of minshot, found on GitHub.
///
/// The check runs once at startup. Nothing is downloaded until the user
/// asks for it: a screenshot tool must not spend bandwidth behind their
/// back.

import type { DownloadEvent } from "@tauri-apps/plugin-updater";

/// The message the launcher shows when a new version is waiting.
export function updateNotice(version: string): string {
  return `Version ${version} is ready to install.`;
}

/// The message while the update downloads.
export function updateProgress(done: number, total: number): string {
  if (!total) return "Downloading the update…";
  return `Downloading the update… ${Math.round((done / total) * 100)}%`;
}

/// How many bytes are down after an event. The plugin reports the chunk
/// that just arrived, so the chunks have to be added up.
export function downloadedSoFar(event: DownloadEvent, already: number): number {
  if (event.event === "Progress") return already + event.data.chunkLength;
  return already;
}
