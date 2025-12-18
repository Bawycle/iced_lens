# TODO - Next Release

> This file tracks planned work for the next release. It lives only in the `dev` branch and is not included in releases.

## Bugs to Fix
- [ ] **[Intermittent]** Image horizontal offset after exiting fullscreen
  - Observed: vertical image with fit-to-window, enter fullscreen, exit fullscreen → image shifted horizontally
  - Not reliably reproducible (happened once, couldn't reproduce after restart)
  - Possible causes: race condition on window resize, stale viewport dimensions, window manager timing
  - If reproduced: note exact steps, window size, image used, timing between actions

## Modifications

### Refactoring

#### Event-Driven Architecture Improvements (Phase 2)

The codebase has several places where actions directly mutate state across domain boundaries instead of using event-driven patterns. This creates coupling and makes the code harder to maintain.

**Key issues to address:**

1. **Dual source of truth for `current_media_path`**
   - `media_navigator.current_media_path` = "what the user selected"
   - `viewer.current_media_path` = "what is displayed"
   - These are manually synchronized in 6+ places
   - **Solution:** Modify `MediaLoaded` message to include the path. Let the viewer set its own `current_media_path` only on successful load. Remove direct mutations from app handlers.

2. **Direct viewer state mutations from app**
   - `viewer.is_loading_media`, `viewer.loading_started_at` set directly in navigation/delete handlers
   - `viewer.set_zoom_step_percent()`, `viewer.set_video_autoplay()` called from settings handlers
   - **Solution:** Use messages instead of direct setters. Viewer should handle `StartLoadingMedia` message (already exists), settings changes should emit events that viewer subscribes to.

3. **Duplicate `video_autoplay` state**
   - Exists in both `App` and `viewer.State`
   - Manually synchronized on settings change
   - **Solution:** Single source of truth in config, viewer reads from passed context.

4. **Duplicated navigation logic**
   - `handle_editor_navigate_next/previous` vs `handle_navigate_next/previous`
   - Nearly identical code with subtle differences
   - **Solution:** Unify into single navigation handler with source parameter.

**Principles to follow:**
- Each domain owns its state and updates it via messages
- App orchestrates by handling effects and dispatching messages
- No direct cross-domain state mutations
- Single source of truth for each piece of data

## Planned Features

### Viewer
- [ ] Temporary rotation in viewer (90° increments, session-only) — currently complex to implement

#### Metadata Sidebar
- [ ] Allow text selection and copying in the metadata sidebar (blocked, pending native support in Iced 0.15.0)
- [x] Add the ability to edit and add new EXIF metadata for images from the sidebar (Phase 1 - images only)
- [ ] Add video metadata editing support (Phase 2 - future work)
- [ ] Add Dublin Core / XMP metadata support (Phase 2 - future work)

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
