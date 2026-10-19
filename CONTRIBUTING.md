# Contributing to minshot

Thanks for looking at minshot. It is a small Windows app: Rust + Tauri 2 in the
backend, plain TypeScript in the frontend, no UI framework.

## Getting set up

You need Windows 11, Node 20+, Rust stable (MSVC) and the Visual Studio 2022
Build Tools with the **VCTools** workload. See the README for the exact
commands.

```powershell
npm install
npm run tauri dev
```

## Before you open a pull request

```powershell
npm test                                          # frontend tests
cargo test --manifest-path src-tauri/Cargo.toml   # backend tests
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets
```

CI runs the same four commands on every push and pull request, so a green
local run means a green build.

Some tests need a real desktop (screen capture, the clipboard, the folder
picker). They are marked `#[ignore]` and skipped in CI. Run them yourself with:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -- --include-ignored
```

## What the app is allowed to be

minshot captures a region, puts it on the clipboard, and hands you the file.
No history, no accounts, no cloud, no annotations. Please keep it that way:
every feature that needs a server, a database or a settings screen is a
feature that makes the app heavier to install and slower to use.

The Windows-specific code lives in `src-tauri/src/win/`. Everything else is
plain Rust, so the app stays portable to read.

## Reporting a bug

Open an issue with what you did, what you expected, and what happened instead.
A screenshot of the window helps more than a description.

## Releasing

The maintainer cuts releases by tagging: bump the version in `package.json`,
`src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json` (all three must match),
commit, then push the tag. The release workflow builds the installer and
attaches it to the GitHub Release.
