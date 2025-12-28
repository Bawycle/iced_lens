# TODO - Next Release

> This file tracks planned work for the next release. It lives only in the `dev` branch and is not included in releases.

## Bugs to Fix

_(No known bugs)_

## Planned Features

### Viewer
- [ ] Temporary rotation in viewer (90° increments, session-only) — currently complex to implement

#### Media Filters for Navigation
- [ ] Add filterable navigation to show only matching media in the current directory

**Filter categories to consider:**

| Category | Filters | Notes |
|----------|---------|-------|
| **Media type** | Images only, Videos only, All | Basic, high priority |
| **Format** | JPEG, PNG, GIF, WebP, MP4, MKV... | Useful for batch processing |
| **Aspect ratio** | Landscape, Portrait, Square | Common photography need |
| **Animation** | Animated (GIF/WebP/video), Static | Distinguish animated content |
| **Audio** | Videos with audio, Silent videos | Filter silent screen recordings |
| **Resolution** | By megapixels ranges, 4K+, HD, SD | For quality filtering |
| **File size** | Small (<1MB), Medium, Large (>10MB) | Storage management |
| **Date** | Today, This week, This month, Custom range | Time-based browsing |
| **Metadata** | Has EXIF, Has GPS, Has title/description | Find tagged content |
| **Camera** | By make/model (from EXIF) | Organize by device |

**UX considerations:**
- Filter UI: dropdown in navbar? sidebar panel? keyboard shortcut?
- Show filter status indicator (e.g., "Showing 42/156 images")
- Remember last filter per directory? Or global preference?
- Quick toggle vs advanced filter panel

**Image editor interaction:**
- When filters are active and user enters editor, what happens on navigation?
  - Option A: Editor ignores filters, navigates all images (may be confusing)
  - Option B: Editor respects filters (consistent but may limit access)
  - Option C: Editor auto-filters to "images only" regardless of viewer filter
- Need to handle case where filter excludes all images (disable Edit button?)

#### Metadata Sidebar
- [ ] Allow text selection and copying in the metadata sidebar (blocked, pending native support in Iced 0.15.0)
- [ ] Add video metadata editing support (Phase 2 - future work)

### Video Player

_(No pending items)_

### Help
- [ ] Allow text selection and copying in the help screen (blocked, pending native support in Iced 0.15.0)

### Video Editor
- [ ] Create a simple video editor allowing users to trim videos by removing segments. The editor should let users play the video, seek to any position, step forward/backward frame by frame, and change the playback speed.

## Code Quality / Refactoring

## Packaging / Distribution

### Flatpak
- [ ] Create a Flatpak build script (`scripts/build-flatpak.sh`)
- [ ] Write the Flatpak manifest
- [ ] Add required metadata for Flathub:
  - [ ] AppStream metadata file (`metainfo.xml`) with app description, screenshots, release notes
  - [ ] Desktop entry file (`.desktop`)
  - [ ] App icons in required sizes (64x64, 128x128, 256x256 PNG + scalable SVG)
- [ ] Test Flatpak build locally with `flatpak-builder`
- [ ] Prepare Flathub submission:
  - [ ] Fork [flathub/flathub](https://github.com/flathub/flathub)
  - [ ] Create PR with manifest following [Flathub submission guidelines](https://docs.flathub.org/docs/for-app-authors/submission/)
  - [ ] Ensure app passes Flathub quality guidelines (no network at build time, proper permissions, etc.)

## Notes

- Test videos can be generated with `scripts/generate-test-videos.sh`
