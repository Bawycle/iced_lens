# Application name term - single source of truth for branding
-app-name = IcedLens

window-title = { -app-name }
settings-back-to-viewer-button = Back to Viewer
settings-title = Settings
settings-section-general = General
settings-section-display = Display
settings-section-video = Video
settings-section-fullscreen = Fullscreen
select-language-label = Select Language:
language-name-en-US = English
language-name-fr = French
language-name-es = Spanish
language-name-de = German
language-name-it = Italian
error-load-image-heading = We couldn't open the image.
error-details-show = Show details
error-details-hide = Hide details
error-details-technical-heading = Technical details
viewer-zoom-label = Zoom
viewer-zoom-input-placeholder = 100
viewer-zoom-reset-button = Reset
viewer-fit-to-window-toggle = Fit to window
viewer-zoom-input-error-invalid = Please enter a valid number.
viewer-zoom-step-error-invalid = The zoom step must be a number.
viewer-zoom-step-error-range = The zoom step must be between 1% and 200%.
viewer-delete-tooltip = Delete the current image
viewer-zoom-in-tooltip = Zoom in
viewer-zoom-out-tooltip = Zoom out
viewer-fullscreen-tooltip = Toggle fullscreen
viewer-double-click = Double-click
viewer-scroll-wheel = Scroll wheel
viewer-click-drag = Click + drag
settings-zoom-step-label = Zoom step
settings-zoom-step-placeholder = 10
settings-zoom-step-hint = Choose how much the zoom changes when using the controls (1% to 200%).
settings-background-label = Viewer background
settings-background-light = Light
settings-background-dark = Dark
settings-background-checkerboard = Checkerboard
settings-theme-mode-label = Application theme
settings-theme-system = Match system
settings-theme-light = Light
settings-theme-dark = Dark
help-usage-heading = USAGE:
help-options-heading = OPTIONS:
help-args-heading = ARGS:
help-examples-heading = EXAMPLES:
help-line-option-help = -h, --help        Show this help text
help-line-option-lang =     --lang <id>    Set locale (e.g. en-US, fr)
help-arg-image-path = <PATH>      Path to a media file or directory to open
help-example-1 = iced_lens ./photo.png
help-example-2 = iced_lens ./my_photos/
help-example-3 = iced_lens --lang fr ./image.jpg
help-description = { -app-name } – Image Viewer
help-line-option-i18n-dir =     --i18n-dir <path>  Load translations from directory
help-line-option-data-dir =     --data-dir <path>  Override data directory (state files)
help-line-option-config-dir =     --config-dir <path>  Override config directory (settings.toml)
settings-sort-order-label = Image navigation sort order
settings-sort-alphabetical = Alphabetical
settings-sort-modified = Modified date
settings-sort-created = Created date
settings-overlay-timeout-label = Fullscreen overlay auto-hide delay
settings-overlay-timeout-hint = Time before controls disappear when in fullscreen mode.
seconds = seconds
image-editor-title = Image Editor
image-editor-back-to-viewer = Back to Viewer
image-editor-cancel = Cancel
image-editor-save = Save
image-editor-save-as = Save As...
image-editor-tool-crop = Crop
image-editor-tool-resize = Resize
image-editor-tool-light = Light
image-editor-rotate-section-title = Rotation
image-editor-rotate-right-tooltip = Rotate image clockwise
image-editor-rotate-left-tooltip = Rotate image counter-clockwise
image-editor-flip-section-title = Flip
image-editor-flip-horizontal-tooltip = Flip image horizontally (mirror left-right)
image-editor-flip-vertical-tooltip = Flip image vertically (mirror top-bottom)
image-editor-resize-section-title = Resize
image-editor-resize-scale-label = Scale
image-editor-resize-dimensions-label = Target size
image-editor-resize-width-label = Width (px)
image-editor-resize-height-label = Height (px)
image-editor-resize-lock-aspect = Lock aspect ratio
image-editor-resize-presets-label = Presets
image-editor-resize-apply = Apply resize
image-editor-light-section-title = Light Adjustments
image-editor-light-brightness-label = Brightness
image-editor-light-contrast-label = Contrast
image-editor-light-reset = Reset
image-editor-light-apply = Apply
image-editor-crop-section-title = Crop
image-editor-crop-ratio-label = Aspect ratio
image-editor-crop-ratio-free = Free
image-editor-crop-ratio-square = Square (1:1)
image-editor-crop-ratio-landscape = Landscape (16:9)
image-editor-crop-ratio-portrait = Portrait (9:16)
image-editor-crop-ratio-photo = Photo (4:3)
image-editor-crop-ratio-photo-portrait = Photo Portrait (3:4)
image-editor-crop-apply = Apply crop
image-editor-undo-redo-section-title = Last modification
image-editor-undo = Undo
image-editor-redo = Redo
image-editor-export-format-label = Export format
media-loading = Loading...
settings-video-autoplay-label = Video autoplay
settings-video-autoplay-enabled = Enabled
settings-video-autoplay-disabled = Disabled
settings-video-autoplay-hint = When enabled, videos start playing automatically when opened.
video-play-tooltip = Play (Space)
video-pause-tooltip = Pause (Space)
video-mute-tooltip = Mute (M)
video-unmute-tooltip = Unmute (M)
video-loop-tooltip = Loop
video-capture-tooltip = Capture current frame
video-step-forward-tooltip = Step forward one frame (.)
video-step-backward-tooltip = Step backward one frame (,)
video-more-tooltip = More options
settings-audio-normalization-label = Audio volume normalization
settings-audio-normalization-enabled = Enabled
settings-audio-normalization-disabled = Disabled
settings-audio-normalization-hint = Automatically levels audio volume between different media files to prevent sudden volume changes.
settings-frame-cache-label = Keyframe cache size (for seeking)
settings-frame-cache-hint = Caches video keyframes to speed up timeline scrubbing and jumping to specific times. Higher values store more keyframes for faster seeking. Changes apply when opening a new video.
settings-frame-history-label = Frame history size (for stepping back)
settings-frame-history-hint = Stores recently displayed frames to allow stepping backward frame-by-frame. Only used when manually stepping through frames, not during normal playback.
settings-keyboard-seek-step-label = Keyboard seek step
settings-keyboard-seek-step-hint = Time to skip when using arrow keys during video playback.
megabytes = MB
error-load-video-heading = We couldn't play this video.
error-load-video-general = Something went wrong while loading the video.
error-load-video-unsupported-format = This file format is not supported.
error-load-video-unsupported-codec = The video codec '{ $codec }' is not supported on this system.
error-load-video-corrupted = The video file appears to be corrupted or invalid.
error-load-video-no-video-stream = No video track was found in this file.
error-load-video-decoding-failed = Video decoding failed: { $message }
error-load-video-io = We couldn't read this file. Check that it still exists and that you have permission to open it.

# Navigation bar
menu-settings = Settings
menu-help = Help
menu-about = About
navbar-edit-button = Edit

# Help screen
help-title = Help
help-back-to-viewer-button = Back to Viewer

# Common labels
help-tools-title = Available Tools
help-shortcuts-title = Keyboard Shortcuts
help-usage-title = How to Use

# ─────────────────────────────────────────────────────────────────────────────
# Viewer Section
# ─────────────────────────────────────────────────────────────────────────────
help-section-viewer = Image and Video Viewer
help-viewer-role = Browse and view your images and videos. Navigate through files in the same folder and adjust the display to your preferences.

help-viewer-tool-navigation = Navigation
help-viewer-tool-navigation-desc = Use arrow buttons or keyboard to move between files in the folder.
help-viewer-tool-zoom = Zoom
help-viewer-tool-zoom-desc = Scroll with mouse wheel, use +/- buttons, or enter a percentage directly.
help-viewer-tool-pan = Pan
help-viewer-tool-pan-desc = When zoomed in, click and drag the image to move around.
help-viewer-tool-fit = Fit to window
help-viewer-tool-fit-desc = Automatically scales the image to fit entirely within the window.
help-viewer-tool-fullscreen = Fullscreen
help-viewer-tool-fullscreen-desc = Immersive view with auto-hiding controls (delay configurable in Settings).
help-viewer-tool-delete = Delete
help-viewer-tool-delete-desc = Permanently remove the current file (moves to system trash if available).

help-viewer-key-navigate = Move to previous/next file
help-viewer-key-edit = Open image in editor
help-viewer-key-fullscreen = Enter/exit fullscreen
help-viewer-key-exit-fullscreen = Exit fullscreen mode
help-viewer-key-info = Toggle file information panel

help-mouse-title = Mouse Interactions
help-viewer-mouse-doubleclick = Double-click on image/video to toggle fullscreen
help-viewer-mouse-wheel = Zoom in/out
help-viewer-mouse-drag = Pan image when zoomed in

# ─────────────────────────────────────────────────────────────────────────────
# Video Playback Section
# ─────────────────────────────────────────────────────────────────────────────
help-section-video = Video Playback
help-video-role = Play video files with full playback controls. Adjust volume, seek through the timeline, and navigate frame by frame for precise positioning.

help-video-tool-playback = Play/Pause
help-video-tool-playback-desc = Start or stop video playback with the play button or Space key.
help-video-tool-timeline = Timeline
help-video-tool-timeline-desc = Click anywhere on the progress bar to jump to that position.
help-video-tool-volume = Volume
help-video-tool-volume-desc = Drag the volume slider or click the speaker icon to mute/unmute.
help-video-tool-loop = Loop
help-video-tool-loop-desc = Enable to automatically restart the video when it ends.
help-video-tool-stepping = Frame stepping
help-video-tool-stepping-desc = When paused, move forward or backward one frame at a time for precise navigation.
help-video-tool-capture = Frame capture
help-video-tool-capture-desc = Save the current video frame as an image file (opens in editor).

help-video-key-playpause = Play or pause the video
help-video-key-mute = Toggle audio mute
help-video-key-seek = Seek backward/forward (during playback)
help-video-key-volume = Increase/decrease volume
help-video-key-step-back = Step backward one frame (when paused)
help-video-key-step-forward = Step forward one frame (when paused)

# ─────────────────────────────────────────────────────────────────────────────
# Image Editor Section
# ─────────────────────────────────────────────────────────────────────────────
help-section-editor = Image Editor
help-editor-role = Make adjustments to your images: rotate, crop to a specific area, or resize to different dimensions.
help-editor-workflow = All changes are non-destructive until you save. Use "Save" to overwrite the original, or "Save As" to create a new file and preserve the original.

help-editor-rotate-title = Rotation
help-editor-rotate-desc = Rotate or flip the image to correct orientation or create mirror effects.
help-editor-rotate-left = Rotate 90° counter-clockwise
help-editor-rotate-right = Rotate 90° clockwise
help-editor-flip-h = Flip horizontally (mirror left/right)
help-editor-flip-v = Flip vertically (mirror top/bottom)

help-editor-crop-title = Crop
help-editor-crop-desc = Remove unwanted areas by selecting the region you want to keep.
help-editor-crop-ratios = Choose a preset ratio (1:1 square, 16:9 landscape, 9:16 portrait, 4:3 or 3:4 photo) or crop freely.
help-editor-crop-usage = Drag the handles to adjust the selection, then click "Apply" to confirm.

help-editor-resize-title = Resize
help-editor-resize-desc = Change the image dimensions to make it larger or smaller.
help-editor-resize-scale = Scale by percentage (e.g., 50% to halve the size)
help-editor-resize-dimensions = Enter exact width and height in pixels
help-editor-resize-lock = Lock aspect ratio to maintain proportions
help-editor-resize-presets = Use presets for common sizes (HD, Full HD, 4K...)

help-editor-light-title = Light
help-editor-light-desc = Fine-tune the brightness and contrast of your image.
help-editor-light-brightness = Brightness: lighten or darken the overall image
help-editor-light-contrast = Contrast: increase or decrease the difference between light and dark areas
help-editor-light-preview = Changes are previewed in real-time before applying

help-editor-save-title = Saving
help-editor-save-overwrite = Save: overwrites the original file
help-editor-save-as = Save As: creates a new file (choose location and format)

help-editor-key-save = Save current changes
help-editor-key-undo = Undo last change
help-editor-key-redo = Redo undone change
help-editor-key-cancel = Cancel all changes and exit

# ─────────────────────────────────────────────────────────────────────────────
# Frame Capture Section
# ─────────────────────────────────────────────────────────────────────────────
help-section-capture = Video Frame Capture
help-capture-role = Extract any frame from a video and save it as an image file. Perfect for creating thumbnails or capturing specific moments.

help-capture-step1 = Play or navigate the video to the desired frame
help-capture-step2 = Pause the video (use frame stepping for precision)
help-capture-step3 = Click the camera button in the video controls
help-capture-step4 = The frame opens in the editor — save as PNG, JPEG, or WebP

help-capture-formats = Supported export formats: PNG (lossless), JPEG (smaller file size), WebP (modern format with good compression).

# About screen
about-title = About
about-back-to-viewer-button = Back to Viewer

about-section-app = Application
about-app-name = { -app-name }
about-app-description = Lightweight image and video viewer with basic image editing.

about-section-license = License
about-license-name = Mozilla Public License 2.0 (MPL-2.0)
about-license-summary = File-level copyleft: modified files must be shared under the same license. Compatible with proprietary code.

about-section-icon-license = Icon License
about-icon-license-name = { -app-name } Icon License
about-icon-license-summary = All icons (application logo and UI icons) may only be redistributed unmodified to represent { -app-name }.

about-section-credits = Credits
about-credits-iced = Built with the Iced GUI toolkit
about-credits-ffmpeg = Video playback powered by FFmpeg
about-credits-fluent = Internationalization by Project Fluent

about-section-links = Links
about-link-repository = Source Code
about-link-issues = Report Issues

# Notifications
notification-save-success = Image saved successfully
notification-save-error = Failed to save image
notification-frame-capture-success = Frame captured successfully
notification-frame-capture-error = Failed to capture frame
notification-delete-success = File deleted successfully
notification-delete-error = Failed to delete file
notification-config-save-error = Failed to save settings
notification-config-load-error = Failed to load settings, using defaults
notification-state-parse-error = Failed to read app state, using defaults
notification-state-read-error = Failed to open app state file
notification-state-path-error = Cannot determine app data path
notification-state-dir-error = Failed to create app data directory
notification-state-write-error = Failed to save app state
notification-state-create-error = Failed to create app state file
notification-scan-dir-error = Failed to scan directory
notification-editor-frame-error = Failed to open editor with captured frame
notification-editor-create-error = Failed to open image editor
notification-editor-load-error = Failed to load image for editing
notification-video-editing-unsupported = Video editing is not supported yet

# Metadata panel
metadata-panel-title = File Information
metadata-section-file = File
metadata-section-camera = Camera
metadata-section-exposure = Exposure
metadata-section-video = Video
metadata-section-audio = Audio
metadata-section-gps = Location
metadata-label-dimensions = Dimensions
metadata-label-file-size = File size
metadata-label-format = Format
metadata-label-date-taken = Date taken
metadata-label-camera = Camera
metadata-label-exposure = Exposure
metadata-label-aperture = Aperture
metadata-label-iso = ISO
metadata-label-focal-length = Focal length
metadata-label-gps = Coordinates
metadata-label-codec = Codec
metadata-label-bitrate = Bit rate
metadata-label-duration = Duration
metadata-label-fps = Frame rate
metadata-value-unknown = Unknown
navbar-info-button = Info

# Empty state (no media loaded)
empty-state-title = No media loaded
empty-state-subtitle = Drop files here or click to open
empty-state-button = Open File
empty-state-drop-hint = Drag and drop images or videos anywhere

# Additional notifications
notification-empty-dir = No supported media files found in this folder
notification-load-error-io = Could not open file. Check that it exists and you have permission.
notification-load-error-svg = Could not render SVG. The file may be malformed.
notification-load-error-video = Could not play video. The format may be unsupported.
notification-load-error-timeout = Loading timed out. The file may be too large or the system is busy.
