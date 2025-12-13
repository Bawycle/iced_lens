# TODO - Next Release

> This file tracks planned work for the next release. It lives only in the `dev` branch and is not included in releases.

## Bugs to Fix

- [ ] Fit-to-window doesn't expand media when hamburger menu collapses
  - When the menu opens, images/videos correctly shrink to fit the reduced space
  - When the menu closes, media doesn't expand to fill the newly available space
  - Workaround: resize the window to trigger recalculation

- [ ] Navigation index not updated after media deletion
  - After deleting a media via toolbar button, the position counter stays correct
  - But navigating to next media jumps to media #1
  - And navigating to previous media jumps to the last media
  - Likely the navigation index is not properly synced after the directory rescan

- [ ] Video seeking with keyboard arrows broken when held down
  - Tapping left/right arrows works but seek step is too large
  - Holding down arrows continuously causes erratic behavior
  - Fix: ignore new seek events until current seek completes (debouncing)
  - Consider: reduce default keyboard seek step, add setting in Settings
    - Note: this is the keyboard shortcut seek step, not the slider step

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
- [ ] Persist video playback preferences in `settings.toml` (without adding to Settings screen)
  - [ ] `muted` state
  - [ ] `volume` level
  - [ ] `loop` toggle
  - Similar to how `fit_to_window` is handled for images

### Image Editor Enhancements
- [ ] Add brightness adjustment tool
- [ ] Add contrast adjustment tool
- [ ] Remember last "Save As" directory
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
