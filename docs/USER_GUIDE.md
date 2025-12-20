# IcedLens User Guide

Complete documentation for IcedLens features, configuration, and usage.

## Table of Contents

1. [Installation](#installation)
2. [Command-Line Interface](#command-line-interface)
3. [Keyboard Shortcuts](#keyboard-shortcuts)
4. [Navigation & Viewing](#navigation--viewing)
5. [Editing Tools](#editing-tools)
6. [AI Deblur](#ai-deblur)
7. [Metadata Editing](#metadata-editing)
8. [Configuration](#configuration)
9. [Internationalization](#internationalization)
10. [Download & Installation](#download--installation)
11. [FAQ](#faq)

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
| `Space` | Play/pause video |
| `M` | Toggle mute |

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

- Slider: 10–200% of original size
- Width/height inputs with optional aspect lock
- Live preview updates as you adjust

### Light

- Brightness slider (-100 to +100)
- Contrast slider (-100 to +100)
- Real-time preview

### Undo/Redo

Full transformation history. Each tool application creates a history entry.

---

## AI Deblur

Experimental feature using the NAFNet neural network to sharpen blurry images.

### Setup

1. Go to Settings → AI / Machine Learning
2. Enable "AI Deblurring"
3. Wait for model download (~92 MB from Hugging Face)
4. Model is validated automatically

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

## Metadata Editing

### Supported Fields

**Dublin Core / XMP:**
- Title, Creator, Description, Keywords, Copyright

**EXIF:**
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

## Configuration

Configuration is stored in a platform-appropriate directory:

- **Linux:** `~/.config/iced_lens/settings.toml`
- **macOS:** `~/Library/Application Support/iced_lens/settings.toml`
- **Windows:** `%APPDATA%\iced_lens\settings.toml`

### Settings (via UI)

| Category | Options |
|----------|---------|
| General | Language, theme mode (System/Light/Dark) |
| Display | Background theme, sort order, zoom step |
| Video | Autoplay, volume, audio normalization, frame cache size |
| Fullscreen | Overlay timeout |
| AI | Enable deblur, model URL |

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
