# macOS Support Implementation Plan

**Goal:** Produce a production-quality `.icns` icon from the existing `data/luminaire.svg` and commit both the iconset source files and the generated binary.

**Architecture:** Use `rsvg-convert` (from `brew install librsvg`) to export 10 PNG files at the required Apple iconset sizes, then `iconutil` to convert the `.iconset/` directory to `luminaire.icns`. Both artifacts are committed to the repo. The CMake build uses the committed `.icns` directly — no build-time icon generation.

**Tech Stack:** rsvg-convert (librsvg), iconutil (macOS built-in), Homebrew

**Scope:** Phase 2 of 3 from original design

**Codebase verified:** 2026-05-14

---

## Acceptance Criteria Coverage

### macos-support.AC5: Icon asset quality
- **macos-support.AC5.1 Success:** `.app` bundle icon renders clearly in Finder at small, medium, and large icon sizes
- **macos-support.AC5.2 Success:** `data/luminaire.icns` and `data/luminaire.iconset/` are both committed to the repo

---

<!-- START_TASK_1 -->
### Task 1: Install rsvg-convert

**Verifies:** None (tool prerequisite)

**Step 1: Install librsvg**

```bash
brew install librsvg
```

**Step 2: Verify installation**

```bash
rsvg-convert --version
```

Expected: prints version like `rsvg-convert version 2.58.x`

No commit needed — this is a developer tool, not a repo change.
<!-- END_TASK_1 -->

<!-- START_TASK_2 -->
### Task 2: Generate iconset PNG files

**Verifies:** macos-support.AC5.1, macos-support.AC5.2

**Files:**
- Create: `data/luminaire.iconset/` directory with 10 PNG files

**Step 1: From the repo root, run:**

```bash
mkdir -p data/luminaire.iconset

rsvg-convert --width=16   --height=16   --keep-aspect-ratio -o data/luminaire.iconset/icon_16x16.png       data/luminaire.svg
rsvg-convert --width=32   --height=32   --keep-aspect-ratio -o data/luminaire.iconset/icon_16x16@2x.png    data/luminaire.svg
rsvg-convert --width=32   --height=32   --keep-aspect-ratio -o data/luminaire.iconset/icon_32x32.png       data/luminaire.svg
rsvg-convert --width=64   --height=64   --keep-aspect-ratio -o data/luminaire.iconset/icon_32x32@2x.png    data/luminaire.svg
rsvg-convert --width=128  --height=128  --keep-aspect-ratio -o data/luminaire.iconset/icon_128x128.png     data/luminaire.svg
rsvg-convert --width=256  --height=256  --keep-aspect-ratio -o data/luminaire.iconset/icon_128x128@2x.png  data/luminaire.svg
rsvg-convert --width=256  --height=256  --keep-aspect-ratio -o data/luminaire.iconset/icon_256x256.png     data/luminaire.svg
rsvg-convert --width=512  --height=512  --keep-aspect-ratio -o data/luminaire.iconset/icon_256x256@2x.png  data/luminaire.svg
rsvg-convert --width=512  --height=512  --keep-aspect-ratio -o data/luminaire.iconset/icon_512x512.png     data/luminaire.svg
rsvg-convert --width=1024 --height=1024 --keep-aspect-ratio -o data/luminaire.iconset/icon_512x512@2x.png  data/luminaire.svg
```

**Step 2: Verify all 10 files exist**

```bash
ls data/luminaire.iconset/
```

Expected output (10 files):
```
icon_128x128.png     icon_256x256.png     icon_512x512.png
icon_128x128@2x.png  icon_256x256@2x.png  icon_512x512@2x.png
icon_16x16.png       icon_32x32.png
icon_16x16@2x.png    icon_32x32@2x.png
```

**Step 3: Spot-check visual quality**

```bash
open data/luminaire.iconset/icon_256x256.png
```

Expected: yellow lightbulb with gradient glow, gray base/threads, visible at 256×256.
<!-- END_TASK_2 -->

<!-- START_TASK_3 -->
### Task 3: Generate `data/luminaire.icns` and commit everything

**Verifies:** macos-support.AC5.1, macos-support.AC5.2

**Files:**
- Create: `data/luminaire.icns` (replaces Phase 1 placeholder)

**Step 1: Run iconutil from the repo root**

```bash
iconutil -c icns data/luminaire.iconset -o data/luminaire.icns
```

**Step 2: Verify the .icns file is non-trivial**

```bash
ls -lh data/luminaire.icns
```

Expected: file is several KB or larger (not a 1-byte placeholder)

**Step 3: Rebuild to verify the real icon bundles correctly**

```bash
cmake --build build
open build/luminaire.app
```

Expected: The app icon in Finder shows the lightbulb design. (If Finder shows a blank icon, quit and reopen Finder or run `touch build/luminaire.app` to force a cache refresh.)

**Step 4: Commit both iconset and .icns**

```bash
git add data/luminaire.iconset/ data/luminaire.icns
git commit -m "feat: add production .icns icon from SVG (iconset + generated file)"
```

Both `data/luminaire.iconset/` and `data/luminaire.icns` are committed. The build uses the committed binary directly — `cmake --build` never runs rsvg-convert or iconutil. Future maintainers can regenerate `luminaire.icns` by re-running iconutil against the iconset.
<!-- END_TASK_3 -->
