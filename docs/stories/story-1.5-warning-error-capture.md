# Story 1.5: Warning and Error Type Enrichment

**Epic:** 1 - Diagnostics Core & Data Collection
**Status:** Completed
**Priority:** High
**Estimate:** 2-3 hours
**Depends On:** Story 1.1

---

## Story

**As a** developer,
**I want** to enrich warning and error diagnostic events with detailed metadata,
**So that** I can see comprehensive error context in diagnostic reports.

---

## Acceptance Criteria

### Type Definitions
1. `WarningEvent` struct defined with: warning_type, message, source_module
2. `ErrorEvent` struct defined with: error_type, error_code (optional), message, source_module
3. `WarningType` and `ErrorType` enums defined for categorization

### Integration with Existing Code
4. `DiagnosticEventKind::Warning` variant updated to use `WarningEvent` struct
5. `DiagnosticEventKind::Error` variant updated to use `ErrorEvent` struct
6. `log_warning()` method updated to accept `WarningEvent`
7. `log_error()` method updated to accept `ErrorEvent`
8. Backward-compatible convenience methods retained for simple string messages

### Sanitization
9. Message sanitizer helper implemented in `src/diagnostics/sanitizer.rs`
10. Sanitizer removes file paths from messages (regex-based)
11. Sanitizer marks potential PII for later anonymization (Epic 2)

### Quality
12. All events include timestamp (via existing `DiagnosticEvent` wrapper)
13. Unit tests verify enriched event creation
14. Unit tests verify sanitization removes paths

---

## Tasks

### Task 1: Define Warning and Error Type Enums
- [x] Create `src/diagnostics/sanitizer.rs` (will also hold enums)
- [x] Define `WarningType` enum:
  - [x] `FileNotFound`, `UnsupportedFormat`, `PermissionDenied`, `NetworkError`, `ConfigurationIssue`, `Other`
- [x] Define `ErrorType` enum:
  - [x] `IoError`, `DecodeError`, `ExportError`, `AIModelError`, `InternalError`, `Other`
- [x] Implement `Serialize`/`Deserialize` for both enums

### Task 2: Define WarningEvent Struct
- [x] In `src/diagnostics/events.rs`
- [x] Fields:
  ```rust
  pub struct WarningEvent {
      pub warning_type: WarningType,
      pub message: String,
      pub source_module: Option<String>,
  }
  ```
- [x] Implement `new()` constructor
- [x] Implement `Serialize`/`Deserialize`

### Task 3: Define ErrorEvent Struct
- [x] In `src/diagnostics/events.rs`
- [x] Fields:
  ```rust
  pub struct ErrorEvent {
      pub error_type: ErrorType,
      pub error_code: Option<String>,
      pub message: String,
      pub source_module: Option<String>,
  }
  ```
- [x] Implement `new()` and `with_code()` constructors
- [x] Implement `Serialize`/`Deserialize`

### Task 4: Update DiagnosticEventKind Variants
- [x] Change `Warning { message: String }` to `Warning { event: WarningEvent }`
- [x] Change `Error { message: String }` to `Error { event: ErrorEvent }`
- [x] Update serde attributes for correct JSON structure

### Task 5: Implement Message Sanitizer
- [x] In `src/diagnostics/sanitizer.rs`
- [x] Function: `sanitize_message(message: &str) -> String`
- [x] Remove Unix paths: `/home/...`, `/Users/...`, `/tmp/...`
- [x] Remove Windows paths: `C:\...`, `D:\...`
- [x] Replace with `<path>` placeholder
- [x] Use lazy_static or once_cell for compiled regex

### Task 6: Update DiagnosticsHandle Methods
- [x] Update `log_warning()` signature:
  ```rust
  pub fn log_warning(&self, event: WarningEvent)
  ```
- [x] Add convenience method `log_warning_simple(&self, message: impl Into<String>)` for backward compatibility
- [x] Update `log_error()` signature:
  ```rust
  pub fn log_error(&self, event: ErrorEvent)
  ```
- [x] Add convenience method `log_error_simple(&self, message: impl Into<String>)` for backward compatibility
- [x] Apply sanitizer to messages before storage

### Task 7: Update Module Exports
- [x] Add `sanitizer` module to `src/diagnostics/mod.rs`
- [x] Export `WarningType`, `ErrorType`, `WarningEvent`, `ErrorEvent`
- [x] Export `sanitize_message` function

### Task 8: Write Unit Tests
- [x] Test `WarningEvent` creation and serialization
- [x] Test `ErrorEvent` creation with and without error_code
- [x] Test `sanitize_message` removes Unix paths
- [x] Test `sanitize_message` removes Windows paths
- [x] Test `sanitize_message` handles messages without paths
- [x] Test `log_warning` and `log_error` apply sanitization

### Task 9: Run Validation
- [x] `cargo fmt --all`
- [x] `cargo clippy --all --all-targets -- -D warnings`
- [x] `cargo test`

### Task 10: Commit Changes
- [x] Stage all changes
- [x] Commit with message: `feat(diagnostics): enrich warning and error events [Story 1.5]`

---

## Dev Notes

### Existing Code to Modify

**`src/diagnostics/events.rs`** - Current placeholder variants:
```rust
// BEFORE (current)
Warning { message: String },
Error { message: String },

// AFTER (this story)
Warning { event: WarningEvent },
Error { event: ErrorEvent },
```

**`src/diagnostics/collector.rs`** - Current simple methods (lines 53-68):
```rust
// BEFORE (current)
pub fn log_warning(&self, message: impl Into<String>) { ... }
pub fn log_error(&self, message: impl Into<String>) { ... }

// AFTER (this story) - enriched + backward-compatible
pub fn log_warning(&self, event: WarningEvent) { ... }
pub fn log_warning_simple(&self, message: impl Into<String>) { ... }
pub fn log_error(&self, event: ErrorEvent) { ... }
pub fn log_error_simple(&self, message: impl Into<String>) { ... }
```

### Sanitizer Regex Patterns

```rust
// Unix paths
r"(/home/[^\s]+|/Users/[^\s]+|/tmp/[^\s]+|/var/[^\s]+)"

// Windows paths
r"([A-Za-z]:\\[^\s]+)"
```

### Integration Deferred

Integration with the notification system (`src/ui/notifications/`) is deferred to Story 1.7. This story only defines the enriched types and sanitization infrastructure.

---

## Testing

### Unit Tests
| Test File | Coverage |
|-----------|----------|
| `events.rs` | WarningEvent, ErrorEvent creation and serde |
| `sanitizer.rs` | Path removal, edge cases |
| `collector.rs` | log_warning, log_error with sanitization |

### Test Cases
1. `WarningEvent::new()` creates with all fields
2. `ErrorEvent::with_code()` includes error_code
3. Serialization produces expected JSON structure
4. Sanitizer removes `/home/user/file.txt` → `<path>`
5. Sanitizer removes `C:\Users\name\file.txt` → `<path>`
6. Sanitizer preserves messages without paths
7. `log_warning` sanitizes message field

---

## Dev Agent Record

### Agent Model Used
Claude Opus 4.5 (claude-opus-4-5-20251101)

### Completion Notes
- Created `src/diagnostics/sanitizer.rs` with `WarningType` and `ErrorType` enums
- Added `WarningEvent` struct with `new()` and `with_source()` constructors
- Added `ErrorEvent` struct with `new()`, `with_code()`, `with_source()`, and `full()` constructors
- Updated `DiagnosticEventKind::Warning` and `Error` variants to use enriched event structs
- Implemented `sanitize_message()` using `LazyLock<Regex>` to remove Unix and Windows paths
- Updated `log_warning()` and `log_error()` to accept event structs and apply sanitization
- Added `log_warning_simple()` and `log_error_simple()` convenience methods for backward compatibility
- Fixed serde rename for `AIModelError` variant (explicit rename needed due to "AI" prefix)
- All 789 tests pass, clippy clean, formatted

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2026-01-13 | Initial implementation | James (Dev Agent) |

### File List
- `src/diagnostics/sanitizer.rs` - NEW: WarningType, ErrorType enums, sanitize_message function
- `src/diagnostics/events.rs` - Added WarningEvent, ErrorEvent structs, updated DiagnosticEventKind
- `src/diagnostics/collector.rs` - Updated log_warning/log_error methods, added _simple variants
- `src/diagnostics/mod.rs` - Updated exports to include new types

---
