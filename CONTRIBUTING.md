# Contributing to IcedLens

Thank you for your interest in contributing to IcedLens! We welcome contributions of all kinds: bug reports, feature suggestions, documentation improvements, translations, and code contributions.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [How Can I Contribute?](#how-can-i-contribute)
3. [Reporting Bugs](#reporting-bugs)
4. [Suggesting Features](#suggesting-features)
5. [Translation Contributions](#translation-contributions)
6. [Code Contributions](#code-contributions)
7. [Development Workflow](#development-workflow)
8. [Pull Request Process](#pull-request-process)
9. [Project Structure](#project-structure)
10. [Style Architecture](#style-architecture)

## Code of Conduct

This project adheres to a Code of Conduct that all contributors are expected to follow. Please read [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) before participating.

## How Can I Contribute?

There are many ways to contribute to IcedLens:

- **Report bugs** you encounter while using the application
- **Suggest new features** or improvements to existing ones
- **Translate** the interface into new languages
- **Improve documentation** (README, code comments, examples)
- **Submit code** for bug fixes or new features
- **Review pull requests** from other contributors
- **Share feedback** on your user experience

## Reporting Bugs

Before submitting a bug report:
1. Check the [issue tracker](https://codeberg.org/Bawycle/iced_lens/issues) to see if the issue has already been reported
2. Ensure you're using the latest version of IcedLens
3. Verify the bug is reproducible

When submitting a bug report, please include:
- **Operating System** (name and version, e.g., "Linux Mint 22.2", "macOS 14.0", "Windows 11")
- **IcedLens version** (from `--help` output or release version)
- **Steps to reproduce** the issue (be as specific as possible)
- **Expected behavior** vs. **actual behavior**
- **Logs or error messages** (if applicable)
- **Sample image** (if the issue is image-specific)

## Suggesting Features

Feature suggestions are welcome! Before opening a feature request:
1. Check if a similar feature request already exists
2. Consider whether the feature aligns with the project's goals (lightweight, privacy-focused image viewing and editing)

When suggesting a feature, please:
- Describe the **problem** the feature would solve
- Explain **why** this feature would be useful
- Provide **examples** or **mockups** if applicable
- Discuss potential **implementation approaches** (if you have ideas)

## Translation Contributions

IcedLens uses [Fluent](https://projectfluent.org/) for internationalization. Contributing translations is a great way to help make IcedLens accessible to more users worldwide.

**You don't need to be a developer to contribute translations!** The process is simple and accessible to anyone.

### How to Add or Update Translations

1.  **Locate Translation Files**: All translation files are in the `assets/i18n/` directory in the repository.

2.  **Naming Convention**: Translation files use the `.ftl` extension and are named according to their language code:
    - `en-US.ftl` for American English
    - `fr.ftl` for French
    - `es.ftl` for Spanish (example for a new language)
    - `de.ftl` for German (example for a new language)

3.  **Create or Edit Translation File**:
    - **For a new language**:
      1. Download or view the [`en-US.ftl`](assets/i18n/en-US.ftl) file as a reference
      2. Create a new file named after your language code (e.g., `pt-BR.ftl` for Brazilian Portuguese)
      3. Copy all the keys from `en-US.ftl` and translate the values
    - **For updates to an existing language**:
      1. Find and edit the corresponding `.ftl` file (e.g., `fr.ftl` for French)

4.  **Translation Format**: Each line follows this simple pattern:
    ```fluent
    key-name = Translated text here
    ```

    **Example** (comparing English and French):
    ```fluent
    # English (en-US.ftl)
    window-title = IcedLens Image Viewer
    zoom-in = Zoom In
    zoom-out = Zoom Out

    # French (fr.ftl)
    window-title = Visionneuse d'images IcedLens
    zoom-in = Zoom avant
    zoom-out = Zoom arrière
    ```

5.  **Important Translation Tips**:
    - **Keep the key names unchanged** (the part before `=`)
    - Only translate the text after the `=` sign
    - Preserve special placeholders like `{$variable}` if you see them
    - Maintain the same line structure as the original file
    - Don't worry if you're unsure about technical terms—we'll help during review!

6.  **Testing Your Translation** (optional):

    **Option A: If you have IcedLens installed**
    - Download a [release binary](https://codeberg.org/Bawycle/iced_lens/releases) for your system
    - Place your `.ftl` file in a custom directory (e.g., `/home/user/my_translations/`)
    - Run IcedLens with the custom translation directory:
      ```bash
      iced_lens --i18n-dir /home/user/my_translations/ --lang <your-language-code>
      ```
      Example: `iced_lens --i18n-dir /home/user/my_translations/ --lang es`

    **Option B: If you're a developer with Rust installed**
    - Use the development environment:
      ```bash
      cargo run -- --lang <your-language-code> /path/to/image.png
      ```

    **Option C: Submit without testing**
    - If testing isn't possible for you, that's perfectly fine! Submit your translation and the maintainers will test it for you.

7.  **Submit Your Translation**:
    - **Via Pull Request** (if you're familiar with Git/Codeberg):
      1. Fork the repository
      2. Add or modify the `.ftl` file in `assets/i18n/`
      3. Commit your changes
      4. Open a Pull Request

    - **Via Issue** (if you're not familiar with Git):
      1. Open a [new issue](https://codeberg.org/Bawycle/iced_lens/issues/new)
      2. Title: "Translation: [Language Name]"
      3. Attach your `.ftl` file or paste its contents
      4. We'll handle adding it to the repository for you!

### Translation Questions?

If you have any questions about translating, feel free to:
- Open an issue asking for clarification
- Check the existing translation files for examples
- Ask in your Pull Request—we're here to help!

## Code Contributions

Code contributions should follow the project's development practices and quality standards.

### Prerequisites

- **Rust 1.92.0 or newer** (install via [rustup](https://rustup.rs/))
- Familiarity with the [Iced GUI framework](https://iced.rs/)
- Understanding of the project structure (see below)

### Before You Start

1. **Open an issue** to discuss your proposed changes (unless it's a trivial fix)
2. **Wait for feedback** from maintainers to ensure alignment with project goals
3. **Fork the repository** and create a feature branch from `dev`

### Code Quality Standards

IcedLens follows strict quality standards to maintain code quality and reliability:

#### Test-Driven Development (TDD)

**Tests should be written before or alongside implementation code.** This ensures:
- Features work as expected from the start
- Changes don't break existing functionality
- Code is maintainable and well-documented

The TDD cycle:
1. Write tests that define expected behavior
2. Write code to make tests pass
3. Run `cargo test` to verify
4. Refactor while keeping tests green

#### Code Style

- Run `cargo fmt --all` before committing to format code consistently
- Run `cargo clippy --all --all-targets -- -D warnings` and fix all warnings
- Use English for all code comments and documentation
- Comments should explain **why**, not **what** (the code shows what)
- Favor clear, readable code over clever tricks

#### Testing Requirements

All code should include appropriate tests:
- **Unit tests** for individual functions/modules (`#[cfg(test)]` modules)
- **Integration tests** for multi-component workflows (`tests/` directory)
- **Documentation tests** for public APIs (examples in doc comments)

#### Security

- Follow secure coding practices
- Validate all user inputs (file paths, zoom values, etc.)
- Use proper error handling (avoid `unwrap()` on user-provided data)
- Run `cargo audit` to check for vulnerable dependencies

### Development Workflow

```bash
# Fork and clone the repository
git clone https://codeberg.org/YourUsername/iced_lens.git
cd iced_lens

# Create a feature branch from dev
git checkout dev
git checkout -b feature/your-feature-name

# Make changes following TDD:
# 1. Write tests first (or alongside implementation)
# 2. Implement feature
# 3. Ensure tests pass
cargo test

# Check code quality
cargo clippy --all --all-targets -- -D warnings
cargo fmt --all

# Build release version for testing
cargo build --release

# Run the application
./target/release/iced_lens /path/to/image.png

# Commit with descriptive messages
git add .
git commit -m "feat: Add descriptive commit message"

# Push to your fork
git push origin feature/your-feature-name
```

### Commit Message Guidelines

Follow conventional commits format for clarity:

- `feat: Add new feature description`
- `fix: Fix bug description`
- `docs: Update documentation`
- `test: Add tests for X`
- `refactor: Refactor component Y`
- `perf: Improve performance of Z`
- `chore: Update dependencies`

## Pull Request Process

1. **Ensure all tests pass**: `cargo test`
2. **Ensure code quality checks pass**: `cargo clippy --all --all-targets -- -D warnings`
3. **Format your code**: `cargo fmt --all`
4. **Update documentation** if needed (README.md, CHANGELOG.md, code comments)
5. **Provide a clear PR description**:
   - What problem does this solve?
   - How does it solve it?
   - Are there any breaking changes?
   - Screenshots (for UI changes)
6. **Reference related issues**: Use "Fixes #123" or "Relates to #456"
7. **Be responsive** to feedback and review comments
8. **Keep PRs focused**: One feature or fix per PR (split large changes into smaller PRs)

### PR Checklist

- [ ] Tests written and passing (`cargo test`)
- [ ] Clippy warnings addressed (`cargo clippy --all --all-targets -- -D warnings`)
- [ ] Code formatted (`cargo fmt --all`)
- [ ] Documentation updated (if applicable)
- [ ] CHANGELOG.md updated (for notable changes)
- [ ] Commit messages follow conventional commits format
- [ ] PR description is clear and complete

## Project Structure

Key files and directories:

```
iced_lens/
├── src/
│   ├── main.rs                 # Entry point
│   ├── app/                    # Main application logic and orchestration
│   │   ├── mod.rs              # App struct and Message enum
│   │   ├── paths.rs            # Application directory paths (data dir, config dir)
│   │   ├── persisted_state.rs  # Persisted application state (CBOR format)
│   │   ├── update.rs           # Message handling
│   │   └── view.rs             # UI rendering dispatch
│   ├── config/                 # Configuration persistence
│   ├── i18n/                   # Internationalization system (Fluent)
│   ├── media/                  # Media loading and transforms
│   │   ├── image/              # Image loading, decoding, caching
│   │   ├── video/              # Video file detection and metadata
│   │   └── navigator.rs        # Media list navigation (single source of truth)
│   ├── video_player/           # Video playback engine
│   │   ├── mod.rs              # VideoPlayer public API
│   │   ├── state.rs            # Playback state machine
│   │   ├── decoder.rs          # FFmpeg video decoding thread
│   │   ├── audio.rs            # Audio decoding and playback (cpal)
│   │   ├── sync.rs             # A/V synchronization (audio-master)
│   │   ├── frame_cache.rs      # Decoded frame caching
│   │   └── time_units.rs       # Time conversion utilities
│   ├── ui/                     # UI components
│   │   ├── viewer/             # Image/video viewer component
│   │   │   ├── component.rs    # Main viewer widget
│   │   │   ├── video_controls.rs # Playback controls toolbar
│   │   │   └── overlays.rs     # Loading, error, info overlays
│   │   ├── image_editor/       # Image editing (crop, resize, rotate)
│   │   ├── widgets/            # Custom reusable widgets
│   │   ├── styles/             # Component-specific styles
│   │   ├── design_tokens.rs    # Base design tokens (colors, spacing)
│   │   ├── theming.rs          # Theme system (light/dark modes)
│   │   ├── theme.rs            # Color helpers for viewer/editor
│   │   ├── settings.rs         # Settings panel
│   │   ├── help.rs             # Help/keyboard shortcuts panel
│   │   └── navbar.rs           # Navigation bar
│   ├── directory_scanner.rs    # Async directory scanning
│   └── error.rs                # Error types
├── assets/
│   ├── i18n/                   # Translation files (.ftl)
│   └── icons/                  # SVG icons
├── tests/                      # Integration tests
├── benches/                    # Performance benchmarks
├── CONTRIBUTING.md             # This file
├── CHANGELOG.md                # Release notes
├── README.md                   # User-facing documentation
└── Cargo.toml                  # Project metadata and dependencies
```

### Key Modules

#### `app/` - Application Core
The main application state machine with three states:
- **Browsing**: File browser view
- **Viewing**: Image/video viewer (includes video playback)
- **Editing**: Image editor with crop, resize, rotate tools

#### `video_player/` - Video Playback Engine
Handles video and audio decoding with A/V synchronization:
- Uses FFmpeg (via `ffmpeg-next`) for decoding
- Audio playback via `cpal` with `rubato` for resampling
- Audio-master sync model (video frames sync to audio PTS)
- Frame caching for smooth playback and frame stepping

#### `media/` - Media Handling
Loads and transforms images and video metadata:
- Image formats via `image` crate
- Video detection and metadata extraction
- EXIF orientation handling

#### `config/` - Configuration
User preferences and application settings:
- **`defaults.rs`**: Centralized default values for all constants (zoom, volume, cache sizes, etc.). **Always add new defaults here** rather than scattering them across the codebase. Includes compile-time validation to ensure constraints are valid.
- **`mod.rs`**: User settings persistence (`settings.toml`)

## Style Architecture

IcedLens uses a layered design system to ensure visual consistency. Understanding this architecture is essential before modifying colors, spacing, or component styles.

### Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Component Styles                         │
│              (src/ui/styles/*.rs)                          │
│         Button, Container, Overlay, Editor styles          │
├─────────────────────────────────────────────────────────────┤
│                    Theme System                             │
│     theming.rs (ColorScheme, AppTheme, ThemeMode)          │
│     theme.rs (viewer/editor color helpers)                 │
├─────────────────────────────────────────────────────────────┤
│                    Design Tokens                            │
│              (src/ui/design_tokens.rs)                     │
│     palette, opacity, spacing, sizing, radius, shadow      │
└─────────────────────────────────────────────────────────────┘
```

### Module Responsibilities

#### 1. Design Tokens (`src/ui/design_tokens.rs`)

The foundation layer. Defines all primitive values following the W3C Design Tokens standard:

| Module | Purpose | Example |
|--------|---------|---------|
| `palette` | Base colors | `palette::PRIMARY_500`, `palette::GRAY_900` |
| `opacity` | Opacity scale | `opacity::OVERLAY_STRONG` (0.7) |
| `spacing` | 8px grid spacing | `spacing::MD` (16.0) |
| `sizing` | Component sizes | `sizing::ICON_LG` (32.0) |
| `radius` | Border radii | `radius::SM` (4.0) |
| `shadow` | Shadow definitions | `shadow::MD` |

**Usage:**
```rust
use crate::ui::design_tokens::{palette, spacing, opacity};

// Create a semi-transparent overlay
let overlay_bg = Color { a: opacity::OVERLAY_STRONG, ..palette::BLACK };

// Use consistent spacing
let padding = spacing::MD; // 16px
```

#### 2. Theme System (`src/ui/theming.rs`)

Manages light/dark mode with semantic color mappings:

- **`ColorScheme`**: Defines surface, text, brand, semantic, and overlay colors
- **`ThemeMode`**: Enum for Light, Dark, or System detection
- **`AppTheme`**: Combines ColorScheme with current mode

**Usage:**
```rust
use crate::ui::theming::{AppTheme, ThemeMode};

let theme = AppTheme::new(ThemeMode::Dark);
let bg_color = theme.colors.surface_primary;
let text_color = theme.colors.text_primary;
```

#### 3. Color Helpers (`src/ui/theme.rs`)

Utility functions for viewer/editor-specific colors:

- `viewer_toolbar_background()` - Toolbar background
- `viewer_light_surface_color()` / `viewer_dark_surface_color()` - Canvas backgrounds
- `error_text_color()` - Error text styling
- `crop_overlay_*()` - Crop tool colors

#### 4. Component Styles (`src/ui/styles/`)

Ready-to-use style functions for Iced widgets:

| File | Purpose |
|------|---------|
| `button.rs` | `primary()`, `overlay()`, `disabled()`, `video_play_overlay()` |
| `container.rs` | `panel()` for settings/editor panels |
| `overlay.rs` | `indicator()`, `controls_container()`, icon styles |
| `editor.rs` | `toolbar()`, `settings_panel()` |

**Usage:**
```rust
use crate::ui::styles::button;

Button::new("Click me")
    .style(button::primary)
```

### Guidelines for Contributors

#### Adding a New Color

1. **Check if it exists** in `design_tokens::palette` first
2. **Add to palette** if it's a new base color:
   ```rust
   // In design_tokens.rs
   pub const SUCCESS_500: Color = Color::from_rgb(0.3, 0.7, 0.3);
   ```
3. **Add to ColorScheme** if it's semantic (used differently in light/dark):
   ```rust
   // In theming.rs ColorScheme
   pub success: Color,
   ```

#### Adding a New Component Style

1. Create a function in the appropriate `styles/*.rs` file
2. Use design tokens, not hard-coded values:
   ```rust
   // ✅ Good
   border: Border { radius: radius::SM.into(), .. }

   // ❌ Bad
   border: Border { radius: 4.0.into(), .. }
   ```
3. Add a test to verify the style compiles and uses expected tokens

#### Modifying Spacing or Sizing

1. **Check impact**: Tokens are used across many components
2. **Maintain ratios**: `spacing::MD` should equal `spacing::XS * 2`
3. **Run validation**: `cargo test` includes compile-time assertions

### Testing Styles

Integration tests in `tests/style_integration.rs` verify:
- All button styles compile correctly
- Design tokens are accessible
- Theme switching works (light ↔ dark)

Run style tests:
```bash
cargo test style_integration
```

## Getting Help

- Read the [README.md](README.md) for user documentation
- Check existing [issues](https://codeberg.org/Bawycle/iced_lens/issues)
- Open a new issue for questions or discussion

---

Thank you for contributing to IcedLens!
