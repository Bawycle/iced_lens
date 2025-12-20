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

#### Metadata Sidebar
- [ ] Allow text selection and copying in the metadata sidebar (blocked, pending native support in Iced 0.15.0)
- [ ] Add video metadata editing support (Phase 2 - future work)

### Image Editor
- [ ] AI-powered image upscaling using Real-ESRGAN when enlarging images (scale > 100%)
  - Optional feature like deblur: enable in Settings → AI / Machine Learning
  - Model downloaded on first use (~64 MB for x4plus)
  - Reuse existing ONNX infrastructure (ort, download with checksum verification)
  - Fixed scale factors (x2, x4) → combine with classic resize for intermediate values

### Help
- [ ] Allow text selection and copying in the help screen (blocked, pending native support in Iced 0.15.0)

### Video Player
- [x] Add new controls to the video player to allow changing the playback speed of the video.

### Video Editor
- [ ] Create a simple video editor allowing users to trim videos by removing segments. The editor should let users play the video, seek to any position, step forward/backward frame by frame, and change the playback speed.

## Code Quality / Refactoring

- [ ] **Newtype pattern audit**: Review codebase for values that should use newtypes for type-safety
  - Example: `PlaybackSpeed` newtype ensures valid range (0.1x - 8.0x) at the type level
  - Candidates to audit: zoom levels, volume, positions, durations, percentages
  - Benefits: compile-time guarantees, single source of truth, self-documenting code
  - Pattern: `struct Foo(f64)` with `new()` that validates/clamps and `value()` accessor
  - **Location check**: Domain types should live in their domain module (e.g., `PlaybackSpeed` in `video_player/`), not in config. Config holds constants, not types.

## Notes

- Test videos can be generated with `scripts/generate-test-videos.sh`
