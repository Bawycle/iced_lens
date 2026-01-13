# Epic 1: Diagnostics Core & Data Collection

**Goal:** Establish the diagnostics module foundation with circular buffer, system metrics collection, event capture, and basic JSON export capability. This epic delivers the core data collection infrastructure that can be validated through tests and debug logging before UI integration.

## Story 1.1: Module Structure and Circular Buffer

**As a** developer,
**I want** a diagnostics module with a circular buffer for storing events,
**So that** I have a foundation for collecting and storing diagnostic data with bounded memory usage.

**Acceptance Criteria:**
1. New `src/diagnostics/` module created with proper module structure
2. `DiagnosticEvent` enum defined to represent different event types (resource snapshot, user action, app state, error)
3. `CircularBuffer<T>` implemented with configurable capacity
4. Buffer correctly overwrites oldest entries when full
5. Buffer provides iterator access to all stored events in chronological order
6. Unit tests verify buffer behavior (add, overflow, iteration)
7. Module exports public API through `mod.rs`

## Story 1.2: System Resource Metrics Collection

**As a** developer,
**I want** to collect system resource metrics (CPU, RAM, disk usage) at regular intervals,
**So that** I can correlate performance issues with resource usage.

**Acceptance Criteria:**
1. `ResourceCollector` struct implemented using `sysinfo` crate (or similar)
2. Collects: CPU usage %, RAM usage (used/total), disk space metrics (used as proxy for I/O activity)
3. Sampling runs on a separate thread to avoid blocking UI
4. Configurable sampling interval (default: 1 second)
5. Each sample stored as `DiagnosticEvent::ResourceSnapshot` with timestamp
6. Collector can be started/stopped via channel commands
7. Cross-platform compatibility verified (Linux, Windows, macOS)
8. Unit tests verify metric collection and thread safety

**Note:** True disk I/O tracking (read/write bytes) would require platform-specific APIs (`/proc/diskstats` on Linux). Current implementation uses disk space as a proxy, which is sufficient for general diagnostics.

## Story 1.3: User Action Event Type Definitions

**As a** developer,
**I want** to define types for capturing user actions as diagnostic events,
**So that** I have a complete type system for tracking user behavior.

**Acceptance Criteria:**
1. `UserAction` enum defined for trackable actions (navigate_next, navigate_prev, load_media, seek_video, toggle_play, open_settings, etc.)
2. Integration points identified in existing message handlers (documented, not implemented)
3. `DiagnosticsCollector::log_action()` method implemented
4. Actions stored as `DiagnosticEvent::UserAction` with timestamp and action details
5. Action logging does not block UI thread (uses channel to send to collector)
6. Unit tests verify action event creation and storage

**Note:** This story defines types only. Actual instrumentation in handlers is deferred to Story 1.7.

## Story 1.4: Application State and Operation Type Definitions

**As a** developer,
**I want** to define comprehensive types for application state changes, internal operations, and AI processing lifecycle,
**So that** I have a complete type system for capturing diagnostic events.

**Acceptance Criteria:**
1. Supporting enums defined: `MediaType`, `SizeCategory`, `EditorTool`, `AIModel`
2. `AppStateEvent` enum defined with comprehensive state variants:
   - Media lifecycle: LoadingStarted, Loaded, Failed
   - Video playback: Playing, Paused, Seeking, Buffering, Error, AtEndOfStream, LoopToggled, SpeedChanged
   - Editor: Opened, Closed, DeblurStarted/Progress/Completed/Cancelled, UnsavedChanges
   - AI Models: DownloadStarted, DownloadCompleted, DownloadFailed
3. `AppOperation` enum defined for internal operations with duration tracking:
   - Original: DecodeFrame, ResizeImage, ApplyFilter, ExportFrame, LoadMetadata
   - AI operations: AIDeblurProcess, AIUpscaleProcess
   - Video: VideoSeek
4. `DiagnosticEventKind` expanded with `AppState` and `Operation` variants
5. `log_state()` and `log_operation()` methods added to `DiagnosticsCollector`
6. `SizeCategory` helper function for privacy-preserving file size classification
7. Does not capture sensitive data (paths excluded, messages sanitized)
8. Unit tests verify all enum variants and helper functions

**Note:** This story defines types only. Actual instrumentation in handlers is deferred to Story 1.7.

## Story 1.5: Warning and Error Type Enrichment

**As a** developer,
**I want** to enrich warning and error diagnostic events with detailed metadata,
**So that** I can see comprehensive error context in diagnostic reports.

**Acceptance Criteria:**
1. `WarningEvent` struct defined with: warning_type, message, source_module
2. `ErrorEvent` struct defined with: error_type, error_code (optional), message, source_module
3. `DiagnosticEventKind::Warning` and `::Error` variants updated to use enriched structs
4. `log_warning()` and `log_error()` methods updated to accept enriched data
5. Message sanitizer helper implemented (removes file paths, potential PII)
6. Sensitive data in messages is sanitized before storage
7. Unit tests verify enriched event creation and sanitization

**Note:** Basic Warning/Error variants and logging methods already exist as placeholders from Story 1.1. This story enriches them. Integration with notification system deferred to Story 1.7.

## Story 1.6: Basic JSON Export (Debug)

**As a** developer,
**I want** to export collected data as JSON for validation,
**So that** I can verify the collection pipeline works before adding anonymization.

**Acceptance Criteria:**
1. `DiagnosticReport` struct defined containing: metadata, events, system info
2. Serde serialization implemented for all diagnostic types
3. `export_json()` function generates valid JSON from buffer contents
4. JSON includes: report timestamp, IcedLens version, collection duration, event count
5. Export accessible via debug command or test
6. JSON output is valid and parseable
7. Integration test verifies full pipeline: collect → buffer → export

## Story 1.7: Diagnostic Event Instrumentation

**As a** developer,
**I want** to instrument application handlers to emit diagnostic events (user actions, state changes, operations),
**So that** the diagnostic system captures real application behavior.

**Acceptance Criteria:**

### User Action Instrumentation
1. At least 5 key user actions instrumented as proof of concept:
   - Navigation (NavigateNext, NavigatePrevious)
   - Media loading (LoadMedia with source context)
   - Playback controls (TogglePlayback, SeekVideo)
   - Screen navigation (OpenSettings, EnterEditor)
2. `DiagnosticsHandle` passed to relevant components

### State Transition Instrumentation
3. At least 3 key state transitions instrumented:
   - Video playback state changes (play/pause/seek/buffering/error)
   - Media loading lifecycle (started/loaded/failed)
   - Editor session (opened/closed)

### Operation Instrumentation
4. At least 3 operations instrumented with duration tracking:
   - AI deblur processing
   - AI upscale processing
   - Video seek operation

### Warning/Error Capture Integration
5. Integration with notification system to capture user-visible warnings/errors
6. Warnings and errors from notification toasts logged as diagnostic events

### Integration Quality
7. Instrumentation integrated at appropriate handler locations:
   - `src/app/update.rs` for user actions and media lifecycle
   - `src/video_player/` for video states and operations
   - `src/ui/image_editor/` for editor states and AI operations
   - `src/ui/notifications/` for warning/error capture
8. All instrumentation is non-blocking (uses existing channel mechanism)
9. Duration tracking uses `Instant` for accurate measurements
10. Integration tests verify events are captured during real operations
11. No performance regression from instrumentation (< 1ms overhead per event)

**Depends On:** Story 1.3, Story 1.4, Story 1.5

---
