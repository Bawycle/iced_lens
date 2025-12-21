# IcedLens

Privacy-first media viewer and editor with AI deblurring.

[![Release](https://img.shields.io/badge/release-v0.4.1-blue)](https://codeberg.org/Bawycle/iced_lens/releases)
[![License: MPL-2.0](https://img.shields.io/badge/License-MPL--2.0-brightgreen.svg)](LICENSE)
[![Rust 1.92+](https://img.shields.io/badge/Rust-1.92.0%2B-blue)](https://www.rust-lang.org)
![i18n](https://img.shields.io/badge/i18n-en--US|fr|es|de|it-green)

> **Pre-1.0**: Functional but under active development. Tested on Linux Mint 22.2.

## Why IcedLens?

- **AI Deblur** — Sharpen blurry photos using NAFNet neural network (experimental)
- **AI Upscaling** — Enlarge images up to 4x with Real-ESRGAN for sharper results
- **Metadata editing** — Edit EXIF and Dublin Core/XMP metadata directly
- **Privacy-first** — No telemetry, no cloud, fully local (except optional AI model download)
- **Live i18n** — Switch languages at runtime without restart
- **Non-destructive editing** — Full undo/redo history, original preserved until save

Built with the [Iced](https://iced.rs/) GUI toolkit.

## Quick Start

**Linux (AppImage):** Download from [Releases](https://codeberg.org/Bawycle/iced_lens/releases), make executable, run.

**Windows:** Download the installer from [Releases](https://codeberg.org/Bawycle/iced_lens/releases) and run the setup wizard.

**Build from source:**

```bash
git clone https://codeberg.org/Bawycle/iced_lens.git
cd iced_lens
cargo build --release
./target/release/iced_lens /path/to/image.jpg
```

**Build requirements:** Rust 1.92+, FFmpeg dev libraries, Clang. See [User Guide](docs/USER_GUIDE.md#requirements) for platform-specific details. macOS: untested, no binaries provided.

## Features

### Viewing
Images (JPEG, PNG, GIF, WebP, TIFF, BMP, ICO, SVG) and videos (MP4, AVI, MOV, MKV, WebM) with zoom, pan, fullscreen, frame-by-frame navigation, playback speed control (0.1x–8x), and volume amplification up to 150%.

### Editing
Rotate, crop, resize, brightness/contrast — all with live preview and undo/redo. Save or Save As when ready.

### AI Deblur (Experimental)
Enable in Settings → AI / Machine Learning. Downloads a ~92 MB ONNX model on first use. Works with any image size.

### Metadata
View and edit EXIF (camera, exposure, GPS) and Dublin Core/XMP (title, creator, copyright) metadata.

## Documentation

- **[User Guide](docs/USER_GUIDE.md)** — Keyboard shortcuts, configuration, CLI options
- **[Contributing](CONTRIBUTING.md)** — How to contribute code or translations
- **[Changelog](CHANGELOG.md)** — Version history

## Security

Local-first: images are processed locally. The AI Deblur feature downloads a model from Hugging Face (BLAKE3 checksum verified). No other network activity. Report vulnerabilities via [SECURITY.md](SECURITY.md).

## License

[MPL-2.0](LICENSE) — File-level copyleft. Icons use a [separate license](ICON_LICENSE.md).

## Acknowledgements

IcedLens is built on the shoulders of great open-source projects:

- [Rust](https://www.rust-lang.org/) — Systems programming language
- [Iced](https://iced.rs/) — Cross-platform GUI toolkit
- [FFmpeg](https://ffmpeg.org/) — Video decoding via [ffmpeg-next](https://crates.io/crates/ffmpeg-next)
- [image-rs](https://github.com/image-rs/image) — Image decoding and processing
- [ONNX Runtime](https://onnxruntime.ai/) — AI inference via [ort](https://crates.io/crates/ort)
- [OpenCV](https://opencv.org/) — NAFNet model training
- [Fluent](https://projectfluent.org/) — Localization system

...and many other excellent crates from the Rust ecosystem. See [Cargo.toml](Cargo.toml) for the full list.
