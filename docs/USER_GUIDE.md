# IcedLens User Guide

Complete documentation for IcedLens features, configuration, and usage.

## Table of Contents

1. [Installation](#installation)
2. [Command-Line Interface](#command-line-interface)
3. [Keyboard Shortcuts](#keyboard-shortcuts)
4. [Navigation & Viewing](#navigation--viewing)
5. [Editing Tools](#editing-tools)
6. [AI Deblur](#ai-deblur)
7. [AI Upscaling](#ai-upscaling)
8. [Metadata Editing](#metadata-editing)
9. [Diagnostics](#diagnostics)
10. [Configuration](#configuration)
11. [Internationalization](#internationalization)
12. [Download & Installation](#download--installation)
13. [FAQ](#faq)

---

## Installation

### Requirements

#### Build Dependencies

| Platform | Requirements |
|----------|--------------|
| **All** | Rust 1.92.0+, FFmpeg development libraries, Clang |
| **Linux** | `libxcb`, `fontconfig`, `libavcodec-dev`, `libavformat-dev`, `libavutil-dev`, `libswscale-dev`, `libclang-dev` |
| **Windows** | MSVC (Visual Studio with "Desktop development with C++"), LLVM (with Clang), vcpkg |
| **macOS** | Xcode Command Line Tools, FFmpeg (via Homebrew) |

#### Runtime Dependencies

| Platform | Requirements |
|----------|--------------|
| **All** | FFmpeg libraries |
| **Windows** | MSVC runtime |

> **Platform Note:** IcedLens is primarily developed and tested on Linux Mint. macOS and Windows have not been tested. Pre-built binaries are only available for Linux (AppImage). Other platforms must build from source—you may encounter issues, please report them!

### Build from Source

```bash
git clone https://codeberg.org/Bawycle/iced_lens.git
cd iced_lens
cargo build --release
```

The binary is located at `target/release/iced_lens`.

### Development Build

```bash
cargo build           # Debug build
cargo test            # Run tests
cargo clippy          # Linting
cargo fmt             # Format code
```

---

## Command-Line Interface

```
USAGE:
    iced_lens [OPTIONS] [PATH]

OPTIONS:
    -h, --help              Show help text
        --lang <id>         Set locale (en-US, fr, es, de, it)
        --i18n-dir <path>   Override translation directory
        --data-dir <path>   Override data directory (state files)
        --config-dir <path> Override config directory (settings.toml)

ARGS:
    <PATH>    Path to a media file or directory
```

### Environment Variables

- `ICED_LENS_DATA_DIR` — Override data directory
- `ICED_LENS_CONFIG_DIR` — Override config directory

### Examples

```bash
# Open a single image
iced_lens photo.jpg

# Open directory (loads first file based on sort order)
iced_lens ~/Pictures/

# Override language
iced_lens --lang fr image.png
```

---

## Keyboard Shortcuts

### Viewer Mode

| Key | Action |
|-----|--------|
| `E` | Enter editor mode (images only) |
| `I` | Toggle metadata panel |
| `F11` | Toggle fullscreen |
| `Esc` | Exit fullscreen |
| `←` / `→` | Navigate media / seek video |
| `↑` / `↓` | Increase / decrease volume |
| `R` | Rotate image clockwise (temporary, images only) |
| `Shift+R` | Rotate image counter-clockwise (temporary, images only) |
| `Space` | Play/pause video |
| `M` | Toggle mute |
| `J` | Decrease playback speed |
| `L` | Increase playback speed |
| `,` | Step back one frame (while paused) |
| `.` | Step forward one frame (while paused) |

### Editor Mode

| Key | Action |
|-----|--------|
| `Ctrl+S` | Save |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Esc` | Cancel changes or exit editor |

On macOS, use `Cmd` instead of `Ctrl`.

---

## Navigation & Viewing

### Mouse Controls

- **Scroll wheel:** Zoom in/out (when cursor is over image)
- **Click + drag:** Pan large images
- **Double-click:** Toggle fullscreen

### Zoom Behavior

- Fit-to-window mode keeps content centered on resize
- Manual zoom level persists until fit-to-window is re-enabled
- Zoom step is configurable in Settings

### Directory Navigation

- Arrow keys or overlay arrows browse sibling files
- Navigation loops at directory boundaries
- Directory is rescanned on each navigation (reflects file changes)
- Corrupted or unloadable files are automatically skipped (configurable limit in Settings)

### Media Filters

Filter the current directory to show only matching files:

- **Media type**: Images only, Videos only, or All
- **Orientation**: Landscape, Portrait, or Square
- **Date range**: Filter by file modification date (start and/or end)

Access filters via the filter dropdown in the toolbar. When filters are active, an indicator shows how many files match. Filters can optionally persist across sessions (Settings → Display).

### Fullscreen

- Enter via F11, double-click, or toolbar button
- Controls auto-hide after configurable delay
- Exit with Esc or F11

---

## Editing Tools

All editing is **non-destructive**. Changes are only applied when you save.

### Rotate

90° increments, instant apply. Full rotation is added to transformation history.

### Crop

- Drag handles to adjust crop region
- Preset ratios: Free, Square (1:1), 16:9, 9:16, 4:3, 3:4
- Click Apply to commit

### Resize

- Slider: 10–400% of original size
- Width/height inputs with optional aspect lock
- Live preview updates as you adjust
- **AI Upscaling:** For enlargements (>100%), optional Real-ESRGAN 4x upscaling produces sharper results than traditional interpolation. Enable in Settings → AI / Machine Learning.

### Light

- Brightness slider (-100 to +100)
- Contrast slider (-100 to +100)
- Real-time preview

### Undo/Redo

Full transformation history. Each tool application creates a history entry.

### Mouse Controls

- **Scroll wheel:** Zoom in/out of the image
- **Click + drag:** Pan when zoomed in

---

## AI Deblur

Experimental feature using the NAFNet neural network to sharpen blurry images.

### Setup

1. Go to Settings → AI / Machine Learning
2. Enable "AI Deblurring"
3. Wait for model download (~92 MB from Hugging Face)
4. Model is validated automatically when you first open the image editor

### Usage

1. Open an image in the editor
2. Select the AI Deblur tool
3. Click "Apply Deblur"
4. Wait for processing (spinner overlay)
5. Save when satisfied

### Notes

- Works with any image size (small images are automatically padded)
- Can only be applied once per editing session (multiple applications degrade quality)
- Model integrity verified with BLAKE3 checksum
- Processing runs on CPU

---

## AI Upscaling

Enhance image enlargements using the Real-ESRGAN neural network for sharper results than traditional interpolation.

### Setup

1. Go to Settings → AI / Machine Learning
2. Enable "AI Upscaling"
3. Wait for model download (~64 MB from Hugging Face)
4. Model is validated automatically when you first open the image editor

### Usage

1. Open an image in the editor
2. Select the Resize tool
3. Set scale above 100% (enlargement)
4. Check "Use AI upscaling (Real-ESRGAN)"
5. Click Apply
6. Wait for processing (spinner overlay)
7. Save when satisfied

### Notes

- Only applies to enlargements (>100%), not reductions
- Uses Real-ESRGAN 4x model for high-quality upscaling
- Model integrity verified with BLAKE3 checksum
- Processing runs on CPU

---

## Metadata Editing

### Supported Fields

**Dublin Core / XMP** (JPEG, PNG, WebP, TIFF):
- Title, Creator, Description, Keywords, Copyright

**EXIF** (JPEG, PNG, WebP, TIFF, HEIC):
- Camera make/model, date taken, exposure, aperture, ISO, focal length, GPS

### Usage

1. Open the metadata panel (I key or Info button)
2. Click Edit
3. Modify fields
4. Save (overwrites original) or Save As (creates copy)

### Notes

- Video metadata viewing only (editing not supported)
- Files with corrupted EXIF data are handled gracefully
- Smart date picker supports multiple input formats

---

## Diagnostics

The Diagnostics screen helps troubleshoot issues by collecting anonymized application data that can be shared with developers.

### Accessing Diagnostics

1. Open Settings menu (gear icon in toolbar)
2. Select "Diagnostics"

### What Data is Collected

| Category | Examples | Privacy |
|----------|----------|---------|
| **User Actions** | Navigation, zoom, playback controls | No file paths |
| **App State** | Media loaded/failed, editor opened | File paths hashed |
| **Operations** | AI processing duration, seek timing | Performance metrics only |
| **System Info** | OS, CPU cores, RAM, disk space | Generic system info |

### Privacy Protection

- **No file paths**: All paths are hashed using blake3 (8-character hash)
- **No file content**: Only metadata like dimensions and file size
- **No personal data**: No usernames, network identities, or sensitive info
- **Local only**: Data stays on your device until you export it

### Using Diagnostics

1. **Enable collection**: Toggle "Collection" on to start capturing events
2. **Reproduce the issue**: Perform the actions that cause the problem
3. **Export report**: Click "Export to File" or "Copy to Clipboard"
4. **Share with developers**: Attach the JSON report to your bug report

### Export Format

Reports are exported as JSON with:
- Session metadata (app version, OS, timestamp)
- System resources snapshot
- Captured events with timestamps
- Summary statistics

---

## Configuration

Configuration is stored in a platform-appropriate directory:

- **Linux:** `~/.config/iced_lens/settings.toml`
- **macOS:** `~/Library/Application Support/iced_lens/settings.toml`
- **Windows:** `%APPDATA%\iced_lens\settings.toml`

### Settings (via UI)

| Category | Options |
|----------|---------|
| General | Language, theme mode (System/Light/Dark) |
| Display | Background theme, sort order, zoom step, auto-skip limit (1–20) |
| Video | Autoplay, volume (0–150% with perceptual scaling), audio normalization, frame cache size |
| Fullscreen | Overlay timeout |
| AI | Enable deblur, enable upscaling, model URLs |

### Reset Configuration

Delete `settings.toml` and restart. Defaults will regenerate.

### Persisted State

Application state (last directory, window position, etc.) is stored separately in the data directory.

---

## Internationalization

Powered by [Fluent](https://projectfluent.org/).

### Available Locales

- English (en-US)
- French (fr)
- German (de)
- Spanish (es)
- Italian (it)

### Runtime Switching

Change language in Settings. UI updates immediately without restart.

### Custom Translations

```bash
iced_lens --i18n-dir /path/to/translations
```

Translation files use `.ftl` extension. See `assets/i18n/` for examples.

---

## Download & Installation

Download the latest release from [Releases](https://codeberg.org/Bawycle/iced_lens/releases):

| Platform | Format | Instructions |
|----------|--------|--------------|
| **Linux x86_64** | AppImage | `chmod +x iced_lens-*.AppImage && ./iced_lens-*.AppImage` |
| **Windows x86_64** | Installer | Run `IcedLens-*-setup.exe` and follow the wizard |

> **macOS:** No pre-built binaries are provided. To build from source or create distribution packages, see [CONTRIBUTING.md](../CONTRIBUTING.md#distribution-packaging).

---

## FAQ

**Q: Why MPL-2.0?**
A: File-level copyleft offers balanced reciprocity without full project copyleft.

**Q: Does it work on Windows/macOS?**
A: Windows is fully supported with a pre-built installer. macOS has not been tested—see [CONTRIBUTING.md](../CONTRIBUTING.md) to build from source.

**Q: Can I edit videos?**
A: Not yet. Video playback and frame capture are supported, but editing is images only.

**Q: How do I report bugs?**
A: Open an issue with OS, Rust version, reproduction steps, and logs if available.

**Q: Is the AI model safe?**
A: The model is downloaded from Hugging Face and verified with BLAKE3 checksum before use.
