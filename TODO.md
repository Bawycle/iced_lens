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

### Help
- [ ] Allow text selection and copying in the help screen (blocked, pending native support in Iced 0.15.0)

### Video Editor
- [ ] Create a simple video editor allowing users to trim videos by removing segments. The editor should let users play the video, seek to any position, step forward/backward frame by frame, and change the playback speed.

## UI / Design Experiments

- [ ] **Test light icons in toolbars**: Try using light theme icons in toolbar buttons instead of dark icons
  - Create a test branch to evaluate visual impact
  - Compare readability and contrast on both light and dark themes

## Code Quality / Refactoring

- [ ] **Generate PNG icons at build time**: Remove stored PNGs from repo, generate from SVGs during compilation
  - **Goal**: Reduce repository size, SVGs become single source of truth
  - **Approach**: Use `build.rs` with `resvg` (already a dependency, see `src/icon.rs`)
  - **Steps**:
    1. Create `build.rs` that reads SVGs from `assets/icons/source/`
    2. Render to PNGs using `resvg` + `tiny_skia` (32x32)
    3. For light theme: invert colors using `image` crate
    4. Save to `OUT_DIR` (e.g., `$OUT_DIR/icons/dark/`, `$OUT_DIR/icons/light/`)
    5. Update `src/ui/icons.rs`: use `include_bytes!(concat!(env!("OUT_DIR"), "/icons/..."))`
    6. Remove `assets/icons/png/` from repository
    7. Update `.gitignore` if needed and 'CONTRIBUTING.md'
  - **Reproducibility**: Fix `resvg` version, use deterministic PNG options (no metadata)
  - **Impact**: +10-15s first build (resvg compilation), then fast (incremental)
  - **Files to modify**: `build.rs` (new), `src/ui/icons.rs`, `Cargo.toml` (build-dependencies)

## Notes

- Test videos can be generated with `scripts/generate-test-videos.sh`
