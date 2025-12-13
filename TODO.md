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
- [ ] Remove dual source of truth between `viewer.image_list` and `app.media_navigator`
  - Currently both maintain their own copy of the media list
  - Goal: `MediaNavigator` in App is the ONLY source of truth
  - Viewer should receive navigation info via props/context, not maintain its own list
  - Remove `image_list` field from `viewer::component::State`

### Project Structure Reorganization
- [ ] Move `src/config/` into `src/app/config/`
  - Config is app infrastructure, not business logic like `media/` or `video_player/`
- [ ] Move `src/i18n/` into `src/app/i18n/`
  - Translations are app-specific infrastructure
- [ ] Create `src/app/paths.rs` with `get_app_data_dir()`
  - Single source of truth for app data directory
  - Uses `dirs::config_dir()` (cross-platform: Linux, macOS, Windows)
  - Refactor `config/mod.rs` to use this module
- [ ] Update `CONTRIBUTING.md` project structure section after refactoring

## Benchmarks

### To Add
- [ ] Media navigation benchmark (time to load next/previous image in directory)
  - Use existing test images in `tests/data/`

## Notes

- Target version: TBD (0.2.1 or 0.3.0 depending on scope)
- Test videos can be generated with `scripts/generate-test-videos.sh`
