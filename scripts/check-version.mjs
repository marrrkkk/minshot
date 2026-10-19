// Fail when the git tag and the version of the app disagree.
//
// The installer is named after the version in tauri.conf.json, so a tag
// that does not match it ships a file called 0.1.0 under a v0.2.0 release.
//
// Usage: node scripts/check-version.mjs v0.1.0

import { readFileSync } from "node:fs";

const read = (path) => readFileSync(path, "utf8");
const tag = (process.argv[2] ?? "").replace(/^v/, "");

const versions = {
  "package.json": JSON.parse(read("package.json")).version,
  "Cargo.toml": read("src-tauri/Cargo.toml").match(/^version = "(.+)"$/m)?.[1],
  "Cargo.lock": read("src-tauri/Cargo.lock").match(
    /\[\[package\]\]\s*\nname = "minshot"\s*\nversion = "(.+)"/,
  )?.[1],
  "tauri.conf.json": JSON.parse(read("src-tauri/tauri.conf.json")).version,
};

const wrong = Object.entries(versions).filter(([, v]) => v !== tag);
if (!tag || wrong.length > 0) {
  console.error(`the tag ${process.argv[2] ?? "(none)"} does not match:`);
  console.error(versions);
  process.exit(1);
}
console.log(
  `version ${tag} matches package.json, Cargo.toml, Cargo.lock and tauri.conf.json`,
);
