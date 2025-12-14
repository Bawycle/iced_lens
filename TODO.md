# TODO - Next Release

> This file tracks planned work for the next release. It lives only in the `dev` branch and is not included in releases.

## Bugs to Fix

- [ ] **[Intermittent]** Image horizontal offset after exiting fullscreen
  - Observed: vertical image with fit-to-window, enter fullscreen, exit fullscreen → image shifted horizontally
  - Not reliably reproducible (happened once, couldn't reproduce after restart)
  - Possible causes: race condition on window resize, stale viewport dimensions, window manager timing
  - If reproduced: note exact steps, window size, image used, timing between actions

## Planned Features

### Viewer
- [ ] Temporary rotation in viewer (90° increments, session-only) — currently complex to implement

### Image editor
- [ ] Image centering in editor canvas — currently complex to implement

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

## Notes

- Test videos can be generated with `scripts/generate-test-videos.sh`
