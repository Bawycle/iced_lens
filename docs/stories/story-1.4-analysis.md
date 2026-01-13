# Story 1.4 Technical Analysis: Missing Application States

**Purpose:** This document provides a technical analysis of application states that should be captured in Story 1.4 but are currently missing from the specification.

**Author:** Architect (Winston)
**Date:** 2026-01-13
**Status:** Ready for PO Review

---

## Current Story 1.4 Scope

### Planned AppStateEvent variants
- MediaLoadingStarted, MediaLoaded, MediaFailed
- VideoPlaying, VideoPaused, VideoSeeking
- EditorOpened, EditorClosed

### Planned AppOperation variants
- DecodeFrame, ResizeImage, ApplyFilter, ExportFrame, LoadMetadata

---

## Missing States Analysis

### 1. Video Player States (High Priority)

**Source:** `src/video_player/state.rs`

| State | Description | Diagnostic Value |
|-------|-------------|------------------|
| `VideoBuffering` | Video is buffering/loading frames | Identifies loading delays, network issues |
| `VideoError { message }` | Playback error occurred | Critical for error diagnosis |
| `VideoAtEndOfStream` | Reached end of video | Understanding playback flow |
| `VideoLoopToggled { enabled }` | Loop mode changed | User behavior tracking |
| `VideoSpeedChanged { speed }` | Playback speed changed | Correlate with performance issues |
| `VideoFrameStepping { direction }` | Frame-by-frame navigation active | Advanced usage tracking |
| `VideoSpeedAutoMuted` | Audio muted due to high speed | Explain unexpected mute |

**Context data needed:**
- `playback_speed: f64` (0.1 - 8.0)
- `position_secs: f64`
- `is_looping: bool`

### 2. Image Editor States (High Priority)

**Source:** `src/ui/image_editor/state/*.rs`

| State | Description | Diagnostic Value |
|-------|-------------|------------------|
| `EditorToolActivated { tool }` | Tool selection changed | Workflow analysis |
| `EditorCropStarted` | Crop overlay activated | Operation timing |
| `EditorCropApplied { dimensions }` | Crop applied | Success tracking |
| `EditorResizeStarted` | Resize dialog opened | Operation timing |
| `EditorResizeApplied { scale, ai_upscale }` | Resize applied | AI vs standard tracking |
| `EditorDeblurStarted` | AI deblur initiated | AI operation tracking |
| `EditorDeblurProgress { percent }` | Deblur progress update | Long operation monitoring |
| `EditorDeblurCompleted` | Deblur finished | Duration calculation |
| `EditorDeblurCancelled` | Deblur cancelled by user | User interruption tracking |
| `EditorAdjustmentChanged { brightness, contrast }` | Adjustments modified | Edit flow analysis |
| `EditorUnsavedChanges { has_changes }` | Edit state changed | Data loss risk tracking |
| `EditorHistoryChanged { can_undo, can_redo }` | Undo/redo state | Edit complexity tracking |

**Context data needed:**
- `tool: EditorTool` enum (Crop, Resize, Adjust, Deblur)
- `has_unsaved_changes: bool`
- `history_depth: u32`

### 3. UI/View States (Medium Priority)

**Source:** `src/ui/state/*.rs`

| State | Description | Diagnostic Value |
|-------|-------------|------------------|
| `ZoomChanged { percent, fit_to_window }` | Zoom level changed | View preference tracking |
| `FullscreenEntered` | Entered fullscreen mode | Mode tracking |
| `FullscreenExited` | Exited fullscreen mode | Mode tracking |
| `RotationChanged { angle }` | Image rotation changed | View preference tracking |
| `DragPanStarted` | User started panning | Interaction tracking |
| `DragPanEnded` | User stopped panning | Interaction tracking |

**Context data needed:**
- `zoom_percent: u16` (10 - 800)
- `rotation_angle: u16` (0 - 359)
- `is_fullscreen: bool`

### 4. Navigation States (Medium Priority)

**Source:** `src/media/navigator.rs`

| State | Description | Diagnostic Value |
|-------|-------------|------------------|
| `FilterActivated { filter_type }` | Media filter applied | Navigation pattern tracking |
| `FilterDeactivated` | Filter removed | Navigation pattern tracking |
| `NavigationPositionChanged { index, total, filtered }` | Position in media list | Context for navigation issues |

**Context data needed:**
- `current_index: usize`
- `total_count: usize`
- `filtered_count: usize`
- `filter_active: bool`

### 5. AI Model States (Medium Priority)

**Source:** `src/ui/settings.rs`, `src/app/persisted_state.rs`

| State | Description | Diagnostic Value |
|-------|-------------|------------------|
| `ModelDownloadStarted { model }` | AI model download initiated | AI feature adoption |
| `ModelDownloadProgress { model, percent }` | Download progress | Long operation tracking |
| `ModelDownloadCompleted { model }` | Download finished | Success tracking |
| `ModelDownloadFailed { model, reason }` | Download failed | Error diagnosis |
| `ModelEnabled { model }` | AI model enabled | Feature usage |
| `ModelDisabled { model }` | AI model disabled | Feature usage |

**Context data needed:**
- `model: AIModel` enum (Deblur, Upscale)
- `model_status: ModelStatus` enum

### 6. Metadata Editor States (Low Priority)

**Source:** `src/ui/metadata_panel/state.rs`

| State | Description | Diagnostic Value |
|-------|-------------|------------------|
| `MetadataEditStarted` | Metadata editing began | Feature usage tracking |
| `MetadataValidationError { field }` | Validation error occurred | UX issue identification |
| `MetadataSaved` | Metadata successfully saved | Success tracking |
| `MetadataSaveFailed { reason }` | Metadata save failed | Error diagnosis |

---

## Missing Operations Analysis

### Operations to Add

| Operation | Duration Tracked | Context Data | Diagnostic Value |
|-----------|------------------|--------------|------------------|
| `AIDeblurProcess` | Yes | `image_size_category`, `success` | AI performance analysis |
| `AIUpscaleProcess` | Yes | `scale_factor`, `image_size_category` | AI performance analysis |
| `VideoSeekOperation` | Yes | `seek_distance_secs`, `success` | Seek performance tracking |
| `AudioVideoSync` | Yes | `drift_ms`, `correction_applied` | A/V sync issue diagnosis |
| `ModelDownload` | Yes | `model`, `size_bytes`, `success` | Network/download issues |
| `FrameHistoryNavigation` | No | `direction`, `frames_available` | Advanced usage tracking |
| `MetadataParse` | Yes | `file_type`, `fields_extracted` | Format compatibility |
| `MetadataWrite` | Yes | `file_type`, `fields_written` | Save performance |

---

## Recommended Priority for Implementation

### Must Have (Story 1.4)
1. Video Player states (Buffering, Error, AtEndOfStream)
2. Editor tool states (DeblurStarted/Progress/Completed/Cancelled)
3. AI operations (AIDeblurProcess, AIUpscaleProcess)

### Should Have (Story 1.4 or 1.4b)
4. Editor session states (UnsavedChanges, HistoryChanged)
5. AI Model states (download lifecycle)
6. VideoSeekOperation with duration

### Nice to Have (Future story)
7. UI view states (Zoom, Fullscreen, Rotation)
8. Navigation states (Filter, Position)
9. Metadata editor states

---

## Size Categories (Already Planned - Confirm)

The story already plans size categories. Confirm these ranges:

| Category | Range | Use Case |
|----------|-------|----------|
| Small | < 1 MB | Quick operations expected |
| Medium | 1 - 10 MB | Normal operations |
| Large | 10 - 100 MB | Slower operations acceptable |
| VeryLarge | > 100 MB | Performance concerns expected |

---

## Recommended AppStateEvent Enum Structure

```rust
pub enum AppStateEvent {
    // Media Lifecycle (already planned)
    MediaLoadingStarted { media_type: MediaType, size_category: SizeCategory },
    MediaLoaded { media_type: MediaType, size_category: SizeCategory },
    MediaFailed { media_type: MediaType, reason: String },

    // Video Playback (already planned + additions)
    VideoPlaying { position_secs: f64 },
    VideoPaused { position_secs: f64 },
    VideoSeeking { target_secs: f64 },
    VideoBuffering { position_secs: f64 },           // NEW
    VideoError { message: String },                   // NEW
    VideoAtEndOfStream,                               // NEW
    VideoLoopToggled { enabled: bool },               // NEW
    VideoSpeedChanged { speed: f64 },                 // NEW

    // Editor (already planned + additions)
    EditorOpened { source: EditorSource },
    EditorClosed { had_unsaved_changes: bool },
    EditorToolActivated { tool: EditorTool },         // NEW
    EditorDeblurStarted,                              // NEW
    EditorDeblurProgress { percent: f32 },            // NEW
    EditorDeblurCompleted,                            // NEW
    EditorDeblurCancelled,                            // NEW
    EditorUnsavedChanges { has_changes: bool },       // NEW

    // AI Models (NEW category)
    ModelDownloadStarted { model: AIModel },          // NEW
    ModelDownloadCompleted { model: AIModel },        // NEW
    ModelDownloadFailed { model: AIModel },           // NEW
}
```

---

## Recommended AppOperation Enum Structure

```rust
pub enum AppOperation {
    // Already planned
    DecodeFrame { duration_ms: u64 },
    ResizeImage { duration_ms: u64, size_category: SizeCategory },
    ApplyFilter { duration_ms: u64, filter_type: String },
    ExportFrame { duration_ms: u64 },
    LoadMetadata { duration_ms: u64 },

    // NEW operations
    AIDeblurProcess {
        duration_ms: u64,
        size_category: SizeCategory,
        success: bool,
    },
    AIUpscaleProcess {
        duration_ms: u64,
        scale_factor: f32,
        size_category: SizeCategory,
        success: bool,
    },
    VideoSeek {
        duration_ms: u64,
        seek_distance_secs: f64,
    },
    ModelDownload {
        duration_ms: u64,
        model: AIModel,
        success: bool,
    },
}
```

---

## Next Steps

1. **PO Review:** Review this analysis and decide which states to include in Story 1.4
2. **Story Update:** Update Story 1.4 acceptance criteria and tasks
3. **Dev Implementation:** Implement the revised story

---

## References

- `src/video_player/state.rs` - PlaybackState enum
- `src/ui/image_editor/state/*.rs` - Editor tool states
- `src/ui/state/*.rs` - UI view states
- `src/media/navigator.rs` - Navigation state
- `src/ui/settings.rs` - Settings and AI model states
- `src/diagnostics/events.rs` - Current diagnostic event structure
