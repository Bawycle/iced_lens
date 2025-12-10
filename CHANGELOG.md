# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - In Development

> **Note:** This version is currently under active development on the `feature/video_player` branch.
> Features listed here are planned or partially implemented but not yet released.

### Added
- **Video playback:** full video player with play/pause, seek bar, volume control, mute, and loop toggle for MP4, AVI, MOV, MKV, and WebM files.
- **Animated format support:** animated GIF and WebP files are detected and play automatically with the same video controls.
- **Audio normalization:** optional setting to normalize audio levels across videos.
- **Video frame caching:** configurable frame cache size (16–512 MB) to optimize playback performance.
- **Fullscreen video controls:** video playback controls (play/pause, seek, volume) available in fullscreen mode alongside zoom and fit-to-window options; controls appear on mouse movement and auto-hide after a configurable delay.
- **Navigation feedback:** loop indicators on navigation arrows at list boundaries and a position counter (e.g. `3/12`) in fullscreen.
- **Application theme mode:** configurable theme mode (System / Light / Dark) stored in `settings.toml` and editable from the Settings view.
- **Scrollable settings:** settings screen now scrolls vertically to accommodate video-related options.

### Changed
- Edit button is disabled when viewing videos (editing not supported for video files).

### Fixed
- Keyboard shortcuts work correctly in fullscreen mode.
- Overlay arrows and HUD remain visible across all viewer background themes (light, dark, checkerboard).

## [0.1.0] - 2025-12-02

### Added

#### Viewing Features
- Support for multiple image formats: JPEG, PNG, GIF, WebP, TIFF, BMP, ICO, and SVG
- Mouse wheel and toolbar zoom controls with configurable zoom step (1-200%)
- Fit-to-window toggle mode with automatic recalculation on window resize
- Grab-and-drag panning for navigating large images
- Multi-image browsing with keyboard arrow keys or overlay navigation arrows
- Automatic directory rescanning on each navigation (reflects added/removed files)
- Looping navigation (wraps around at directory boundaries)
- Fullscreen mode with multiple triggers: F11 key, double-click, or toolbar button
- HUD indicators showing scroll position when needed
- Cursor state feedback (grab/grabbing modes)
- Selectable background themes: light, dark, and checkerboard patterns
- Configurable sort order for image navigation: alphabetical, modified date, or created date

#### Editing Features
- Non-destructive editing pipeline with transformation history
- Rotate tool with 90-degree increments and instant apply
- Crop tool with interactive overlay, drag handles, and preset aspect ratios:
  - Free crop (no constraints)
  - Square (1:1)
  - Landscape ratios: 16:9, 4:3
  - Portrait ratios: 9:16, 3:4
- Resize tool with:
  - Slider control (10-200% range)
  - Width and height input fields
  - Aspect ratio lock toggle
  - Live preview with auto-commit on tool exit
- Unlimited undo/redo with full transformation replay
- Save and Save As functionality with format preservation
- Keyboard shortcuts for efficient workflow:
  - `E`: Enter editor mode
  - `Ctrl+S` (Cmd+S on macOS): Save current image
  - `Ctrl+Z` (Cmd+Z on macOS): Undo transformation
  - `Ctrl+Y` (Cmd+Y on macOS): Redo transformation
  - `Esc`: Cancel unsaved changes or exit editor

#### Internationalization
- Fluent-based localization system with runtime language switching
- Supported locales: English (en-US) and French (fr)
- Automatic locale detection from system settings
- CLI flag `--lang` to override system locale
- CLI flag `--i18n-dir` to load translations from custom directory (for testing)

#### Configuration & Persistence
- Platform-appropriate configuration directory using XDG standards (Linux) or equivalent
- TOML-based settings file storing:
  - UI language preference
  - Fit-to-window state
  - Zoom step increment
  - Background theme selection
  - Image navigation sort order
- Automatic configuration loading and saving
- Graceful fallback to defaults if config file is missing or corrupted

#### Performance & Quality
- Criterion-based benchmarks for image loading performance
- SVG rasterization using resvg + tiny-skia
- Optimized viewport tracking for smooth zoom and pan operations
- Cursor-aware zoom: Ctrl+Scroll zoom only activates when cursor is over image (prevents scrollbar conflicts)

#### Developer Experience
- Comprehensive unit tests for core functionality
- Integration tests for multi-module workflows
- Test-driven development workflow documented in project constitution
- Clippy linting with warnings denied in CI (`-D warnings`)
- Code formatting enforcement with rustfmt
- Security audit tooling integration (`cargo audit`)
- Documentation tests for public APIs

### Notes
- **Status**: Experimental (pre-1.0)
- **Testing**: Tested exclusively on Linux Mint 22.2 by the maintainer
- **Platform Support**: Primary development on Linux; macOS and Windows support via Iced cross-platform backends
- **MSRV**: Rust 1.78+
- **License**: Mozilla Public License 2.0 (MPL-2.0)

[unreleased]: https://codeberg.org/Bawycle/iced_lens/compare/v0.1.0...HEAD
[0.1.0]: https://codeberg.org/Bawycle/iced_lens/releases/tag/v0.1.0
