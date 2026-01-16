# Claude Instructions for IcedLens

- Fais tout en anglais excepté me parler, parle-moi en français.
- Applique les directives détaillées du fichier **CONTRIBUTING.md**.

## Quick Reference

IcedLens is a privacy-first media viewer and editor with AI enhancement capabilities, built with Rust and Iced.

### Core Architecture Principles

1. **Single Source of Truth**
   - Each data piece has one authoritative source
   - Configuration defaults centralized in `config/defaults.rs`
   - Media list managed by `MediaNavigator`

2. **Event-Driven Architecture (Elm/Iced Pattern)**
   - Each domain manages its own state via messages
   - App orchestrates by reacting to `Effect` enums and dispatching messages
   - **No cross-domain mutations** (never `ctx.viewer.field = value` from handlers)
   - Use messages like `ClearMedia`, `MediaLoaded`, `StartLoadingMedia` to communicate

3. **Unidirectional Data Flow**
   - Data flows down, events flow up
   - State changes happen in predictable places

### Key Patterns

| Pattern | When to Use |
|---------|-------------|
| **Newtype** | Bounded values (see table below) |
| **Message/Effect** | Cross-component communication |
| **Design Tokens** | Colors, spacing, sizing (never hardcode) |
| **Action Icons** | Semantic action → visual icon mapping |

### Newtypes (Bounded Values)

Always use newtypes for constrained values instead of primitives with `.clamp()`:

| Type | Range | Module |
|------|-------|--------|
| `Volume` | 0.0–1.5 | `domain/video/newtypes.rs` |
| `PlaybackSpeed` | 0.1–8.0 | `domain/video/newtypes.rs` |
| `KeyboardSeekStep` | 0.5–30.0 sec | `domain/video/newtypes.rs` |
| `FrameCacheMb` | 16–512 MB | `video_player/frame_cache_size.rs` |
| `FrameHistoryMb` | 32–512 MB | `video_player/frame_history_size.rs` |
| `ZoomPercent` | 10%–800% | `domain/ui/newtypes.rs` |
| `ZoomStep` | 1%–200% | `domain/ui/newtypes.rs` |
| `OverlayTimeout` | 1–30 sec | `domain/ui/newtypes.rs` |
| `RotationAngle` | 0–359° | `domain/ui/newtypes.rs` |
| `ResizeScale` | 10%–400% | `domain/editing/newtypes.rs` |
| `MaxSkipAttempts` | 1–20 | `domain/editing/newtypes.rs` |
| `AdjustmentPercent` | -100–+100 | `domain/editing/newtypes.rs` |

### User Feedback Strategy

| Type | Method | When |
|------|--------|------|
| **Toast (Notification)** | `Notification::success/warning/error()` | Recoverable errors, action confirmations |
| **ErrorDisplay** | `ErrorDisplay::new()` | Critical blocking errors (rare) |
| **Silent** | Early return | Internal errors with acceptable fallback |

### Style System Layers

```
Component Styles (src/ui/styles/*.rs)
        ↓
Theme System (theming.rs, theme.rs)
        ↓
Action Icons (action_icons.rs) → Icons (icons.rs)
        ↓
Design Tokens (design_tokens.rs)
```

- **Never** use `Color::from_rgb(...)` inline → use `design_tokens::palette`
- **Icons**: Named by visual appearance (`play()`, `cross()`)
- **Action Icons**: Named by semantic action (`video::step_forward()`)
- Changing an icon = modify only `action_icons.rs`

### Configuration & Defaults

- All default values in `src/app/config/defaults.rs`
- Compile-time validation ensures constraints are valid
- Add new defaults here, not scattered in modules

### AI/ML Features

The project includes AI capabilities via ONNX Runtime (`ort`):
- **Deblur**: NAFNet model for image deblurring
- **Upscale**: Real-ESRGAN x4 for AI upscaling
- Models are downloaded on first use, cached locally

### Essential Commands

```bash
cargo test                                    # Run all tests (required before commit)
cargo clippy --all --all-targets -- -D warnings  # Lint check
cargo fmt --all                               # Format code
cargo audit                                   # Security audit
```

### File Organization

| Location | Purpose |
|----------|---------|
| `src/domain/` | Pure domain types, newtypes, value objects (ZERO external deps) |
| `src/application/` | Traits (ports), query services |
| `src/infrastructure/` | FFmpeg, ONNX, diagnostics adapters |
| `src/app/` | Application core, message handling, config, i18n |
| `src/media/` | Media loading, transforms, navigation |
| `src/video_player/` | Video playback engine (FFmpeg, audio sync) |
| `src/ui/` | UI components, styles, design system |
| `src/ui/viewer/clusters/` | Feature clusters (zoom+pan+rotation, loading+media+errors, video) |
| `src/ui/viewer/subcomponents/` | Reusable state units (State + Message + Effect) |
| `src/ui/image_editor/` | Image editing tools (crop, resize, deblur) |
| `assets/i18n/` | Translation files (Fluent `.ftl`) |

### TDD Workflow

1. Write tests defining expected behavior
2. Implement code to pass tests
3. `cargo test` must pass before commit
4. Refactor while keeping tests green

### Security Reminders

- Validate all user inputs
- Never log/display full file paths
- Use proper error handling (no `unwrap()` on user data)
- Check vulnerable dependencies with `cargo audit`
