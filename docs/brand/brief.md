# Design brief — minshot

**Summary**: A minimal Windows 11 screenshot-to-terminal utility — capture a region once, drop or paste it
straight into a terminal / OpenCode session. No history, no save dialogs, no friction.

**Audience**: Developers and terminal users on Windows (OpenCode / TUI workflows). They meet the mark as a
Windows app icon, taskbar/tray icon, the 32 px tile in the app header, and in a README.

**Brand adjectives**: minimal, instant, precise, quiet, native. **Avoid feeling**: cute, corporate, "camera-y",
suite-like.

**Promise / big idea**: "Screenshot → terminal, minus the saving." One gesture; the file never exists.

**Competitive landscape** (screenshot tools): literal capture chrome and blue/colour gradients — Snipping Tool
(scissors), Snagit / CleanShot (lens circles + shutter), ShareX (X), Flameshot (flame), Lightshot (feather),
Greenshot (camera), Gyazo. Open territory: the *subtraction* itself — a monochrome, almost anti-icon mark; and
terminal-native cues (none of them sell "gone faster"). The current minshot icon is a literal blue selection
marquee with four handles — to be replaced.

**Mark type hypothesis**: a symbol that works alone (app icon) + lowercase wordmark lockup. Letter-heavy
candidates explored: squared-m letterform, two-corner frame, minus window.

**Must work on**: 16 px favicon, Windows app icon (tile), 32 px in-app tile, one colour, light + dark,
monochrome printing.

**Constraints**: shadcn zinc palette (#09090b / #fafafa); system font stack (Inter / Segoe UI); current equity
worth keeping is only the "selection frame" idea itself. No blue, no extra dependencies.

**Success criteria**: reads at 16 px; not a camera / scissors / marquee cliché; distinct on a shelf next to
Snagit, ShareX, Flameshot, Snipping Tool; the tile at 32 px is crisp in one colour.

**Deliverables**: symbol + lockup concepts first (checkpoint), then the chosen direction as a full kit —
colour, lockups, favicon/ICO/app-icon set, usage notes.

**Assumptions I made** (correct me): name stays lowercase "minshot"; no existing identity to preserve beyond
the selection-frame idea; monochrome-first; English UI.

## Word map

- **minshot** → min (minimise, minus, minimal, minimum) · shot (capture, snapshot, click, region)
- **screenshot** → selection frame, crop marks, corners, ✅ marquee, Ctrl+Shift+4, clipboard, paste, terminal
- **minimal** → one line, a dash, a dot, a single square, reduction, white space
- **instant** → one gesture, a click, direct, no dialog
- **terminal** → prompt ❯, block cursor ▮, monospace, `>_`
- **"minus the saving"** → minus sign, subtraction, "the file never exists"

Intersections chosen for concepting:
1. A window reduced to its dash — the app minimises itself; the minus is the logo. (minimise + minus + window)
2. The capture reduced to two corners — drag start and drag end; everything else removed. (gesture + minimal)
3. The name reduced to one letter — a squared m whose top corners are the capture frame. (name + frame)

Rejected: prompt chevrons (crowded: Jira, Switch, web.dev, Terminal.app), `>_` in a window (Terminal.app,
web.dev own it), cut-corner tile (Qt and many dev tiles), scissors/camera/marquee (category clichés),
dashed marquee letters (dies at 16 px), i-dot pixel (kept at most as a wordmark detail in the kit).
