# TODO - Next Release

> This file tracks planned work for the next release. It lives only in the `dev` branch and is not included in releases.

## Bugs to Fix
- [ ] **[Intermittent]** Image horizontal offset after exiting fullscreen
  - Observed: vertical image with fit-to-window, enter fullscreen, exit fullscreen → image shifted horizontally
  - Not reliably reproducible (happened once, couldn't reproduce after restart)
  - Possible causes: race condition on window resize, stale viewport dimensions, window manager timing
  - If reproduced: note exact steps, window size, image used, timing between actions

## Modifications
- [ ] Automatically collapse the metadata sidebar when entering fullscreen mode, if it is currently expanded.
- [ ] Migrate from SVG icons to PNG icons for better performance (lighter, more universal rendering). Icons should be loaded into memory at application startup or upon first composition, not within components, to prevent repeated disk access on re-renders.

### Refactoring

## Planned Features

### Core
- [ ] Support providing a directory path as a command-line argument.
  - When a directory path is given, the application should scan it for media files.
  - The first supported media file (based on current sorting settings) will be opened.
  - If no supported media is found, the application will start as if no argument was provided.

### UX/UI
- [ ] Implement an "empty state" view for the main screen when no media is loaded:
  - This view is shown by a component that replaces the media viewer if the app is launched without a path or if the path is invalid.
  - The component area should be a drop zone for media files or folders.
  - It should feature a central button to open a file or folder via the system dialog (`rfd` crate).
  - All UI controls irrelevant in this state (e.g., zoom, navigation, metadata) must be disabled.
  - Error handling for failed loads must use the existing notification system.
- [ ] Add a collapse control directly on the metadata sidebar. This provides a more intuitive way for users to hide the sidebar than re-clicking the info button.
- [ ] Implement dynamic window title display:
  - Use "Media Title - AppName" if a title is available in the media's metadata.
  - Fallback to "FileName.ext - AppName" if no title metadata is found.
  - Just "AppName" if no media loaded.
  - Prefix with an asterisk (*Media Title/FileName.ext - AppName) if the file has unsaved modifications.

### Viewer
- [ ] Temporary rotation in viewer (90° increments, session-only) — currently complex to implement

#### Metadata Sidebar
- [ ] Allow text selection and copying in the metadata sidebar (blocked, pending native support in Iced 0.15.0)
- [ ] Add the ability to edit and add new media metadata from the sidebar (in accordance with common standards such as Exif, Dublin Core, etc.). The help screen must be updated accordingly.

### Image editor
- [ ] Image centering in editor canvas — currently complex to implement
- [ ] Add an image deblurring tool based on the Nafnet AI model
  - The tool will be optional to avoid bundling the large ONNX model file in the binary.
  - The download URL for the ONNX model must be configurable in the settings. The application will provide a default URL already configured.
  - In the settings, the user can enable/disable the option:
    - If enabled:
      - the model will be automatically downloaded into the "data directory"
      - the model's checksum will be verified
      - the application will check that deblurring is functional
      - if all checks pass, the option is enabled and the tool becomes available in the image editor
      - the user is informed of each step via the notification system
    - If disabled:
      - the option is disabled, the tool is no longer available, and the ONNX file is deleted
  - In the image editor, the deblurring tool must:
    - display a message advising to export only in Webp lossless or PNG for optimal deblurring quality
    - elegantly indicate that deblurring is in progress (e.g., spinner)
    - allow the user to cancel the ongoing deblurring
    - use the notification system to inform the user of success or failure
  - The UI for the deblurring tool must be in a dedicated file (like crop_panel.rs, resize_panel.rs, adjustments_panel.rs)
  - The help screen must be updated
  - The business logic must be in image_transform.rs or a dedicated file in src/media/
  - Use the "experiment/ai-deblur-nafnet" branch as inspiration for model use
  - Provide an option for GPU usage if available (configurable in settings)

### Help
- [ ] Allow text selection and copying in the help screen (blocked, pending native support in Iced 0.15.0)

### Video Player
- [ ] Add new controls to the video player to allow changing the playback speed of the video.

### Video Editor
- [ ] Create a simple video editor allowing users to trim videos by removing segments. The editor should let users play the video, seek to any position, step forward/backward frame by frame, and change the playback speed.

## Notes

- Test videos can be generated with `scripts/generate-test-videos.sh`
