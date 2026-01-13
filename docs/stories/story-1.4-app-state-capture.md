# Story 1.4: Application State and Operation Capture

**Epic:** 1 - Diagnostics Core & Data Collection
**Status:** Completed
**Priority:** High
**Estimate:** 4-5 hours
**Depends On:** Story 1.1, 1.3

---

## Story

**As a** developer,
**I want** to capture application state changes, internal operations, and AI processing lifecycle,
**So that** I can understand what the application was doing during issues, especially for video playback and AI-powered features.

---

## Acceptance Criteria

### Core Infrastructure
1. `AppStateEvent` enum defined with all state variants (see Task 1)
2. `AppOperation` enum defined with duration tracking (see Task 2)
3. Supporting enums defined: `MediaType`, `SizeCategory`, `EditorTool`, `AIModel`
4. `DiagnosticEventKind` expanded with `AppState` and `Operation` variants
5. `log_state()` and `log_operation()` methods added to `DiagnosticsCollector`
6. All logging is non-blocking via existing channel mechanism

### State Coverage (Must Have)
7. Video playback states captured: Playing, Paused, Seeking, Buffering, Error, AtEndOfStream
8. Editor AI states captured: DeblurStarted, DeblurProgress, DeblurCompleted, DeblurCancelled
9. Media lifecycle states captured: LoadingStarted, Loaded, Failed

### State Coverage (Should Have)
10. Video configuration states: LoopToggled, SpeedChanged
11. Editor session states: UnsavedChanges tracking
12. AI Model lifecycle: DownloadStarted, DownloadCompleted, DownloadFailed

### Operations Coverage
13. AI operations tracked with duration: AIDeblurProcess, AIUpscaleProcess
14. Video seek operation tracked with duration and seek distance
15. Original operations retained: DecodeFrame, ResizeImage, ApplyFilter, ExportFrame, LoadMetadata

### Privacy & Quality
16. Size categories used instead of exact file sizes (Small/Medium/Large/VeryLarge)
17. No sensitive data captured (paths excluded, messages sanitized)
18. Unit tests verify all enum variants and helper functions
19. All tests pass, code formatted and linted

---

## Tasks

### Task 1: Define Supporting Enums
- [x] Create `MediaType` enum in `src/diagnostics/events.rs`
  - [x] Variants: `Image`, `Video`, `Unknown`
- [x] Create `SizeCategory` enum
  - [x] Variants: `Small` (<1MB), `Medium` (1-10MB), `Large` (10-100MB), `VeryLarge` (>100MB)
  - [x] Add `SizeCategory::from_bytes(bytes: u64) -> Self` helper
- [x] Create `EditorTool` enum
  - [x] Variants: `Crop`, `Resize`, `Adjust`, `Deblur`, `Upscale`
- [x] Create `AIModel` enum
  - [x] Variants: `Deblur`, `Upscale`
- [x] Implement `Serialize`/`Deserialize` for all enums

### Task 2: Define `AppStateEvent` Enum
- [x] In `src/diagnostics/events.rs`
- [x] **Media Lifecycle variants:**
  - [x] `MediaLoadingStarted { media_type: MediaType, size_category: SizeCategory }`
  - [x] `MediaLoaded { media_type: MediaType, size_category: SizeCategory }`
  - [x] `MediaFailed { media_type: MediaType, reason: String }`
- [x] **Video Playback variants (Must Have):**
  - [x] `VideoPlaying { position_secs: f64 }`
  - [x] `VideoPaused { position_secs: f64 }`
  - [x] `VideoSeeking { target_secs: f64 }`
  - [x] `VideoBuffering { position_secs: f64 }`
  - [x] `VideoError { message: String }`
  - [x] `VideoAtEndOfStream`
- [x] **Video Playback variants (Should Have):**
  - [x] `VideoLoopToggled { enabled: bool }`
  - [x] `VideoSpeedChanged { speed: f64 }`
- [x] **Editor variants (Must Have):**
  - [x] `EditorOpened { tool: Option<EditorTool> }`
  - [x] `EditorClosed { had_unsaved_changes: bool }`
  - [x] `EditorDeblurStarted`
  - [x] `EditorDeblurProgress { percent: f32 }`
  - [x] `EditorDeblurCompleted`
  - [x] `EditorDeblurCancelled`
- [x] **Editor variants (Should Have):**
  - [x] `EditorUnsavedChanges { has_changes: bool }`
- [x] **AI Model variants (Should Have):**
  - [x] `ModelDownloadStarted { model: AIModel }`
  - [x] `ModelDownloadCompleted { model: AIModel }`
  - [x] `ModelDownloadFailed { model: AIModel, reason: String }`
- [x] Implement `Serialize`/`Deserialize` with `#[serde(tag = "state", rename_all = "snake_case")]`

### Task 3: Define `AppOperation` Enum
- [x] In `src/diagnostics/events.rs`
- [x] **Original operations:**
  - [x] `DecodeFrame { duration_ms: u64 }`
  - [x] `ResizeImage { duration_ms: u64, size_category: SizeCategory }`
  - [x] `ApplyFilter { duration_ms: u64, filter_type: String }`
  - [x] `ExportFrame { duration_ms: u64 }`
  - [x] `LoadMetadata { duration_ms: u64 }`
- [x] **New AI operations (Must Have):**
  - [x] `AIDeblurProcess { duration_ms: u64, size_category: SizeCategory, success: bool }`
  - [x] `AIUpscaleProcess { duration_ms: u64, scale_factor: f32, size_category: SizeCategory, success: bool }`
- [x] **New video operation (Should Have):**
  - [x] `VideoSeek { duration_ms: u64, seek_distance_secs: f64 }`
- [x] Implement `Serialize`/`Deserialize` with `#[serde(tag = "operation", rename_all = "snake_case")]`

### Task 4: Expand `DiagnosticEventKind`
- [x] Replace placeholder `AppState` variant with:
  ```rust
  AppState { state: AppStateEvent }
  ```
- [x] Add new `Operation` variant:
  ```rust
  Operation { operation: AppOperation }
  ```
- [x] Ensure both serialize correctly with nested tag

### Task 5: Add Collector Methods
- [x] Add `log_state(&self, state: AppStateEvent)` to `DiagnosticsCollector`
  - [x] Creates `DiagnosticEvent` with `AppState` kind
  - [x] Sends via existing channel (non-blocking)
- [x] Add `log_operation(&self, operation: AppOperation)` to `DiagnosticsCollector`
  - [x] Creates `DiagnosticEvent` with `Operation` kind
  - [x] Sends via existing channel (non-blocking)
- [x] Follow same pattern as existing `log_action()`

### Task 6: Write Unit Tests
- [x] Test `SizeCategory::from_bytes()` with boundary values
- [x] Test all `AppStateEvent` variants serialize/deserialize correctly
- [x] Test all `AppOperation` variants serialize/deserialize correctly
- [x] Test `log_state()` creates correct event kind
- [x] Test `log_operation()` creates correct event kind
- [x] Test JSON output structure for nested enums

### Task 7: Run Validation
- [x] `cargo fmt --all`
- [x] `cargo clippy --all --all-targets -- -D warnings`
- [x] `cargo test`

### Task 8: Commit Changes
- [x] Stage all changes
- [x] Commit with message: `feat(diagnostics): add app state and operation capture [Story 1.4]`
- [x] Reference story number in commit body

---

## Dev Notes

### Architecture Decisions
- `AppStateEvent` and `AppOperation` are separate enums (not combined) for clarity and extensibility
- All enums use serde tag-based serialization for clean JSON output
- Duration is tracked in milliseconds (`u64`) for precision without floating-point issues

### Privacy Considerations
- `SizeCategory` abstracts exact file sizes to preserve privacy
- Error messages in `VideoError` and `MediaFailed` should be sanitized (no paths)
- `ModelDownloadFailed.reason` should contain generic failure reasons only

### Integration Points (Deferred to Later Stories)
- Video player state transitions → `src/video_player/state.rs`
- Editor deblur lifecycle → `src/ui/image_editor/state/deblur.rs`
- Model download events → `src/ui/settings.rs`
- Actual instrumentation will be added in Story 1.7 or later

### Size Category Thresholds
| Category | Bytes Range | Typical Content |
|----------|-------------|-----------------|
| Small | 0 - 1,048,575 | Thumbnails, small images |
| Medium | 1,048,576 - 10,485,759 | Standard photos |
| Large | 10,485,760 - 104,857,599 | RAW images, short videos |
| VeryLarge | 104,857,600+ | Long videos, 4K content |

---

## Testing

### Unit Tests
| Test File | Coverage |
|-----------|----------|
| `events.rs` | `SizeCategory::from_bytes()`, all enum variant creation, serde round-trip |
| `collector.rs` | `log_state()`, `log_operation()` event creation |

### Test Cases
1. `SizeCategory` boundary tests: 0, 1MB-1, 1MB, 10MB-1, 10MB, 100MB-1, 100MB
2. `AppStateEvent` variants with all context fields
3. `AppOperation` variants with duration and context
4. JSON serialization produces expected structure
5. Deserialization reconstructs correct variants

### Integration Tests
- None (handler integration deferred to later story)

---

## Revision History

| Date | Change | Author |
|------|--------|--------|
| 2026-01-13 | Initial draft | - |
| 2026-01-13 | Expanded scope based on technical analysis (story-1.4-analysis.md) | Sarah (PO) |

---

## QA Results

### Review Date: 2026-01-13

### Reviewed By: Quinn (Test Architect)

### Code Quality Assessment

Extensive AppStateEvent (20 variants) and AppOperation (8 variants) enums. SizeCategory abstracts file sizes for privacy. Well-documented thresholds. Proper serde tagging for clean JSON output.

### Refactoring Performed

None required.

### Compliance Check

- Coding Standards: ✓
- Project Structure: ✓
- Testing Strategy: ✓ 39 tests in events.rs
- All ACs Met: ✓ All 19 acceptance criteria verified

### Gate Status

Gate: PASS → docs/qa/gates/1.4-app-state-capture.yml

### Recommended Status

[✓ Ready for Done]

---

## Dev Agent Record

### Agent Model Used
Claude Opus 4.5 (claude-opus-4-5-20251101)

### Completion Notes
- Added supporting enums: `MediaType`, `SizeCategory`, `EditorTool`, `AIModel`
- Added `SizeCategory::from_bytes()` helper with correct boundary thresholds
- Added `AppStateEvent` enum with 20 variants covering media lifecycle, video playback, editor states, and AI model lifecycle
- Added `AppOperation` enum with 8 variants covering frame operations, AI processing, and video seek
- Replaced placeholder `AppState` variant in `DiagnosticEventKind` with proper `state: AppStateEvent`
- Added new `Operation` variant to `DiagnosticEventKind`
- Added `log_state()` and `log_operation()` methods to both `DiagnosticsHandle` and `DiagnosticsCollector`
- Fixed serde renaming for `AIDeblurProcess` and `AIUpscaleProcess` variants (explicit rename needed due to "AI" prefix)
- All 738 tests pass, clippy clean, formatted

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2026-01-13 | Initial implementation | James (Dev Agent) |

### File List
- `src/diagnostics/events.rs` - Added supporting enums, AppStateEvent, AppOperation, updated DiagnosticEventKind, added tests
- `src/diagnostics/collector.rs` - Added log_state(), log_operation() methods and tests
- `src/diagnostics/mod.rs` - Updated exports to include new types

---
