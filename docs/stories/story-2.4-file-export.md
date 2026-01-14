# Story 2.4: Anonymized File Export

**Epic:** 2 - Anonymization & Export System
**Status:** Done
**Priority:** High
**Estimate:** 2-3 hours
**Depends On:** Story 1.6, 2.1, 2.2, 2.3

---

## Story

**As a** developer,
**I want** to export anonymized diagnostic reports to a file,
**So that** I can share reports without exposing sensitive information.

---

## Acceptance Criteria

### Export Function
1. `export_to_file()` method added to `DiagnosticsCollector`
2. Default filename format: `iced_lens_diagnostics_YYYYMMDD_HHMMSS.json`
3. File written atomically (temp file + rename) to prevent corruption
4. Returns `Result<PathBuf, ExportError>` with the saved file path

### Anonymization Pipeline
5. `AnonymizationPipeline` struct combines `PathAnonymizer` + `IdentityAnonymizer`
6. Pipeline applied to all string fields in events before export:
   - `UserAction` details
   - `WarningEvent` message
   - `ErrorEvent` message
   - `AppStateEvent` string fields (e.g., `MediaFailed.reason`)
7. Same salt used within a single export (consistent hashing)

### File Dialog Integration
8. `export_with_dialog()` opens native save dialog via `rfd`
9. User can cancel dialog (returns `ExportError::Cancelled`)
10. Default directory: user's documents folder (`dirs::document_dir()`)

### Quality
11. Unit tests verify anonymization is applied before export
12. Integration test verifies file creation and JSON validity

---

## Tasks

### Task 1: Create `AnonymizationPipeline` struct (AC: 5, 7)
- [x] Add to `src/diagnostics/anonymizer.rs`
- [x] Combine `PathAnonymizer` and `IdentityAnonymizer` with shared seed
- [x] Method `anonymize_string(&self, input: &str) -> String`
- [x] Apply identity anonymization (paths already sanitized at collection time)

### Task 2: Create `ExportError` enum (AC: 4, 9)
- [x] Add to `src/diagnostics/export.rs` (new file)
- [x] Variants: `Io(std::io::Error)`, `Serialization(serde_json::Error)`, `Cancelled`
- [x] Implement `std::error::Error` and `Display`

### Task 3: Implement `generate_default_filename()` (AC: 2)
- [x] Format: `iced_lens_diagnostics_YYYYMMDD_HHMMSS.json`
- [x] Use `chrono::Local::now()` for timestamp

### Task 4: Implement `anonymize_event()` helper (AC: 6)
- [x] Takes `SerializableEvent` + `AnonymizationPipeline`
- [x] Returns new `SerializableEvent` with anonymized string fields
- [x] Match on `DiagnosticEventKind` variants to find string fields

### Task 5: Implement `build_anonymized_report()` (AC: 5, 6)
- [x] Add method to `DiagnosticsCollector`
- [x] Create `AnonymizationPipeline` with random seed
- [x] Apply `anonymize_event()` to all events
- [x] Include summary from Story 2.3

### Task 6: Implement atomic file write (AC: 3)
- [x] Write to `{path}.tmp` first
- [x] On success: `std::fs::rename()` to final path
- [x] On failure: remove temp file

### Task 7: Implement `export_to_file()` (AC: 1, 3, 4)
- [x] Add method to `DiagnosticsCollector`
- [x] Parameter: `path: impl AsRef<Path>`
- [x] Call `build_anonymized_report()`
- [x] Serialize to pretty JSON
- [x] Write atomically
- [x] Return `Ok(PathBuf)` or `Err(ExportError)`

### Task 8: Implement `export_with_dialog()` (AC: 8, 9, 10)
- [x] Add method to `DiagnosticsCollector`
- [x] Use `rfd::FileDialog::new().save_file()`
- [x] Set default directory and filename
- [x] Handle `None` (cancelled) → `ExportError::Cancelled`
- [x] Call `export_to_file()` with selected path

### Task 9: Write unit tests (AC: 11)
- [x] Test `AnonymizationPipeline` combines both anonymizers
- [x] Test `anonymize_event()` processes all string fields
- [x] Test `generate_default_filename()` format

### Task 10: Write integration test (AC: 12)
- [x] Export to temp directory
- [x] Verify file exists and JSON is valid
- [x] Verify no raw IPs/usernames in output

### Task 11: Run validation
- [x] `cargo fmt --all`
- [x] `cargo clippy --all --all-targets -- -D warnings`
- [x] `cargo test`

### Task 12: Commit changes
- [x] Stage all changes
- [x] Commit: `feat(diagnostics): add anonymized file export [Story 2.4]`

---

## Dev Notes

### Source Tree

```
src/diagnostics/
├── mod.rs              # MODIFY: export new types
├── anonymizer.rs       # MODIFY: add AnonymizationPipeline
├── export.rs           # NEW: ExportError, file operations
├── collector.rs        # MODIFY: add export methods
├── report.rs           # EXISTING: DiagnosticReport
└── events.rs           # EXISTING: event types for matching
```

### Existing Code

**`collector.rs`** already has:
- `export_json(&self) -> serde_json::Result<String>` - raw export (no anonymization)
- `build_report(&self) -> DiagnosticReport` - builds report from buffer

**`anonymizer.rs`** (from Stories 2.1, 2.2) has:
- `PathAnonymizer` - hashes file paths
- `IdentityAnonymizer` - hashes IPs, domains, usernames

### AnonymizationPipeline Design

```rust
/// Combined anonymization pipeline for export.
pub struct AnonymizationPipeline {
    path_anonymizer: PathAnonymizer,
    identity_anonymizer: IdentityAnonymizer,
}

impl AnonymizationPipeline {
    /// Creates a new pipeline with random salt.
    pub fn new() -> Self {
        let seed = rand::random::<u64>();
        Self {
            path_anonymizer: PathAnonymizer::with_seed(seed),
            identity_anonymizer: IdentityAnonymizer::with_seed(seed),
        }
    }

    /// Anonymizes a string by applying all anonymizers in sequence.
    pub fn anonymize_string(&self, input: &str) -> String {
        // Identity first (IPs, domains, username), then paths
        let step1 = self.identity_anonymizer.anonymize_string(input);
        // PathAnonymizer works on Path, not String - skip for messages
        // Paths in messages are already sanitized to "<path>" by sanitizer.rs
        step1
    }
}
```

### Event Anonymization Pattern

```rust
fn anonymize_event(event: &SerializableEvent, pipeline: &AnonymizationPipeline) -> SerializableEvent {
    let anonymized_kind = match &event.kind {
        DiagnosticEventKind::UserAction { action, details } => {
            DiagnosticEventKind::UserAction {
                action: action.clone(),
                details: details.as_ref().map(|d| pipeline.anonymize_string(d)),
            }
        }
        DiagnosticEventKind::Warning { event: w } => {
            DiagnosticEventKind::Warning {
                event: WarningEvent {
                    message: pipeline.anonymize_string(&w.message),
                    ..w.clone()
                },
            }
        }
        DiagnosticEventKind::Error { event: e } => {
            DiagnosticEventKind::Error {
                event: ErrorEvent {
                    message: pipeline.anonymize_string(&e.message),
                    ..e.clone()
                },
            }
        }
        DiagnosticEventKind::StateChange { event: s } => {
            // Anonymize string fields in AppStateEvent variants
            DiagnosticEventKind::StateChange {
                event: anonymize_state_event(s, pipeline),
            }
        }
        // ResourceSnapshot and Operation don't contain user strings
        other => other.clone(),
    };

    SerializableEvent {
        timestamp_ms: event.timestamp_ms,
        kind: anonymized_kind,
    }
}
```

### File Dialog Usage

```rust
use rfd::FileDialog;

pub fn export_with_dialog(&self) -> Result<PathBuf, ExportError> {
    let default_dir = dirs::document_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let default_name = generate_default_filename();

    let path = FileDialog::new()
        .set_directory(&default_dir)
        .set_file_name(&default_name)
        .add_filter("JSON", &["json"])
        .save_file()
        .ok_or(ExportError::Cancelled)?;

    self.export_to_file(&path)
}
```

### Atomic Write Pattern

```rust
fn write_atomic(path: &Path, content: &str) -> std::io::Result<()> {
    let temp_path = path.with_extension("json.tmp");

    // Write to temp file
    std::fs::write(&temp_path, content)?;

    // Atomic rename
    std::fs::rename(&temp_path, path)?;

    Ok(())
}
```

---

## Testing

### Unit Tests

| Test | Input | Expected Output |
|------|-------|-----------------|
| `pipeline_anonymizes_ip` | `"Error from 192.168.1.1"` | `"Error from <ip:hash>"` |
| `pipeline_anonymizes_username` | `"User john logged in"` | `"User <user:hash> logged in"` (if john is system user) |
| `anonymize_event_warning` | Warning with IP in message | Warning with hashed IP |
| `anonymize_event_preserves_type` | UserAction event | Same action type, anonymized details |
| `generate_filename_format` | Current time | `iced_lens_diagnostics_YYYYMMDD_HHMMSS.json` |
| `atomic_write_creates_file` | Path + content | File exists with content |
| `atomic_write_no_temp_on_success` | Path + content | No `.tmp` file remains |

### Integration Tests

| Test | Verification |
|------|--------------|
| `export_creates_valid_json` | File exists, parses as JSON, has metadata/events/summary |
| `export_anonymizes_content` | No raw IPs (192.x.x.x pattern) in output |
| `export_to_temp_directory` | Works with `tempfile::tempdir()` |

---

## Dev Agent Record

### Agent Model Used
Claude Opus 4.5 (claude-opus-4-5-20251101)

### Completion Notes
- Created `AnonymizationPipeline` in `anonymizer.rs` - combines path and identity anonymizers with shared seed
- Created `export.rs` module with `ExportError`, `generate_default_filename()`, `anonymize_event()`, `write_atomic()`
- Added `export_to_file()` and `export_with_dialog()` to `DiagnosticsCollector`
- Path anonymization not applied in pipeline since paths are already sanitized at collection time
- 19 new unit tests for export functionality, 5 integration tests for end-to-end export
- All 207 diagnostics tests pass, clippy clean, formatted

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2026-01-13 | Story created | PO |
| 2026-01-13 | PO Validation: Complete rewrite - clarified AC4 anonymization, added AnonymizationPipeline design, Task-AC mapping, source tree, code examples | Sarah (PO) |
| 2026-01-14 | Implementation complete | James (Dev) |

### File List
| File | Action | Description |
|------|--------|-------------|
| `src/diagnostics/anonymizer.rs` | Modified | Added AnonymizationPipeline struct |
| `src/diagnostics/export.rs` | Created | ExportError, filename generation, event anonymization, atomic write |
| `src/diagnostics/collector.rs` | Modified | Added export_to_file(), export_with_dialog(), build_anonymized_report() |
| `src/diagnostics/mod.rs` | Modified | Export new types |

---
