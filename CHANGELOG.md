# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Directory path as CLI argument:** The application now accepts a directory path as a command-line argument. When provided, it scans the directory for media files and opens the first one (based on the current sorting settings). If no supported media is found, the application starts without loading any file.
- **Empty state view:** The application now displays a welcoming UI when no media is loaded:
  - Shows a large icon, title, and helpful instructions
  - Provides an "Open File" button to launch the file picker dialog
  - Accepts drag-and-drop of files and folders anywhere in the window
  - Disables Edit and Info buttons when no media is present
  - Supports both single files and directories (loads first media file in directory)
- **Remember last opened directory:** The file picker dialog now opens in the directory of the last successfully loaded media file, persisted across sessions.
- **Automatic error notification cleanup:** Load error notifications are automatically dismissed when a media file is successfully loaded, keeping the UI in sync with the application state.
- **Dynamic window title:** The window title now displays contextual information:
  - Shows "AppName" when no media is loaded
  - Shows "FileName.ext - AppName" when a media file is loaded
  - Shows "*FileName.ext - AppName" when the image editor has unsaved changes on an existing file
  - Shows "New Image - AppName" when editing a captured video frame (no asterisk since it's a new document)
- **Application executable icons:** The application now has proper icons on all platforms:
  - Windows: ICO embedded in executable via `build.rs`
  - macOS: ICNS for `.app` bundle via `cargo-bundle`
  - Linux: PNG icons for `.deb` package via `cargo-bundle`
- **cargo-bundle support:** Added `[package.metadata.bundle]` configuration for creating macOS `.app` and Linux `.deb` packages with proper icons.
- **Sidebar collapse buttons:** Both sidebars (metadata panel and image editor) now use consistent double chevron icons for expand/collapse actions, following UX best practices for panel controls.

### Changed
- **Image editor sidebar toggle:** Replaced hamburger menu icon (☰) with double chevron icons (`<<`/`>>`) for sidebar expand/collapse, improving visual consistency across the application.
- **Auto-collapse metadata sidebar in fullscreen:** The metadata sidebar now automatically closes when entering fullscreen mode, providing an unobstructed view.
- **Non-blocking error notifications:** Media loading errors (invalid path, corrupted file, timeout) now display as toast notifications instead of a modal error panel. This provides a non-blocking UX and preserves the current view when navigation fails.
- **Migrated from SVG to PNG icons:** All UI icons now use pre-rendered PNG images instead of SVG for consistent cross-platform rendering and improved performance on Windows. Icons are cached using `OnceLock` for optimal memory usage.
- **Reorganized icon assets:** Icons are now organized in a cleaner structure:
  - `assets/branding/` - Application icon (SVG source, PNG exports, ICO, ICNS)
  - `assets/icons/source/` - SVG sources for UI icons
  - `assets/icons/png/dark/` - Dark icons for light backgrounds
  - `assets/icons/png/light/` - Light icons for dark backgrounds (overlays, HUD)

### Fixed
- **Frame capture and step unavailable after opening video:** Capture frame and step forward now work immediately after opening a video without requiring playback first.
- **Frame stepping skipping frames:** Stepping through video frames one by one no longer skips frames due to FFmpeg decoder buffering.
- **Frame navigation after stepping backward:** Stepping forward after stepping backward now correctly replays frames from history instead of decoding new frames.
- **Step backward button not disabling:** The step backward button now correctly disables when frame history is exhausted.
- **Frame history size setting not saved:** The `frame_history_mb` setting is now correctly persisted to `settings.toml`.
- **Frame history size setting ignored:** The `frame_history_mb` setting now correctly controls the maximum memory used for frame history.
- **Navigation arrows unresponsive during video playback:** Clicking near (but not exactly on) the navigation arrows during video playback now correctly triggers navigation instead of shifting the viewport. The clickable zone has been extended to 80px on each edge.

## [0.3.0] - 2025-12-14

### Added
- **Media metadata panel:** new Info button in navbar displays technical information about the current media file.
  - For images: EXIF data including camera make/model, exposure settings, focal length, and GPS coordinates (decimal degrees).
  - For videos: codec, container format, bitrate, duration, and frame rate information.
  - Panel uses push layout in windowed mode, overlay in fullscreen mode.
  - Toggle with the `I` keyboard shortcut or Info button in navbar.
- **Light adjustment tool:** brightness and contrast sliders in the image editor with live preview and full undo/redo support.
- **Help screen enhancements:** tool descriptions now display their corresponding icons, and a new "Mouse Interactions" section documents double-click fullscreen, scroll wheel zoom, and drag to pan.
- **Toast notification system:** visual feedback for user actions replaces silent console output.
  - Success notifications for save, delete, copy, and frame capture operations.
  - Warning notifications for configuration issues (corrupted settings, permission errors).
  - Error notifications for failed operations requiring user acknowledgment.
  - Auto-dismiss for success/info (3s) and warnings (5s); errors require manual dismiss.
- **Path override CLI arguments:** `--data-dir` and `--config-dir` allow overriding default directories for portable installations or testing.
- **Environment variable overrides:** `ICED_LENS_DATA_DIR` and `ICED_LENS_CONFIG_DIR` for CI/CD and portable deployments.
- **Keyboard seek step setting:** configurable time skip for arrow keys during video playback (0.5–30 seconds), accessible from the Settings screen (Video section).
- **Video playback preferences persistence:** volume, mute state, and loop toggle are now saved to `settings.toml` and restored on startup.
- **Remember last Save As directory:** the file dialog now opens in the last used save location, persisted across sessions.
- **Additional translations:** Spanish (`es`), German (`de`), and Italian (`it`) are now available in the language selector.

### Changed
- **Iced 0.14.0 upgrade:** migrated to the latest Iced framework version with VideoShader for persistent GPU textures.
- **UI style harmonization:** centralized design tokens for typography, spacing, borders, and opacity. All hardcoded values replaced with semantic tokens for consistent styling across screens.
- **Sectioned configuration file:** `settings.toml` now uses TOML sections (`[general]`, `[display]`, `[video]`, `[fullscreen]`) for better organization. Existing flat config files are automatically migrated on load.
- Default keyboard seek step reduced from 5 seconds to 2 seconds.
- Configuration and state loading now provides user feedback when falling back to defaults.
- **Project structure reorganization:** moved `config/` and `i18n/` modules into `app/` as they are application infrastructure rather than independent business logic. Public API unchanged via re-exports.
- Image editor navigation now skips videos automatically instead of showing a "video editing unsupported" notification.

### Fixed
- Fit-to-window now correctly updates when the hamburger menu collapses/expands.
- Video seeking with arrow keys no longer "snaps back" when held down.
- Image centering recalculates correctly on layout changes.
- Navigation index now updates correctly after deleting a media file.
- Video error messages now correctly display the codec name (e.g., "H264 not supported") instead of showing a placeholder.
- System locale detection now falls back to base language (e.g., `fr_FR` correctly matches `fr` translation).

## [0.2.0] - 2025-12-12

### Added
- **Video playback:** full video player with play/pause, seek bar, volume control, mute, and loop toggle for MP4, AVI, MOV, MKV, and WebM files.
- **Animated format support:** animated GIF and WebP files are detected and play automatically with the same video controls.
- **Frame-by-frame navigation:** step forward/backward through video frames when paused; backward stepping uses a frame history buffer.
- **Frame capture:** capture current video frame and open it in the editor for saving (PNG, JPEG, WebP).
- **Audio normalization:** optional setting to normalize audio levels across videos.
- **Video frame caching:** configurable frame cache size (16–512 MB) to optimize playback performance.
- **Frame history size:** configurable memory (32–512 MB) for backward frame stepping.
- **Fullscreen video controls:** video playback controls (play/pause, seek, volume) available in fullscreen mode alongside zoom and fit-to-window options; controls appear on mouse movement and auto-hide after a configurable delay.
- **Navigation feedback:** loop indicators on navigation arrows at list boundaries and a position counter (e.g. `3/12`) in fullscreen.
- **Application theme mode:** configurable theme mode (System / Light / Dark) stored in `settings.toml` and editable from the Settings view.
- **Scrollable settings:** settings screen now scrolls vertically to accommodate video-related options.

### Changed
- Edit button is disabled when viewing videos (editing not supported for video files).
- **MSRV**: Minimum Supported Rust Version upgraded from 1.78 to 1.92.0.

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

[unreleased]: https://codeberg.org/Bawycle/iced_lens/compare/v0.3.0...HEAD
[0.3.0]: https://codeberg.org/Bawycle/iced_lens/compare/v0.2.0...v0.3.0
[0.2.0]: https://codeberg.org/Bawycle/iced_lens/releases/tag/v0.2.0
[0.1.0]: https://codeberg.org/Bawycle/iced_lens/releases/tag/v0.1.0
