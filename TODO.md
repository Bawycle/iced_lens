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
- [ ] Add brightness adjustment tool
- [ ] Add contrast adjustment tool
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

#### Phase 1: Toast/Snackbar Component
- [ ] Create `src/ui/notifications/` module
  - [ ] `notification.rs` — `Notification` struct with severity (Success, Info, Warning, Error), message, timestamp, optional action
  - [ ] `toast.rs` — Toast widget component (auto-dismiss, themed by severity)
  - [ ] `manager.rs` — `NotificationManager` to queue and manage notifications
- [ ] Add `notifications: NotificationManager` field to `App` struct
- [ ] Add `Message::Notification(NotificationMessage)` variants (Push, Dismiss, Tick)
- [ ] Render toast overlay in `app/view.rs` (positioned bottom-center or top-right)
- [ ] Add i18n keys for common notifications (save success, save error, etc.)

#### Phase 2: Replace eprintln! with Notifications
- [ ] `app/mod.rs` — Save success/error, frame capture, directory scan errors
- [ ] `app/update.rs` — Editor operations, navigation errors
- [ ] `app/persistence.rs` — Config save errors
- [ ] `app/persisted_state.rs` — State load/save errors (silent or log-only?)
- [ ] `ui/viewer/component.rs` — Video player errors
- [ ] `ui/image_editor/state/*.rs` — Crop, resize, history errors
- [ ] `video_player/*.rs` — Audio/video decoding errors (may need special handling)

#### Phase 3: Extend ErrorDisplay Usage
- [ ] Use `ErrorDisplay` for recoverable errors in editor (e.g., invalid crop region)
- [ ] Add i18n keys for generic `Error` variants (Io, Svg, Config) — currently only `VideoError` has them
- [ ] Consider inline error states vs modal errors vs toasts based on severity

#### Phase 4: Documentation
- [ ] Update `CONTRIBUTING.md` with notification system guidelines
  - When to use toasts vs ErrorDisplay vs silent logging
  - How to push notifications from message handlers
  - i18n requirements for notification messages
  - Code examples for common use cases

#### Design Considerations
- Toast duration: ~3s for success/info, ~5s for warnings, manual dismiss for errors
- Max visible toasts: 3 (queue others)
- Animation: fade in/out (if Iced supports, else instant)
- Accessibility: ensure sufficient contrast, screen reader support

### Media Metadata
- [ ] Retrieve metadata from media files (EXIF, video info, etc.)
- [ ] Design UX/UI for displaying metadata (TBD - needs exploration)
  - Options to consider: side panel, overlay, modal dialog, info button

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

### UI Style Harmonization
- [ ] Review and harmonize styles across all screens (viewer, editor, settings)
  - Ensure consistent button styles, spacing, and colors
  - Improve overall UX and aesthetics
  - Study current UI/UX best practices (Material Design, Human Interface Guidelines, etc.)

### Configuration Improvements
- [ ] Add sections to `settings.toml` for better organization
  - `[general]` for language, theme
  - `[display]` for background, zoom, sort order
  - `[video]` for autoplay, volume, muted, seek step, cache settings
  - `[fullscreen]` for overlay timeout
  - Requires migration logic for existing config files

### Media Navigation Single Source of Truth
- [x] ~~Move navigation logic to `src/media/navigator.rs`~~ ✅
  - `MediaNavigator` (renamed from `ImageNavigator`) is now in `media/` module
- [x] ~~Remove dual source of truth between `viewer.image_list` and `app.media_navigator`~~ ✅
  - Added `NavigationInfo` struct to provide navigation state snapshot
  - `ViewEnv` now passes `NavigationInfo` from `MediaNavigator` to viewer
  - Removed `image_list` field from `viewer::component::State`
  - Moved deletion logic to App (handles file deletion via `media_navigator`)

### Path Injection for Testability
- [ ] Refactor `AppState` to support path injection
  - Add `load_from(path: Option<PathBuf>)` method for custom paths
  - Keep `load()` as convenience method using default path
  - Support `ICED_LENS_DATA_DIR` environment variable override
  - Benefits:
    - Isolated tests (each test uses temp directory)
    - Portable mode (store data on USB drive)
    - CI/CD friendly (no interference between parallel tests)
- [ ] Apply same pattern to `config` module
  - `Config::load_from(path)` + `Config::load()` convenience method
  - Support `ICED_LENS_CONFIG_DIR` environment variable

### Project Structure Reorganization
- [ ] Move `src/config/` into `src/app/config/`
  - Config is app infrastructure, not business logic like `media/` or `video_player/`
- [ ] Move `src/i18n/` into `src/app/i18n/`
  - Translations are app-specific infrastructure
- [x] ~~Create `src/app/paths.rs` with `get_app_data_dir()`~~ ✅
  - Single source of truth for app data directory
  - Uses `dirs::data_dir()` (cross-platform: Linux, macOS, Windows)
- [ ] Refactor `config/mod.rs` to use `app::paths` module
  - Currently config has its own `get_default_config_path()` function
- [ ] Update `CONTRIBUTING.md` project structure section after refactoring

## Benchmarks

### To Add
- [ ] Media navigation benchmark (time to load next/previous image in directory)
  - Use existing test images in `tests/data/`

## Notes

- Target version: TBD (0.2.1 or 0.3.0 depending on scope)
- Test videos can be generated with `scripts/generate-test-videos.sh`
