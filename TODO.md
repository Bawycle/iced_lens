# TODO - Next Release

> This file tracks planned work for the next release. It lives only in the `dev` branch and is not included in releases.

## Bugs to Fix


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

### Video Player
- [ ] Allow volume amplification above 100% (like VLC)
  - Extend `Volume` newtype max from 1.0 to ~1.5 or 2.0
  - Audio samples are already multiplied in `audio_output.rs`, just needs higher factor
  - Clipping is acceptable (no hardware risk, just distortion)
  - UI: visually distinguish the > 100% zone on the slider

### Help
- [ ] Allow text selection and copying in the help screen (blocked, pending native support in Iced 0.15.0)

### Video Editor
- [ ] Create a simple video editor allowing users to trim videos by removing segments. The editor should let users play the video, seek to any position, step forward/backward frame by frame, and change the playback speed.

## Code Quality / Refactoring

## Notes

- Test videos can be generated with `scripts/generate-test-videos.sh`
