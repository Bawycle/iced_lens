# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Updated `image` crate from 0.24 to 0.25.9 for latest features and bug fixes
- Updated `rfd` crate from 0.15 to 0.16.0 for improved file dialog functionality
- Removed redundant `fluent-syntax` dependency (already included transitively via `fluent-bundle`)

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
