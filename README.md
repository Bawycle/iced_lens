# IcedLens

Lightweight, internationalized image and video viewer with image editing, powered by the [Iced](https://iced.rs/) GUI toolkit.

[![Release](https://img.shields.io/badge/release-v0.1.0-blue)](https://codeberg.org/Bawycle/iced_lens/releases)
[![License: MPL-2.0](https://img.shields.io/badge/License-MPL--2.0-brightgreen.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/Rust-1.92.0%2B-blue)](https://www.rust-lang.org)
![Locales](https://img.shields.io/badge/i18n-en--US|fr-green)
![Status: Pre-1.0](https://img.shields.io/badge/status-pre--1.0-orange)

> **Note**: This is a pre-1.0 release. The software is functional but still under active development. Testing has been performed exclusively on Linux Mint 22.2. Feedback and contributions are welcome!

</div>

## Table of Contents
1. Motivation
2. Features
3. Screenshots (Coming Soon)
4. Installation & Requirements
5. Usage
6. Configuration & Preferences
7. Internationalization (i18n)
8. Zoom & Viewing Model
9. Performance & Benchmarks
10. Development Workflow
11. Testing & Quality Gates
12. Security Notes
13. Roadmap
14. Contributing
15. License
16. FAQ

## 1. Motivation
IcedLens aims to provide a simple, privacy‑friendly image viewer and editor focusing on responsive zoom ergonomics, clean layout, and multilingual support. It includes essential editing tools (rotate, crop, resize) with a non-destructive workflow, while maintaining a lightweight footprint. It is designed as a foundation that can evolve toward a minimal asset inspector or media desk utility.

## 2. Features
### Viewing
- **Images:** JPEG, PNG, GIF, WebP, TIFF, BMP, ICO, SVG
- **Videos:** MP4, AVI, MOV, MKV, WebM (plus animated GIF/WebP)
- Zoom, pan, fit-to-window, fullscreen with auto-hiding controls
- Multi-media browsing with keyboard navigation
- Frame-by-frame video navigation and frame capture
- Fluent-based localization (en-US, fr)

### Editing
- **Non-destructive pipeline** with undo/redo history and Save / Save As flows
- **Rotate, Crop, Resize** tools share a live preview canvas
  - Rotate: 90° increments, instant apply
  - Crop: overlay with drag handles and preset ratios (Free, Square, 16:9, 9:16, 4:3, 3:4)
  - Resize: slider (10–200%), width/height inputs, aspect lock, and live preview (auto-commit when leaving the tool)
- **Keyboard shortcuts** keep the workflow fast (E to edit, Ctrl+S, Ctrl+Z/Y, Esc)

## 3. Screenshots
Screenshots will be added once UI stabilizes. Feel free to open an issue and propose layout improvements.

## 4. Installation & Requirements
### Rust Toolchain
Requires Rust >= 1.92.0 (stable recommended). Install via:
```bash
curl https://sh.rustup.rs -sSf | sh
```

### Platform Notes
- Linux: Ensure development packages for font rendering and Wayland/X11 are present (e.g. `libxcb`, `fontconfig`). Most distros already include what Iced needs.
- macOS: No extra system deps expected.
- Windows: Works with the default toolchain; use the MSVC build for best compatibility.

### Build From Source
```bash
git clone https://codeberg.org/Bawycle/iced_lens.git
cd iced_lens
cargo build --release
```
Binary: `target/release/iced_lens`

## 5. Usage
Open an image :
```bash
iced_lens /path/to/image.png
```
Specify language:
```bash
iced_lens --lang fr /path/to/image.jpg
```

### Command-Line Help
Display usage information:
```bash
iced_lens --help
```
Output includes syntax:
```
USAGE:
	iced_lens [OPTIONS] [IMAGE_PATH]

OPTIONS:
	-h, --help        Show help text
			--lang <id>   Set locale (en-US, fr)
			--i18n-dir <path> Override translation directory (defaults to assets/i18n/)

ARGS:
	<IMAGE_PATH>     Path to image file to open
```

### Navigation Controls
**Image Navigation:**
- Arrow keys or overlay arrows browse sibling images (loops automatically)
- Left-click + drag pans large images; HUD shows scroll percentage when needed
- Fullscreen via toolbar button, double-click, or **F11**; exit with **Esc**

**Zoom Controls:**
- Mouse wheel or toolbar buttons adjust zoom; fit-to-window keeps content centered

**Visual Feedback:**
- Cursor switches between grab/grabbing states and scrollbars stay hidden for a clean view

**Directory Scanning:**
- Each navigation step rescans the folder so added/removed files are reflected immediately

### Keyboard Shortcuts
**Viewer Mode:**
- **E**: Enter editor mode (images only)
- **F11**: Toggle fullscreen
- **Esc**: Exit fullscreen
- **← / →**: Navigate media or seek video
- **Space**: Play/pause video
- **M**: Toggle mute

**Editor Mode:**
- **Ctrl+S** (Cmd+S on macOS): Save current image
- **Ctrl+Z** (Cmd+Z on macOS): Undo last transformation
- **Ctrl+Y** (Cmd+Y on macOS): Redo transformation
- **Esc**: Cancel changes (if unsaved) or exit editor (if no changes)

## 6. Configuration & Preferences
User config is stored in a platform‑appropriate directory (TOML file). Configurable via the Settings screen:
- Language, theme, background, sort order
- Zoom step, fit-to-window
- Video: autoplay, volume, audio normalization, frame cache size, frame history size

Resetting: remove the config file and restart; defaults will regenerate.

## 7. Internationalization (i18n)
Localization powered by [Fluent](https://projectfluent.org/).
- Translation bundles loaded at startup.
- Runtime language switching updates UI without restart.
- Current locales: `en-US`, `fr`.
To contribute a new locale, see `CONTRIBUTING.md` (translation workflow section).
Override directory: pass `--i18n-dir /custom/translations` to load `.ftl` files from an alternate location. If the path is invalid, the application falls back to the built‑in `assets/i18n/` directory.

## 8. Zoom & Viewing Model
- Mouse wheel zoom in/out when the cursor is over the image
- Optional fit‑to‑window mode or manual zoom level
- Grab‑and‑drag panning for oversized images
- Reasonable zoom step limits to avoid extreme jumps

## 9. Performance & Benchmarks
Criterion benchmarks: see `benches/image_loading.rs`.
Run benchmarks:
```bash
cargo bench
```
Performance goals center on responsive zoom and fast first decode; no advanced caching yet.

## 10. Development Workflow
Common tasks:
```bash
cargo build
cargo test
cargo clippy --all --all-targets -- -D warnings
cargo fmt --all
```
Generate docs:
```bash
cargo doc --all-features --open
```

## 11. Testing & Quality Gates
- Unit & integration tests cover zoom logic, config persistence, and i18n loading.
- Linting enforced via `clippy` with warnings denied.
- (Optional) Coverage via `cargo tarpaulin`:
```bash
cargo tarpaulin --out Html
```
- Security audit (manual step):
```bash
cargo audit
```

## 12. Security Notes
This project does not process untrusted remote input; images are opened from local paths. Still, malformed files could trigger decoding edge cases in dependencies. Please report potential vulnerabilities via a private issue or (future) `SECURITY.md` contact channel.

## 13. Roadmap

### Planned / Aspirational
Future items (subject to change based on community feedback):
- Basic image/video metadata panel (EXIF, codec info)
- Temporary rotation in viewer (90° increments, session-only)
- Additional edit tools: flip (horizontal/vertical), brightness/contrast adjustments
- Filters: grayscale, sepia, blur, sharpen
- Video editing: trim, basic cuts
- Batch operations
- Hardware-accelerated video decoding (VAAPI/NVDEC)

## 14. Contributing
Contributions welcome! Start by reading [`CONTRIBUTING.md`](CONTRIBUTING.md).
Preferred steps:
1. Open an issue describing motivation
2. Discuss scope & approach
3. Submit focused PR with tests if feasible
4. Keep changes modular

Translations: Add a new `.ftl` file under `assets/i18n/` and update loader logic if needed.

## 15. License
Distributed under the Mozilla Public License 2.0. See [`LICENSE`](LICENSE).
Key concepts (informative only):
- File‑level copyleft: only modified files must be shared.
- Compatible with combining proprietary code as long as MPL rules are respected.
- Includes a limited patent grant; no trademark rights.
SPDX: `MPL-2.0`

### Icon Asset Exception
The application icon (`assets/icons/iced_lens.svg` and its PNG exports) is **not** covered by MPL-2.0. It uses a restricted license allowing only unmodified redistribution to represent IcedLens. See [`ICON_LICENSE.md`](ICON_LICENSE.md). SPDX reference: `LicenseRef-IcedLens-Icon`.

## 16. FAQ
**Why not GPL or MIT?** MPL offers balanced file‑level reciprocity without imposing network or full project copyleft.

**Does it support Windows/macOS?** Yes, via Iced's cross‑platform backends; primary development on Linux.

**Does it have editing capabilities?** Yes, for images: rotate, crop, resize with undo/redo. Video editing is not yet supported.

**How does editing work?** Editing is non-destructive—changes are only applied when you save. You can undo/redo transformations, and the original file is preserved until you explicitly save or save-as.

**How do I report a bug?** Open an issue with OS, Rust version, steps, and logs if available.

---

Happy viewing!
