# TODO - Next Release

> This file tracks planned work for the next release. It lives only in the `dev` branch and is not included in releases.

## Planned Features

### Framework Upgrade
- [ ] Upgrade to Iced 0.14.0
  - Review breaking changes
  - Update deprecated APIs
  - Test all UI components after migration
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

## Benchmarks

### To Add
- [ ] Media navigation benchmark (time to load next/previous image in directory)
  - Use existing test images in `tests/data/`

## Notes

- Target version: TBD (0.2.1 or 0.3.0 depending on scope)
- Test videos can be generated with `scripts/generate-test-videos.sh`
