# IcedLens

Lightweight, internationalized image viewer powered by the [Iced](https://iced.rs/) GUI toolkit.

<!-- Badges: replace <ORG>/<REPO> when repository is published -->
<!-- Example GitHub Actions badge (uncomment once CI is configured) -->
<!-- [![Build](https://github.com/<ORG>/<REPO>/actions/workflows/ci.yml/badge.svg)](https://github.com/<ORG>/<REPO>/actions) -->
[![License: MPL-2.0](https://img.shields.io/badge/License-MPL--2.0-brightgreen.svg)](LICENSE)
![Status: Experimental](https://img.shields.io/badge/status-experimental-orange)
<!-- Optional future crates.io badge -->
<!-- [![Crates.io](https://img.shields.io/crates/v/iced_lens.svg)](https://crates.io/crates/iced_lens) -->

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
IcedLens aims to provide a simple, privacy‑friendly viewer focusing on responsive zoom ergonomics, clean layout, and multilingual support—without bundling heavy editing logic. It is designed as a foundation that can evolve toward a minimal asset inspector or media desk utility.

## 2. Features
- Image formats: JPEG, PNG, GIF (static), TIFF, WebP, BMP, ICO
- SVG rasterization for scalable assets
- Fit‑to‑window vs manual zoom retention
- Cursor‑aware Ctrl+Scroll zoom (only when hovering the image)
- Centered image with dynamic padding when smaller than viewport
- Persistent preferences (language, zoom step, fit toggle) via TOML config
- Fluent‑based runtime language switching (English, French initial set)
- Modular architecture prepared for future extensions (e.g. editing, video, annotations)

## 3. Screenshots
Screenshots will be added once UI stabilizes. Feel free to open an issue and propose layout improvements.

## 4. Installation & Requirements
### Rust Toolchain
Requires Rust >= 1.78 (stable recommended). Install via:
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

ARGS:
	<IMAGE_PATH>     Path to image file to open
```

## 6. Configuration & Preferences
User config is stored in a platform‑appropriate directory (implementation detail: uses a TOML file). Currently persisted:
- `language`
- `fit_to_window`
- `zoom_step`

Resetting config: remove the file and restart; defaults will regenerate.

## 7. Internationalization (i18n)
Localization powered by [Fluent](https://projectfluent.org/).
- Translation bundles loaded at startup.
- Runtime language switching updates UI without restart.
- Current locales: `en-US`, `fr`.
To contribute a new locale, see `CONTRIBUTING.md` (translation workflow section).

## 8. Zoom & Viewing Model
- Manual zoom retains last chosen level when toggling fit‑to‑window off.
- Fit zoom recalculates dynamically based on viewport rectangle (not cached stale dimensions).
- Ctrl+Scroll zoom input gated: only active if pointer overlaps the image (prevents scroll conflicts with sidebars or gutters).
- Padding ensures small images remain visually centered.
- Zoom step is clamped to a reasonable range to avoid extreme jumps.

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
Planned / aspirational items (subject to change):
- Animated GIF/WebP frame playback
- Basic image metadata panel (EXIF)
- Overlay HUD
- Configurable background theme
- Richer argument parsing (additional flags)
- Packaging (AppImage / Flatpack)

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
**Will it become an editor?** Editing may appear as opt‑in extensions; core viewer stays lean.
**How do I report a bug?** Open an issue with OS, Rust version, steps, and logs if available.

---

Happy viewing!
