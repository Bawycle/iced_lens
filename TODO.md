# TODO - Next Release

> This file tracks planned work for the next release. It lives only in the `dev` branch and is not included in releases.

## Bugs to Fix

- [ ] **Missing loading indicator on startup** - When launching the app with a heavy media file (e.g., a multi-minute video), no loading indicator is shown. The user sees "No media loaded" with "Open file" button instead of a spinner, while the media is actually loading in the background. Regression introduced in `fix/seeking-and-disk-performance` merge.
- [ ] **Launching with a folder path doesn't work** - Opening the app with a directory path as argument no longer loads the folder contents. Regression introduced in `fix/seeking-and-disk-performance` merge.
- [ ] **Software tag and modification date not written** - When saving or "Save As" in the image editor with "Add software tag and modification date" checkbox enabled, these metadata fields are not added to the saved image.

## Technical Debt (P3)

### Diagnostics - Missing Instrumentation
> Ref: [Epic 4 Final Audit](docs/qa/assessments/epic4-final-audit.md)

- [ ] `EditorUnsavedChanges` state event not logged
- [ ] `DecodeFrame` operation not instrumented
- [ ] `ResizeImage` operation not instrumented
- [ ] `ApplyFilter` operation not instrumented
- [ ] `ExportFrame` operation not instrumented
- [ ] `LoadMetadata` operation not instrumented

## Changed


## Planned Features

### Viewer

#### Media Filters for Navigation
- [ ] Add auto-focus between segmented date input fields (blocked, requires `text_input::focus(id)` Task API, expected in future iced versions)

#### Metadata Sidebar
- [ ] Allow text selection and copying in the metadata sidebar (blocked, pending native support in Iced 0.15.0)
- [ ] Add video metadata editing support

### Video Player



### Help
- [ ] Allow text selection and copying in the help screen (blocked, pending native support in Iced 0.15.0)

### Video Editor
- [ ] Create a simple video editor allowing users to trim videos by removing segments. The editor should let users play the video, seek to any position, step forward/backward frame by frame, and change the playback speed.

## Refactoring

### Architecture Migration: Domain-Driven Design + Clean Architecture

The codebase has grown organically and some files exceed healthy sizes (e.g., `viewer/component.rs` at 2,657 LOC, `app/mod.rs` at 2,134 LOC). Before adding major features like the video editor, a progressive migration to a cleaner architecture is recommended.

#### Target Architecture

```
src/
├── domain/                      # Core business logic (ZERO external deps)
│   ├── media/                   # Media entities, navigation, filtering
│   ├── video/                   # Playback state machine, sync logic
│   ├── editing/                 # Edit operations (Command pattern)
│   ├── metadata/                # EXIF, IPTC, GPS value objects
│   └── ai/                      # Enhancement job definitions
│
├── application/                 # Use cases & orchestration
│   ├── command/                 # CQRS write operations
│   ├── query/                   # CQRS read operations
│   ├── port/                    # Traits (MediaLoader, VideoDecoder, AIProcessor)
│   └── effect.rs                # Side effect definitions
│
├── infrastructure/              # Concrete implementations
│   ├── ffmpeg/                  # FFmpeg adapter (implements VideoDecoder)
│   ├── onnx/                    # ONNX adapter (implements AIProcessor)
│   ├── filesystem/              # File I/O adapters
│   └── persistence/             # Config storage
│
└── presentation/                # UI layer (Iced)
    ├── app/                     # Shell (< 300 LOC)
    ├── screen/                  # ViewerScreen, EditorScreen, SettingsScreen
    ├── component/               # Reusable widgets (< 200 LOC each)
    ├── gesture/                 # Pan, zoom, drag handlers (extracted)
    └── design_system/           # Tokens, themes, icons
```

#### Key Principles

| Principle | Application |
|-----------|-------------|
| **Dependency Inversion** | `domain/` has zero dependencies; `infrastructure/` implements `application/port/` traits |
| **Components < 400 LOC** | Split large files; exceeding this is a refactoring signal |
| **Aggregate Roots** | `MediaList` manages its `MediaItem`s; no direct item mutations |
| **Immutable Value Objects** | `Zoom`, `Volume`, `Rotation` use `with_value()`, not setters |
| **One message = one intent** | `NavigateToNext` rather than `SetIndex(current + 1)` |

#### Benefits

- **Testability**: Domain testable without UI; infrastructure mockable via traits
- **Maintainability**: Small files with clear responsibilities
- **Scalability**: Adding video editor = new bounded context, no existing code touched
- **Parallelization**: Teams can work on separate domains independently

#### Next Step

- [ ] Establish a progressive migration plan (identify migration order, define intermediate milestones, keep the app functional at each step)

## Packaging / Distribution

### Flatpak
- [ ] Test Flatpak build locally with `flatpak-builder` (Waiting new Freedesktop SDK with support of Rust 1.92)
- [ ] Prepare Flathub submission:
  - [ ] Fork [flathub/flathub](https://github.com/flathub/flathub)
  - [ ] Create PR with manifest following [Flathub submission guidelines](https://docs.flathub.org/docs/for-app-authors/submission/)
  - [ ] Ensure app passes Flathub quality guidelines (no network at build time, proper permissions, etc.)

## Notes

- Test videos can be generated with `scripts/generate-test-videos.sh`
