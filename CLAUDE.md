# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

IcedLens is a lightweight, internationalized image viewer built with the Iced GUI toolkit. It provides responsive zoom ergonomics, clean layout, and multilingual support for viewing various image formats including JPEG, PNG, GIF, TIFF, WebP, BMP, ICO, and SVG.

## Project Constitution

This project adheres to strict governance principles documented in `.specify/memory/constitution.md` (Version 1.0.0). All development must comply with these core principles:

### Mandatory Development Practices

**Test-Driven Development (TDD)**: For any new feature or bug fix, unit tests MUST be written first to define requirements. The cycle is:
1. Write failing tests that define expected behavior
2. Write minimal code to make tests pass
3. Run `cargo test` to verify
4. Refactor while keeping tests green

**Code Quality Standards**:
- Run `cargo check` and `cargo clippy` regularly
- All clippy warnings must be addressed (denied in CI with `-D warnings`)
- Any ignored warning requires explicit and strong justification
- Favor functional programming principles: immutability, pure functions where reasonable
- Code comments must be in English and explain "why", not "what"

**Comprehensive Testing**: All code must include:
- Unit tests (inline `#[cfg(test)]` modules)
- Integration tests where applicable (`tests/` directory)
- Documentation tests for public APIs
- Performance must not regress (treat as bugs)

**Security-First Approach**:
- Follow secure coding practices
- Validate all inputs
- Run `cargo audit` regularly to check dependencies
- Address vulnerabilities proactively

**User-Centric Design**: All UI features must prioritize:
- Clean, elegant, intuitive interfaces
- Modern UX/UI best practices
- User needs and workflow efficiency

**Git Practices**:
- Consistent branching model (e.g., GitFlow)
- Descriptive commit messages
- Regular, small commits
- All commits should pass tests

## Development Commands

### Building
```bash
# Debug build
cargo build

# Release build (recommended for testing performance)
cargo build --release
```

Binary location: `target/release/iced_lens` (or `target/debug/iced_lens` for debug builds)

### Testing
```bash
# Run all tests (unit + integration)
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration

# Run benchmarks (image loading performance)
cargo bench
```

### Code Quality
```bash
# Run clippy (linting) - warnings are denied in CI
cargo clippy --all --all-targets -- -D warnings

# Format code
cargo fmt --all

# Check formatting without modifying files
cargo fmt --all -- --check

# Generate and open documentation
cargo doc --all-features --open

# Optional: Run coverage analysis
cargo tarpaulin --out Html

# Optional: Security audit
cargo audit
```

### Running the Application
```bash
# Open an image
./target/release/iced_lens /path/to/image.png

# Specify language
./target/release/iced_lens --lang fr /path/to/image.jpg

# Override translation directory
./target/release/iced_lens --i18n-dir /custom/path /path/to/image.png

# Display help
./target/release/iced_lens --help
```

## Architecture Overview

### Core Modules

**app.rs** (src/app.rs) - Central application logic
- `App` struct: Main application state including image data, zoom state, viewport tracking, i18n, and UI mode
- `Message` enum: All UI events (image loading, zoom changes, language switching, viewport updates, keyboard/mouse events)
- `AppMode` enum: Switches between Viewer and Settings screens
- Zoom system: Maintains both manual zoom and fit-to-window mode with automatic recalculation
- Cursor-aware Ctrl+Scroll zoom: Only active when pointer is over the image (not over scrollbars)
- Configuration persistence: Saves user preferences (language, fit_to_window, zoom_step) to TOML

**config/mod.rs** - User preferences management
- Platform-appropriate config directory using `dirs` crate
- TOML serialization for settings (language, fit_to_window, zoom_step)
- Config file location: `~/.config/IcedLens/settings.toml` (Linux) or platform equivalent
- Graceful fallback to defaults if config file is missing or invalid

**i18n/fluent.rs** - Internationalization system
- Fluent-based localization with runtime language switching
- Translation bundles embedded via `rust-embed` from `assets/i18n/`
- Supports: en-US, fr
- CLI flag `--i18n-dir` allows overriding translation directory for testing

**image_handler/mod.rs** - Image loading and decoding
- Unified `ImageData` structure with `iced::widget::image::Handle`, width, height
- Raster formats: Uses `image_rs` crate to decode JPEG, PNG, GIF, TIFF, WebP, BMP, ICO
- SVG support: Uses `resvg` + `tiny-skia` to rasterize SVGs to PNG in memory
- All images converted to RGBA8 format for consistent handling

**ui/viewer.rs** - Image display component
- Renders scaled image based on current zoom percentage
- Padding computation: Centers images smaller than viewport
- Dynamic viewport tracking for fit-to-window calculations

**ui/settings.rs** - Settings screen
- Language selector (radio buttons for available locales)
- Zoom step configuration with validation
- Input error handling with localized error messages

**error.rs** - Error types
- `Error` enum: Io, Svg variants
- Used throughout codebase for consistent error handling

**icon.rs** - Application icon loading
- Embeds window icon from `assets/icons/`
- PNG format for cross-platform compatibility

### Key Design Patterns

**Zoom State Management** (src/app.rs:424-502)
- Two zoom modes: manual and fit-to-window
- `manual_zoom_percent`: Last user-set zoom level, restored when disabling fit-to-window
- `zoom_percent`: Current display zoom (may be auto-calculated if fit_to_window is true)
- `compute_fit_zoom_percent()`: Dynamically recalculates based on current viewport bounds (not cached)
- Zoom clamped to 10-800% range
- Zoom step configurable (1-200% range)

**Viewport Tracking** (src/app.rs:425-430)
- `viewport_bounds`: Current scrollable viewport rectangle
- `viewport_offset`: Current scroll position
- `previous_viewport_offset`: Tracks scroll deltas
- Updates on window resize and scrollable viewport changes
- Used to compute fit zoom and image positioning

**Cursor-Over-Image Detection** (src/app.rs:545-597)
- `is_cursor_over_image()`: Checks if cursor is within image bounds (excluding scrollbar gutters)
- Used to gate Ctrl+Scroll zoom (prevents conflicts with sidebar/gutter scrolling)
- Accounts for scrollbar gutter width (16px) when image overflows viewport
- Intersects image bounds with viewport to handle partial visibility

**Configuration Persistence** (src/app.rs:599-613)
- `persist_zoom_preferences()`: Called after zoom/fit changes
- Disabled during tests (`cfg!(test)`)
- Saves to platform config directory via `config::save()`

**Event Handling** (src/app.rs:638-671)
- `handle_raw_event()`: Processes window, mouse, and keyboard events
- `handle_ctrl_zoom()`: Applies zoom on Ctrl+Scroll if cursor is over image
- Tracks modifier keys for Ctrl detection
- Updates cursor position for hover detection

### Translation Workflow

To add a new locale:
1. Create `assets/i18n/{locale}.ftl` (e.g., `de.ftl` for German)
2. Copy key structure from existing `.ftl` files (en-US.ftl, fr.ftl)
3. Translate all strings
4. Update locale loading logic in `i18n/fluent.rs` if needed (auto-detects `.ftl` files)

### Testing Strategy

**CRITICAL**: This project follows Test-Driven Development (TDD). Tests must be written BEFORE implementation code.

**Unit Tests**
- Inline `#[cfg(test)]` modules in each source file
- Write tests first to define expected behavior
- Config tests use `tempfile` crate with isolated temp directories
- App tests use `with_temp_config_dir()` to avoid config file conflicts between tests
- Image handler tests create temporary PNG/SVG files for loading validation

**Integration Tests**
- Located in `tests/integration.rs`
- End-to-end scenarios combining multiple modules
- Test interactions between components

**Benchmarks**
- `benches/image_loading.rs`: Measures image decode performance with Criterion
- Run with `cargo bench`
- Performance regressions are treated as bugs and must be addressed

**Documentation Tests**
- Include examples in doc comments with `///` or `//!`
- These are executed by `cargo test` to ensure accuracy

### Dependencies of Note

- `iced 0.13.1`: GUI framework (tokio runtime, SVG/image support)
- `image_rs 0.24`: Raster image decoding (aliased as `image` conflicts with Iced)
- `resvg 0.45.1` + `tiny-skia 0.11.4`: SVG rasterization
- `fluent-bundle 0.16.0`: Localization
- `pico-args 0.5.0`: Minimal CLI argument parsing
- `serde 1.0` + `toml 0.9.8`: Config serialization
- `dirs 6.0`: Platform-appropriate config paths

## Project Constraints

**MSRV**: Rust 1.78+ (see README.md badge, `rust-toolchain.toml`)

**License**: Mozilla Public License 2.0 (MPL-2.0) - file-level copyleft
- Modified files must be shared under MPL
- Application icon has separate restricted license (see ICON_LICENSE.md)

**Platform Support**: Linux (primary), macOS, Windows
- Linux may require libxcb, fontconfig dev packages

## Common Pitfalls

1. **TDD Violation**: Never write implementation code before tests. This violates the project constitution.
2. **Config Test Isolation**: Use `with_temp_config_dir()` wrapper to avoid XDG_CONFIG_HOME conflicts between parallel tests
3. **Image Crate Aliasing**: Import as `image_rs` to avoid conflict with `iced::widget::image`
4. **Zoom State Synchronization**: Always update both `zoom_percent` and `manual_zoom_percent` appropriately when changing zoom modes
5. **Viewport Bounds**: Check for `None` before using - bounds not set until first layout
6. **Scrollbar Gutter**: Account for 16px gutter when computing cursor-over-image detection
7. **Fit Zoom Recalculation**: Always call `refresh_fit_zoom()` after viewport changes if `fit_to_window` is true
8. **Clippy Warnings**: Never ignore clippy warnings without explicit justification. They are denied in CI.
9. **Performance Regressions**: Treat performance degradation as bugs requiring immediate attention.
10. **Security**: Always validate inputs, especially file paths and user-provided zoom values.

## Future Extensions (from Roadmap)

- Animated GIF/WebP frame playback
- EXIF metadata panel
- Overlay HUD
- Configurable background theme
- Advanced argument parsing
- Packaging (AppImage, Flatpak)
