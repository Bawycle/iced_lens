# TODO - Next Release

> This file tracks planned work for the next release. It lives only in the `dev` branch and is not included in releases.

## Bugs to Fix

- [x] ~~Fit-to-window doesn't expand media when hamburger menu collapses~~ ✅
  - Fixed with responsive widget for automatic layout detection
  - No more manual LayoutChanged events needed

- [x] ~~Navigation index not updated after media deletion~~ ✅
  - Fixed with MediaDeleted effect to sync media_navigator after deletion

- [x] ~~Video seeking with keyboard arrows broken when held down~~ ✅
  - Fixed with debouncing (check Seeking state before new seek)
  - Reduced default keyboard seek step to 2 seconds

- [ ] **[Intermittent]** Image horizontal offset after exiting fullscreen
  - Observed: vertical image with fit-to-window, enter fullscreen, exit fullscreen → image shifted horizontally
  - Not reliably reproducible (happened once, couldn't reproduce after restart)
  - Possible causes: race condition on window resize, stale viewport dimensions, window manager timing
  - If reproduced: note exact steps, window size, image used, timing between actions

- [x] ~~Image editor navigation should skip videos~~ ✅
  - Added `navigate_next_image()` and `navigate_previous_image()` methods to `MediaNavigator`
  - Editor navigation now automatically skips videos and finds the next/previous image
  - Handles looping navigation (wraps around if no more images in direction)
  - Removes need for "can't edit video" notification when navigating

## Planned Features

### Framework Upgrade
- [x] Upgrade to Iced 0.14.0 ✅
  - [x] Review breaking changes
  - [x] Update deprecated APIs
  - [x] Test all UI components after migration
  - [x] Fix video flickering (implemented VideoShader with persistent GPU texture)
  - Evaluate if Iced 0.14.0 facilitates:
    - [ ] Temporary rotation in viewer (90° increments, session-only) — currently complex to implement
    - [ ] Image centering in editor canvas — currently complex to implement

### Settings Persistence
- [x] ~~Persist video playback preferences in `settings.toml` (without adding to Settings screen)~~ ✅
  - [x] ~~`muted` state~~
  - [x] ~~`volume` level~~
  - [x] ~~`loop` toggle~~
  - Similar to how `fit_to_window` is handled for images
- [x] `keyboard_seek_step_secs` (added to Settings screen + persisted) ✅

### Image Editor Enhancements
- [x] ~~Add brightness adjustment tool~~ ✅
- [x] ~~Add contrast adjustment tool~~ ✅
  - Added `adjust_brightness` and `adjust_contrast` functions in `image_transform.rs`
  - Added "Light" tool in sidebar with sliders for brightness and contrast
  - Live preview during adjustment, full undo/redo support
- [x] ~~Remember last "Save As" directory~~ ✅
  - When opening the rfd save dialog, start in the last used save directory
  - Persist the path in CBOR format in the app data directory (use `app::paths::get_app_data_dir()`)
  - Applies to both image saves and video frame captures

### User Feedback & Notifications

Unify how the application communicates with the user. Currently:
- `eprintln!` is used throughout (~54 occurrences) — invisible to GUI users
- `ErrorDisplay` component exists but only for media loading errors in viewer
- No feedback for save operations, frame captures, config errors, etc.

**Goal:** Replace `eprintln!` with a proper notification system following UX best practices.

#### Phase 1: Toast/Snackbar Component ✅
- [x] Create `src/ui/notifications/` module
  - [x] `notification.rs` — `Notification` struct with severity (Success, Info, Warning, Error), message, timestamp, optional action
  - [x] `toast.rs` — Toast widget component (auto-dismiss, themed by severity)
  - [x] `manager.rs` — `NotificationManager` to queue and manage notifications
- [x] Add `notifications: NotificationManager` field to `App` struct
- [x] Add `Message::Notification(NotificationMessage)` variants (Push, Dismiss, Tick)
- [x] Render toast overlay in `app/view.rs` (positioned bottom-right)
- [x] Add i18n keys for common notifications (save success, save error, etc.)

#### Phase 2: Replace eprintln! with Notifications ✅
- [x] `app/mod.rs` — Save success/error, frame capture, editor errors
- [x] `app/update.rs` — Save success/error, delete success/error, editor errors
- [x] `app/persistence.rs` — Config save/load errors → warning notifications
- [x] `app/persisted_state.rs` — State save/load errors → warning notifications
- [x] `ui/image_editor/state/*.rs` — Removed debug eprintln! (silent early returns)
- [x] `ui/viewer/component.rs` — Video player errors (handled by ErrorDisplay, intentional)
- [x] `video_player/*.rs` — Decoder errors (handled by ErrorDisplay, intentional)
- [x] `i18n/fluent.rs` — FTL parsing errors (kept as eprintln!, developer/translator info)

#### Phase 3: Extend ErrorDisplay Usage ✅
- [x] Add i18n keys for generic `Error` variants (Io, Svg) — already exist in `error-load-image-*` keys
- [x] Document error handling strategy (see below)

#### Phase 4: Documentation ✅
- [x] Update `CONTRIBUTING.md` with notification system guidelines
  - Error handling strategy table
  - Code examples for adding notifications
  - Severity guidelines and best practices

#### Error Handling Strategy

| Type | Method | When to use |
|------|--------|-------------|
| **Toast Notification** | `notifications.push(Notification::*)` | User-initiated actions: save, delete, copy, config changes. Non-blocking feedback. |
| **ErrorDisplay** | `ErrorDisplay::new()` in view | Content loading failures (image, video). Contextual, shown where content would appear. Requires user acknowledgment. |
| **Silent (no output)** | Early return / `let else` | Recoverable internal errors where fallback behavior is acceptable. |
| **eprintln!** | `eprintln!()` | Developer/translator info only (FTL parsing errors). Never for user-facing issues. |

**Toast Duration:**
- Success/Info: 3 seconds (auto-dismiss)
- Warning: 5 seconds (auto-dismiss)
- Error: Manual dismiss only

**Max visible toasts:** 3 (others queued)

### Help Screen Enhancements ✅
- [x] ~~Display icons alongside button references in help text~~ ✅
  - Added `build_tool_item_with_icon()` and `build_bullet_with_icon()` helpers
  - Viewer: zoom, fit-to-window, fullscreen, delete icons
  - Video: play, volume, loop, step, capture icons
  - Editor: rotate left/right, flip horizontal/vertical icons
  - Uses `action_icons` module for consistency with UI

### Media Metadata ✅
- [x] ~~Retrieve metadata from media files (EXIF, video info, etc.)~~ ✅
  - Added `kamadak-exif` dependency for EXIF extraction
  - `src/media/metadata.rs` handles both image EXIF and video metadata
- [x] ~~Design UX/UI for displaying metadata~~ ✅
  - Info button in navbar (toggle with `I` shortcut)
  - Push layout in windowed mode, overlay in fullscreen
  - `src/ui/metadata_panel.rs` with collapsible sections

### Internationalization
- [x] ~~Add Spanish translation (`es.ftl`)~~ ✅
- [x] ~~Add German translation (`de.ftl`)~~ ✅
- [x] ~~Add Italian translation (`it.ftl`)~~ ✅

### AI Frame Enhancement (Exploratory)

Reduce motion blur on captured video frames using AI deblurring models.
See `docs/AI_DEBLURRING_MODELS.md` for detailed comparison.

**Phase 1: Proof of Concept**
- [ ] Evaluate NAFNet
  - Download ONNX model from [HuggingFace](https://huggingface.co/opencv/deblurring_nafnet)
  - Test with `ort` crate in standalone Rust binary
  - Measure inference time and quality on test images
- [ ] Evaluate Real-ESRGAN
  - Download ONNX model
  - Compare quality/speed with NAFNet
- [ ] Document findings and decide on model choice

**Phase 2: Integration (if Phase 1 successful)**
- [ ] Add model selection setting + model download / remove
- [ ] Add a way for the user to choose whether they want to capture the frame with or without AI enhancement.

## Refactoring

### Legacy Code Review
- [ ] Audit codebase for deprecated patterns and outdated code
  - Review old TODOs and FIXMEs in source files
  - Check for unused code paths
  - Update patterns to match current architecture
  - Ensure consistency with latest Iced 0.14 APIs

### UI Style Harmonization ✅
- [x] ~~Review and harmonize styles across all screens (viewer, editor, settings)~~ ✅
  - Added typography tokens (TITLE_LG/MD/SM, BODY_LG/BODY/BODY_SM, CAPTION)
  - Added border width tokens (WIDTH_SM, WIDTH_MD) and opacity::SURFACE
  - Replaced hardcoded button padding with spacing tokens
  - Aligned semantic colors with palette tokens in theming.rs
  - Created button::control_active style for toggle controls

### Configuration Improvements ✅
- [x] ~~Add sections to `settings.toml` for better organization~~ ✅
  - `[general]` for language and theme mode
  - `[display]` for fit_to_window, zoom_step, background_theme, sort_order
  - `[video]` for autoplay, volume, muted, loop, audio_normalization, caching settings
  - `[fullscreen]` for overlay_timeout_secs
  - Migration logic auto-converts old flat configs to new sectioned format on load

### Media Navigation Single Source of Truth
- [x] ~~Move navigation logic to `src/media/navigator.rs`~~ ✅
  - `MediaNavigator` (renamed from `ImageNavigator`) is now in `media/` module
- [x] ~~Remove dual source of truth between `viewer.image_list` and `app.media_navigator`~~ ✅
  - Added `NavigationInfo` struct to provide navigation state snapshot
  - `ViewEnv` now passes `NavigationInfo` from `MediaNavigator` to viewer
  - Removed `image_list` field from `viewer::component::State`
  - Moved deletion logic to App (handles file deletion via `media_navigator`)

### Path Injection for Testability ✅
- [x] ~~Refactor `AppState` to support path injection~~ ✅
  - Added `load_from(base_dir: Option<PathBuf>)` and `save_to(base_dir: Option<PathBuf>)` methods
  - `load()` and `save()` remain as convenience methods using default path
  - Support `ICED_LENS_DATA_DIR` environment variable override
  - Benefits:
    - Isolated tests (each test uses temp directory)
    - Portable mode (store data on USB drive)
    - CI/CD friendly (no interference between parallel tests)
- [x] ~~Apply same pattern to `config` module~~ ✅
  - Added `load_with_override(base_dir)` and `save_with_override(config, base_dir)` functions
  - Support `ICED_LENS_CONFIG_DIR` environment variable

### Project Structure Reorganization
- [x] ~~Move `src/config/` into `src/app/config/`~~ ✅
  - Config is app infrastructure, not business logic like `media/` or `video_player/`
- [x] ~~Move `src/i18n/` into `src/app/i18n/`~~ ✅
  - Translations are app-specific infrastructure
- [x] ~~Create `src/app/paths.rs` with `get_app_data_dir()`~~ ✅
  - Single source of truth for app data directory
  - Added `get_app_config_dir()` for config directory
  - Environment variable overrides: `ICED_LENS_DATA_DIR`, `ICED_LENS_CONFIG_DIR`
  - Uses `dirs::data_dir()` / `dirs::config_dir()` (cross-platform: Linux, macOS, Windows)
- [x] ~~Refactor `config/mod.rs` to use `app::paths` module~~ ✅
  - Removed duplicated `get_default_config_path()` function
  - Now uses centralized path resolution from `app::paths`
- [x] ~~Update `CONTRIBUTING.md` project structure section after refactoring~~ ✅

## Benchmarks

### To Add
- [ ] Media navigation benchmark (time to load next/previous image in directory)
  - Use existing test images in `tests/data/`

## Notes

- Target version: TBD (0.2.1 or 0.3.0 depending on scope)
- Test videos can be generated with `scripts/generate-test-videos.sh`
