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
**I want** to collect system resource metrics (CPU, RAM, disk I/O) at regular intervals,
**So that** I can correlate performance issues with resource usage.

**Acceptance Criteria:**
1. `ResourceCollector` struct implemented using `sysinfo` crate (or similar)
2. Collects: CPU usage %, RAM usage (used/total), disk read/write bytes
3. Sampling runs on a separate thread to avoid blocking UI
4. Configurable sampling interval (default: 1 second)
5. Each sample stored as `DiagnosticEvent::ResourceSnapshot` with timestamp
6. Collector can be started/stopped via channel commands
7. Cross-platform compatibility verified (Linux, Windows, macOS)
8. Unit tests verify metric collection and thread safety

## Story 1.3: User Action Event Capture

**As a** developer,
**I want** to capture user actions as diagnostic events,
**So that** I can understand what the user was doing when issues occurred.

**Acceptance Criteria:**
1. `UserAction` enum defined for trackable actions (navigate_next, navigate_prev, load_media, seek_video, toggle_play, open_settings, etc.)
2. Integration points identified in existing message handlers
3. `DiagnosticsCollector::log_action()` method implemented
4. Actions stored as `DiagnosticEvent::UserAction` with timestamp and action details
5. Action logging does not block UI thread (uses channel to send to collector)
6. At least 5 key user actions instrumented as proof of concept
7. Unit tests verify action event creation and storage

## Story 1.4: Application State and Operation Capture

**As a** developer,
**I want** to capture application state changes and internal operations,
**So that** I can understand what the application was doing during issues.

**Acceptance Criteria:**
1. `AppStateEvent` enum defined for key states (media_loading_started, media_loaded, media_failed, video_playing, video_paused, video_seeking, etc.)
2. `AppOperation` enum defined for internal operations (decode_frame, resize_image, apply_filter, etc.)
3. Integration with existing state management to capture transitions
4. Events stored with timestamp and relevant context (e.g., media type, file size category)
5. At least 3 key state transitions and 3 operations instrumented
6. Does not capture sensitive data (paths anonymized or excluded at this stage)
7. Unit tests verify state event creation

## Story 1.5: Warning and Error Capture

**As a** developer,
**I want** to capture warnings and errors from the application,
**So that** I can see error context in diagnostic reports.

**Acceptance Criteria:**
1. `DiagnosticEvent::Warning` and `DiagnosticEvent::Error` variants added
2. Integration with existing notification system to capture user-visible warnings/errors
3. Integration with log macros or console output capture for internal errors
4. Error events include: timestamp, error type/code, message (sanitized), source module
5. Warnings include: timestamp, warning type, message (sanitized)
6. Sensitive data in error messages is not captured (or marked for anonymization)
7. Unit tests verify error event capture

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

---
