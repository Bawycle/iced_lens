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
- [x] Add the ability to edit and add new EXIF metadata for images from the sidebar (Phase 1 - images only)
- [ ] Add video metadata editing support (Phase 2 - future work)
- [x] Add Dublin Core / XMP metadata support for images (Title, Creator, Description, Keywords, Copyright)

### Image editor
- [ ] Image centering in editor canvas — currently complex to implement
- [x] Add an image deblurring tool based on the NAFNet AI model (implemented in v0.4.0)

### Help
- [ ] Allow text selection and copying in the help screen (blocked, pending native support in Iced 0.15.0)

### Video Player
- [ ] Add new controls to the video player to allow changing the playback speed of the video.

### Video Editor
- [ ] Create a simple video editor allowing users to trim videos by removing segments. The editor should let users play the video, seek to any position, step forward/backward frame by frame, and change the playback speed.

## Notes

- Test videos can be generated with `scripts/generate-test-videos.sh`
