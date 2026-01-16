# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Diagnostics system:** new screen (Settings menu → Diagnostics) to collect and export anonymized diagnostic data for troubleshooting.
  - Captures user actions, application state changes, and operation performance metrics
  - Privacy-first: all file paths are hashed (blake3), no sensitive data collected
  - Export to JSON file or clipboard for sharing with developers
  - Toggle collection on/off with status display
  - Help section explains what data is collected
- **Image prefetching:** adjacent images are preloaded in the background for faster navigation. Uses an LRU cache (32 MB default) to store 2 images ahead and 2 behind current position.
- **Metadata preservation options:** checkboxes in the editor sidebar to control metadata when saving.
  - Add software tag and modification date (default: on)
  - Strip GPS location data (shown only if image has GPS coordinates)
- **Processing metadata display:** software and modification date now shown in a new "Processing" section of the metadata panel.
- **Editable processing metadata:** software and modification date fields can now be edited in the metadata panel, like other metadata fields.

### Changed
- Directory scanning is now asynchronous, preventing UI freezes when opening folders with many files (especially with date-based sorting).
- AI model validation (for deblur and upscale) is now deferred until the user first enters the image editor. This avoids CPU-intensive operations at startup, especially when the user frequently opens and closes the application without using the editor.

### Fixed
- Video player no longer gets stuck after seeking. Previously, A/V sync could skip the target frame, leaving the player unresponsive.
- Edited images now preserve original metadata (EXIF, XMP, ICC) when saved. Previously, all metadata was lost.
- EXIF orientation is now automatically reset after rotation/flip transformations.
- Image editor now correctly saves files with uppercase extensions (`.JPG`, `.PNG`).

## [0.6.0] - 2025-01-02

### Added
- **Media filters:** filter navigation by media type (images/videos), orientation (landscape/portrait/square), and date range. Access via the filter dropdown in the toolbar. Filters can optionally persist across sessions (Settings → Display).
- **XMP metadata for PNG, WebP, and TIFF:** title, author, description, keywords, and copyright fields now work on PNG, WebP, and TIFF files, not just JPEG.
- **Temporary rotation in viewer:** rotate images 90° clockwise (`R`) or counter-clockwise (`Shift+R`). Rotation is session-only and resets when navigating to another image.
- **Flatpak packaging:** IcedLens can now be packaged as a Flatpak for easier installation on Linux.

### Changed
- Alphabetical sorting now matches your file manager: case-insensitive, numbers sorted naturally (`file2` before `file10`).
- Video playback errors are now displayed as notifications instead of failing silently.
- Audio controls (mute button, volume slider) are now disabled with a grayed appearance when media has no audio track (videos without audio, animated GIFs/WebPs). A tooltip explains "No audio track".
- Metadata fields are now filtered based on file format support—only relevant fields appear in the editor.
- Toolbar buttons reorganized for better UX: zoom and fit-to-window grouped together, rotation buttons after, fullscreen before delete (isolated as destructive action).

### Fixed
- Frame-by-frame video navigation now works immediately after loading a video.
- Video seeking no longer hangs on corrupted or incomplete files.
- Improved audio/video synchronization during playback.
- Saving metadata no longer shows false "success" for unsupported formats.
- Help screen expand/collapse icons no longer display with blue rectangles on Windows.
- Undo/redo now correctly preserves AI upscaling results in the image editor.
- Resize tool now works correctly after undoing an AI upscaling operation.
- Audio now plays correctly on surround sound systems (5.1, 7.1). Previously, audio on multi-channel devices could sound robotic or distorted.
- Audio now plays at correct speed for videos with non-standard sample rates (e.g., 32000 Hz). Previously, sample rate conversion produced distorted/robotic audio.
- Metadata panel now shows correct file info when opening a specific file via command line.
- Error notifications can now be dismissed by clicking the close button.

## [0.5.0] - 2025-12-22

### Added
- **Volume amplification:** volume slider now goes up to 150% for louder playback on quiet videos, with percentage display.
- **Video playback speed:** control video speed from 0.1x to 8x using `J`/`L` keys or overflow menu buttons.
- **Image editor zoom:** use mouse wheel to zoom in/out while editing, matching the viewer behavior.
- **Image editor pan:** grab-and-drag to navigate when zoomed, like in the viewer.
- **Resize preview:** thumbnail preview in the sidebar shows the result before applying.
- **AI upscaling:** optional Real-ESRGAN 4x upscaling for image enlargements. Enable in Settings → AI / Machine Learning. Produces sharper results than traditional interpolation for scales above 100%.
- **Auto-skip corrupted files:** when navigating, corrupted or unloadable files are automatically skipped. A notification lists skipped files. The maximum number of consecutive skips is configurable in Settings → Display (default: 5).

### Changed
- Crop handles are now larger for easier selection.
- Images are now centered in the editor canvas when smaller than the viewport.
- Toolbar buttons now use light (white) icons for better contrast on dark button backgrounds.
- Resize scale maximum increased from 200% to 400% to match Real-ESRGAN 4x native capability.
- Resize preset buttons now have uniform width for better visual consistency.
- Tooltips now have improved visibility with opaque background, shadow, and proper contrast adapting to light/dark theme.

### Fixed
- Navigation overlay buttons no longer display rendering artifacts on Windows.
- Window title now consistently shows the image title from metadata across viewer and editor.
- Resize tool now allows free dimension input when aspect ratio is unlocked.
- Resize slider no longer causes lag with large images.
- Video fit-to-window now displays correctly on drop and layout changes.
- Video navigation no longer causes frame distortion when switching between videos and images.

## [0.4.1] - 2025-12-19

### Fixed
- Navigation arrow clickable zones no longer span the entire viewport height.

## [0.4.0] - 2025-12-18

### Added
- **AI-powered image deblurring:** sharpen blurry photos using the NAFNet neural network (experimental). Enable in Settings → AI / Machine Learning. The model (~92 MB) downloads automatically on first use.
- **Metadata editing:** edit EXIF and Dublin Core/XMP metadata directly in the Info panel.
  - Dublin Core fields: title, creator, description, keywords, copyright
  - EXIF fields: camera info, date taken, exposure settings, GPS coordinates
  - Save to update the original file, or Save As to create a copy
- **Empty state view:** welcoming screen when no media is loaded, with "Open File" button and drag-and-drop support.
- **Directory as CLI argument:** open a folder to load the first media file automatically.
- **Dynamic window title:** shows filename, unsaved changes indicator (*), and document title from metadata.
- **Remember last directory:** file picker opens where you last loaded a file.
- **Application icons:** proper icons on Windows, macOS, and Linux.

### Changed
- Sidebar toggle buttons now use consistent double chevron icons (`<<`/`>>`).
- Metadata panel auto-collapses in fullscreen mode.
- Media loading errors now appear as toast notifications instead of blocking the view.
- Improved icon rendering consistency across platforms.

### Fixed
- Frame capture and step controls now work immediately after opening a video.
- Frame stepping no longer skips frames.
- Navigation arrows respond correctly during video playback.
- Frame history settings now save and apply correctly.

## [0.3.0] - 2025-12-14

### Added
- **Media info panel:** view technical metadata (EXIF for images, codec/bitrate for videos). Toggle with `I` key or Info button.
- **Brightness and contrast:** new light adjustment tool in the image editor.
- **Toast notifications:** visual feedback for save, delete, copy, and error operations.
- **Portable mode:** `--data-dir` and `--config-dir` CLI options to override default directories.
- **Keyboard seek step:** configurable time skip (0.5–30s) for arrow keys during video playback.
- **Video preferences:** volume, mute, and loop settings now persist across sessions.
- **Remember Save As directory:** file dialog opens in the last used save location.
- **New translations:** Spanish, German, and Italian.

### Changed
- Upgraded to Iced 0.14.0 with improved video rendering.
- Unified UI styling with consistent spacing, typography, and colors.
- Settings file now uses organized sections (`[general]`, `[display]`, `[video]`).
- Image editor automatically skips videos when navigating.

### Fixed
- Fit-to-window updates correctly when sidebar expands/collapses.
- Video seeking no longer snaps back when holding arrow keys.
- Navigation index updates correctly after deleting a file.
- System locale detection works with regional variants (e.g., `fr_FR` matches `fr`).

## [0.2.0] - 2025-12-12

### Added
- **Video playback:** play MP4, AVI, MOV, MKV, and WebM with controls for play/pause, seek, volume, mute, and loop.
- **Animated formats:** GIF and animated WebP play automatically with video controls.
- **Frame-by-frame navigation:** step forward/backward through video frames when paused.
- **Frame capture:** save the current video frame as an image.
- **Fullscreen video controls:** playback controls available in fullscreen mode.
- **Theme selection:** choose between System, Light, or Dark theme in Settings.

### Changed
- Edit button disabled for videos (image editing only).
- Minimum Rust version: 1.92.0.

### Fixed
- Keyboard shortcuts work correctly in fullscreen mode.
- Overlay controls visible on all background themes.

## [0.1.0] - 2025-12-02

### Added
- **Image viewing:** support for JPEG, PNG, GIF, WebP, TIFF, BMP, ICO, and SVG.
- **Zoom and pan:** mouse wheel zoom, fit-to-window toggle, drag to pan.
- **Navigation:** browse images in a folder with arrow keys or overlay buttons.
- **Fullscreen:** press F11, double-click, or use the toolbar button.
- **Background themes:** light, dark, or checkerboard.
- **Image editing:** rotate, crop (with aspect ratio presets), and resize with live preview.
- **Undo/redo:** unlimited history for all edits.
- **Save options:** Save to overwrite, Save As to create a copy.
- **Internationalization:** English and French, with runtime language switching.
- **Configuration:** settings saved automatically (language, zoom step, theme, sort order).

### Notes
- Pre-1.0 experimental release
- Tested on Linux Mint 22.2
- Licensed under MPL-2.0

[unreleased]: https://codeberg.org/Bawycle/iced_lens/compare/v0.6.0...HEAD
[0.6.0]: https://codeberg.org/Bawycle/iced_lens/compare/v0.5.0...v0.6.0
[0.5.0]: https://codeberg.org/Bawycle/iced_lens/compare/v0.4.1...v0.5.0
[0.4.1]: https://codeberg.org/Bawycle/iced_lens/compare/v0.4.0...v0.4.1
[0.4.0]: https://codeberg.org/Bawycle/iced_lens/compare/v0.3.0...v0.4.0
[0.3.0]: https://codeberg.org/Bawycle/iced_lens/compare/v0.2.0...v0.3.0
[0.2.0]: https://codeberg.org/Bawycle/iced_lens/releases/tag/v0.2.0
[0.1.0]: https://codeberg.org/Bawycle/iced_lens/releases/tag/v0.1.0
