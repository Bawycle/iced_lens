# IcedLens Diagnostics Tool - Product Requirements Document (PRD)

**Version:** 1.0
**Date:** 2026-01-13
**Author:** Product Manager (BMAD Method)
**Status:** Draft

---

## Goals and Background Context

### Goals

- Enable developers to diagnose performance issues with precise, contextual data
- Provide zero-friction data collection that runs automatically in the background
- Maintain IcedLens's privacy-first principles through comprehensive anonymization
- Export AI-ready JSON reports compatible with Claude Code analysis workflow
- Reduce average time to diagnose performance issues by 50%
- Keep collection overhead below 1% CPU/RAM impact

### Background Context

IcedLens is a privacy-first media viewer and editor with growing complexity (AI deblur, upscaling, video playback). As features expand, performance issues become harder to diagnose. Currently, when users report problems like "it's slow" or "it freezes," developers lack the precise data needed for effective troubleshooting.

The Diagnostics Tool addresses this gap by automatically collecting performance metrics, user actions, and application states in a circular buffer. When issues occur, developers can export an anonymized JSON report and use AI assistants like Claude Code to analyze root causes. This approach respects privacy (no data leaves the device without explicit user action) while enabling data-driven debugging.

### Change Log

| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2026-01-13 | 1.0 | Initial PRD creation | PM |

---

## Requirements

### Functional Requirements

- **FR1:** The system shall continuously collect system resource metrics (CPU usage, RAM usage, disk I/O) in lightweight mode with configurable sampling interval
- **FR2:** The system shall log user actions (navigation, media loading, video controls, settings changes) with timestamps
- **FR3:** The system shall capture application state changes and internal operations (media loading states, video player states, editor states)
- **FR4:** The system shall capture warnings and errors from notifications and console output
- **FR5:** The system shall store collected data in a circular buffer with a default capacity of 10 minutes of history
- **FR6:** The system shall anonymize file paths by hashing while preserving file extensions
- **FR7:** The system shall anonymize IP addresses, domain names, and usernames in collected data
- **FR8:** The system shall export collected data as structured JSON with a well-defined schema
- **FR9:** The system shall provide file export functionality to save JSON reports to disk
- **FR10:** The system shall provide clipboard export functionality to copy JSON reports
- **FR11:** The system shall provide a dedicated Diagnostics screen accessible from the hamburger menu
- **FR12:** The Diagnostics screen shall display current collection status (enabled/disabled, lightweight mode active)
- **FR13:** The Diagnostics screen shall provide a toggle to enable/disable data collection
- **FR14:** The Diagnostics screen shall provide buttons to export reports (file and clipboard)
- **FR15:** The system shall preserve hardware configuration data (OS, CPU model, RAM size, disk type) without anonymization for diagnostic value

### Non-Functional Requirements

- **NFR1:** Collection overhead shall not exceed 1% CPU usage and 1% additional RAM consumption during lightweight mode
- **NFR2:** The circular buffer shall use no more than 10MB of memory by default
- **NFR3:** Export operation shall complete in under 2 seconds for a full buffer
- **NFR4:** All anonymization shall be one-way (non-reversible) using cryptographic hashing
- **NFR5:** The system shall work consistently across Linux, Windows, and macOS platforms
- **NFR6:** The Diagnostics screen shall follow existing IcedLens UI patterns and design tokens
- **NFR7:** Collection shall run on a separate thread to avoid blocking the UI
- **NFR8:** JSON output shall be valid, parseable, and optimized for LLM consumption
- **NFR9:** No data shall be transmitted over the network; all exports are local and user-initiated
- **NFR10:** The system shall gracefully degrade if system metrics are unavailable on a platform

---

## User Interface Design Goals

### Overall UX Vision

The Diagnostics tool should be unobtrusive during normal use and easily accessible when needed. The interface should feel like a natural extension of IcedLens, following the existing design language. Users (developers/contributors) should be able to understand the collection status at a glance and export reports with minimal friction.

### Key Interaction Paradigms

- **Passive by default:** Collection runs silently when enabled; no user attention required
- **On-demand interaction:** Users access the Diagnostics screen only when they need to check status or export
- **One-click exports:** Both file and clipboard exports should be single-action operations
- **Clear status indication:** Visual feedback on whether collection is active and what mode it's in

### Core Screens and Views

1. **Diagnostics Screen** (new)
   - Collection status indicator (enabled/disabled, mode)
   - Toggle switch to enable/disable collection
   - Export to file button
   - Copy to clipboard button
   - Brief explanation of what data is collected

2. **Hamburger Menu** (modification)
   - Add "Diagnostics" entry alongside Settings, Help, About

### Accessibility

**WCAG AA** - Following IcedLens's existing accessibility standards:
- Sufficient contrast for status indicators
- Keyboard navigable controls
- Clear focus states
- Screen reader compatible status announcements

### Branding

Follow existing IcedLens design system:
- Use design tokens from `design_tokens.rs`
- Match existing button and toggle styles
- Use action icons from `action_icons.rs` for new icons
- Consistent spacing and typography

### Target Device and Platforms

**Desktop Only** - Linux, Windows, macOS (same as IcedLens core application)

---

## Technical Assumptions

### Repository Structure

**Monorepo** - The Diagnostics module will be added to the existing IcedLens repository as a new module under `src/diagnostics/`.

### Service Architecture

**Monolith with modular design** - IcedLens is a single desktop application. The Diagnostics module will integrate as a new domain following the existing Elm/Iced architecture pattern:
- Own message types and state
- Effect-based communication with other modules
- Separate collector thread for non-blocking data collection

### Testing Requirements

**Unit + Integration testing:**
- Unit tests for anonymization functions (verify hashing, extension preservation)
- Unit tests for circular buffer operations
- Unit tests for JSON serialization
- Integration tests for collection pipeline
- Manual testing for UI integration and cross-platform behavior

### Additional Technical Assumptions and Requests

- Use `sysinfo` crate or similar for cross-platform system metrics
- Use existing `serde` and `serde_json` for serialization
- Use `blake3` or `sha2` for fast, secure hashing
- Use `arboard` crate for cross-platform clipboard access
- Follow existing newtype patterns for bounded values (e.g., buffer size limits)
- Integrate with existing message/event system for action logging
- Use existing notification system integration for error capture
- The collector should use channels (e.g., `crossbeam-channel`) for thread communication

---

## Epic List

| Epic | Title | Goal |
|------|-------|------|
| **Epic 1** | Diagnostics Core & Data Collection | Establish the diagnostics module foundation with circular buffer, system metrics collection, event capture, and basic JSON export for validation |
| **Epic 2** | Anonymization & Export System | Implement comprehensive anonymization pipeline and polished export functionality (file + clipboard) with well-defined JSON schema |
| **Epic 3** | UI Integration | Add the Diagnostics screen to the application with status display, controls, and export buttons following IcedLens design patterns |

**Rationale:** Three epics provide clean separation of concerns while each delivering testable incremental value. Epic 1 delivers the core infrastructure that can be validated programmatically. Epic 2 adds the privacy layer and export mechanisms. Epic 3 completes the user-facing integration.

---

## Epic 1: Diagnostics Core & Data Collection

**Goal:** Establish the diagnostics module foundation with circular buffer, system metrics collection, event capture, and basic JSON export capability. This epic delivers the core data collection infrastructure that can be validated through tests and debug logging before UI integration.

### Story 1.1: Module Structure and Circular Buffer

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

### Story 1.2: System Resource Metrics Collection

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

### Story 1.3: User Action Event Capture

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

### Story 1.4: Application State and Operation Capture

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

### Story 1.5: Warning and Error Capture

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

### Story 1.6: Basic JSON Export (Debug)

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

## Epic 2: Anonymization & Export System

**Goal:** Implement comprehensive anonymization to ensure privacy compliance, and deliver polished export functionality with a well-defined JSON schema optimized for AI analysis.

### Story 2.1: Path Anonymization

**As a** developer,
**I want** file paths to be anonymized in diagnostic reports,
**So that** user privacy is protected while preserving diagnostic value.

**Acceptance Criteria:**
1. `PathAnonymizer` struct implemented with hash-based anonymization
2. File paths hashed using blake3 or sha256 (fast, secure)
3. File extensions preserved (e.g., `/home/user/photos/image.jpg` → `a1b2c3d4.jpg`)
4. Directory structure depth preserved but names hashed (e.g., `hash1/hash2/hash3.jpg`)
5. Consistent hashing within a session (same path = same hash)
6. Hash is one-way (cannot reverse to original path)
7. Unit tests verify anonymization correctness and consistency

### Story 2.2: Network and Identity Anonymization

**As a** developer,
**I want** IP addresses, domain names, and usernames anonymized,
**So that** no personally identifiable information appears in reports.

**Acceptance Criteria:**
1. IP address detection and hashing implemented (IPv4 and IPv6 patterns)
2. Domain name detection and hashing implemented
3. Username detection based on common patterns and system username
4. `IdentityAnonymizer` struct with methods for each type
5. Anonymization applied to all string fields in events
6. Original values cannot be recovered from hashes
7. Unit tests with various input patterns verify detection and anonymization

### Story 2.3: JSON Schema Definition and Implementation

**As a** developer,
**I want** a well-defined JSON schema for diagnostic reports,
**So that** reports are consistent and optimized for AI analysis.

**Acceptance Criteria:**
1. JSON schema documented (can be in code comments or separate doc)
2. Schema includes: metadata section, system_info section, events array, summary statistics
3. Metadata: report_id, generated_at, iced_lens_version, collection_duration_ms, event_count
4. System info: os, cpu_model, ram_total_mb, disk_type (without identifying info)
5. Events: array of typed events with consistent timestamp format (ISO 8601)
6. Summary: event counts by type, resource usage min/max/avg
7. All fields use consistent naming (snake_case)
8. Serde attributes configured for clean JSON output

### Story 2.4: File Export Implementation

**As a** developer,
**I want** to export diagnostic reports to a file,
**So that** I can save reports for later analysis or sharing.

**Acceptance Criteria:**
1. `export_to_file()` function implemented
2. Default filename format: `iced_lens_diagnostics_YYYYMMDD_HHMMSS.json`
3. User can choose save location via native file dialog (if available) or default to user's documents/downloads
4. Export applies full anonymization pipeline before writing
5. File is written atomically (temp file + rename) to prevent corruption
6. Success/failure feedback provided (returns Result)
7. Integration test verifies file creation and content validity

### Story 2.5: Clipboard Export Implementation

**As a** developer,
**I want** to copy diagnostic reports to the clipboard,
**So that** I can quickly paste them into Claude Code or other tools.

**Acceptance Criteria:**
1. `export_to_clipboard()` function implemented using `arboard` crate
2. Export applies full anonymization pipeline before copying
3. Clipboard contains the same JSON as file export
4. Works on Linux (X11/Wayland), Windows, and macOS
5. Graceful error handling if clipboard access fails
6. Success/failure feedback provided (returns Result)
7. Manual testing on all three platforms

---

## Epic 3: UI Integration

**Goal:** Add the Diagnostics screen to IcedLens, providing users with a clean interface to view collection status, toggle collection, and export reports.

### Story 3.1: Diagnostics Screen Layout and Navigation

**As a** developer,
**I want** a Diagnostics screen accessible from the hamburger menu,
**So that** I can access diagnostic controls when needed.

**Acceptance Criteria:**
1. New `DiagnosticsScreen` component created in `src/ui/`
2. Screen added to hamburger menu alongside Settings, Help, About
3. Menu entry uses appropriate icon from action_icons
4. Navigation to/from Diagnostics screen works correctly
5. Screen follows existing IcedLens layout patterns (header, content area)
6. Back navigation returns to previous screen
7. Keyboard navigation works (Tab, Enter, Escape)

### Story 3.2: Collection Status Display

**As a** developer,
**I want** to see the current collection status on the Diagnostics screen,
**So that** I know whether diagnostics are active and collecting data.

**Acceptance Criteria:**
1. Status indicator shows: Disabled, Enabled (Collecting), or Error state
2. Visual indicator uses appropriate colors (following design tokens)
3. Status text describes current state clearly
4. Shows collection duration if active (e.g., "Collecting for 5m 32s")
5. Shows buffer fill level (e.g., "Buffer: 847 events")
6. Status updates in real-time (subscription to collector state)
7. Accessible: status is screen-reader friendly

### Story 3.3: Collection Toggle Control

**As a** developer,
**I want** to enable/disable diagnostic collection from the UI,
**So that** I can control when data is being collected.

**Acceptance Criteria:**
1. Toggle switch component for enabling/disabling collection
2. Toggle follows existing IcedLens toggle style
3. Toggling sends appropriate message to DiagnosticsCollector
4. UI reflects state change immediately
5. Toggle state persists across screen navigation (but not app restart for MVP)
6. Clear label indicates toggle purpose
7. Keyboard accessible (Space to toggle when focused)

### Story 3.4: Export Buttons and Feedback

**As a** developer,
**I want** buttons to export reports to file and clipboard,
**So that** I can easily get diagnostic data out of the application.

**Acceptance Criteria:**
1. "Export to File" button implemented with appropriate icon
2. "Copy to Clipboard" button implemented with appropriate icon
3. Buttons disabled when collection is disabled or buffer is empty
4. Clicking triggers respective export function
5. Success feedback shown via toast notification ("Report exported" / "Copied to clipboard")
6. Error feedback shown via toast notification with error description
7. Buttons follow existing IcedLens button styles
8. Buttons are keyboard accessible

### Story 3.5: Information and Help Content

**As a** developer,
**I want** brief explanatory content on the Diagnostics screen,
**So that** users understand what the tool does and what data is collected.

**Acceptance Criteria:**
1. Brief description of the Diagnostics tool purpose (2-3 sentences)
2. Summary of what data is collected (bullet list)
3. Privacy assurance statement (data is anonymized, never sent automatically)
4. Link or reference to documentation for more details (if docs exist)
5. Content is concise and doesn't clutter the interface
6. Text follows existing typography styles
7. Content is translatable (uses i18n system)

---

## Checklist Results Report

*To be completed after PRD review and before implementation.*

---

## Next Steps

### UX Expert Prompt

> Review the IcedLens Diagnostics Tool PRD (`docs/prd.md`), focusing on the UI Design Goals and Epic 3 (UI Integration). Evaluate the proposed Diagnostics screen design for usability, accessibility, and consistency with IcedLens's existing design language. Provide recommendations for the screen layout, component placement, and interaction patterns.

### Architect Prompt

> Review the IcedLens Diagnostics Tool PRD (`docs/prd.md`) and create the technical architecture for the diagnostics module. Focus on: module structure within `src/diagnostics/`, integration points with existing message handlers, thread model for the collector, data structures for events and buffer, and the anonymization pipeline. Ensure the design follows IcedLens's existing patterns (Elm/Iced architecture, newtypes, design tokens).

---

*Generated using the BMAD-METHOD PRD Template v2.0*
