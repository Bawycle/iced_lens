# Story 1.6: Basic JSON Export (Debug)

**Epic:** 1 - Diagnostics Core & Data Collection
**Status:** Completed
**Priority:** High
**Estimate:** 2-3 hours
**Depends On:** Story 1.1

---

## Story

**As a** developer,
**I want** to export collected data as JSON for validation,
**So that** I can verify the collection pipeline works before adding anonymization.

---

## Acceptance Criteria

### Report Structure
1. `DiagnosticReport` struct defined containing: metadata, events, system info
2. `ReportMetadata` includes: report_id, generated_at, iced_lens_version, collection_duration_ms, event_count
3. `SystemInfo` includes: os, os_version, cpu_cores, ram_total_mb

### Serialization
4. All diagnostic types serializable to JSON via serde
5. `DiagnosticEvent.timestamp` (Instant) converted to relative milliseconds for serialization
6. JSON output uses snake_case naming convention

### Export Function
7. `export_json()` function generates valid JSON from buffer contents
8. JSON output is pretty-printed for debugging readability
9. Function returns `Result<String, Error>` for proper error handling

### Quality
10. JSON output is valid and parseable (verified by test)
11. Integration test verifies full pipeline: collect → buffer → export
12. Export accessible via `DiagnosticsCollector::export_json()` method

---

## Tasks

### Task 1: Create `src/diagnostics/report.rs`
- [x] Create new file
- [x] Add module to `src/diagnostics/mod.rs`

### Task 2: Define `ReportMetadata` Struct
- [x] Fields:
  ```rust
  pub struct ReportMetadata {
      pub report_id: String,           // UUID or random hex string
      pub generated_at: String,        // ISO 8601 timestamp
      pub iced_lens_version: String,   // From env!("CARGO_PKG_VERSION")
      pub collection_started_at: String, // ISO 8601
      pub collection_duration_ms: u64,
      pub event_count: usize,
  }
  ```
- [x] Implement `Serialize`/`Deserialize`

### Task 3: Define `SystemInfo` Struct
- [x] Fields:
  ```rust
  pub struct SystemInfo {
      pub os: String,
      pub os_version: String,
      pub cpu_cores: usize,
      pub ram_total_mb: u64,
  }
  ```
- [x] Implement `SystemInfo::collect()` using `sysinfo` crate
- [x] Implement `Serialize`/`Deserialize`

### Task 4: Define `SerializableEvent` Struct
- [x] Wrapper for `DiagnosticEvent` that is JSON-serializable:
  ```rust
  #[derive(Serialize, Deserialize)]
  pub struct SerializableEvent {
      /// Milliseconds since collection started (relative timestamp)
      pub timestamp_ms: u64,
      /// The event data
      #[serde(flatten)]
      pub kind: DiagnosticEventKind,
  }
  ```
- [x] Implement `From<(&DiagnosticEvent, Instant)>` for conversion
  - First `Instant` is the event timestamp
  - Second `Instant` is the collection start time (for relative calculation)

### Task 5: Define `DiagnosticReport` Struct
- [x] Fields:
  ```rust
  #[derive(Serialize, Deserialize)]
  pub struct DiagnosticReport {
      pub metadata: ReportMetadata,
      pub system_info: SystemInfo,
      pub events: Vec<SerializableEvent>,
  }
  ```
- [x] Implement builder pattern or `new()` constructor

### Task 6: Implement `export_json()` in DiagnosticsCollector
- [x] Add `collection_started_at: Instant` field to `DiagnosticsCollector`
- [x] Initialize in `DiagnosticsCollector::new()`
- [x] Implement method:
  ```rust
  pub fn export_json(&self) -> Result<String, serde_json::Error> {
      let report = self.build_report();
      serde_json::to_string_pretty(&report)
  }
  ```
- [x] Implement `build_report()` helper:
  - Collect system info
  - Convert buffer events to `SerializableEvent`
  - Calculate collection duration
  - Build metadata

### Task 7: Write Unit Tests
- [x] Test `ReportMetadata` serialization
- [x] Test `SystemInfo::collect()` returns valid data
- [x] Test `SerializableEvent` conversion preserves event data
- [x] Test timestamp conversion is correct (relative to start)

### Task 8: Write Integration Test
- [x] Create collector
- [x] Add sample events (ResourceSnapshot, UserAction, Warning, Error)
- [x] Call `export_json()`
- [x] Parse JSON with `serde_json::from_str()`
- [x] Verify all expected fields present
- [x] Verify event count matches

### Task 9: Run Validation
- [x] `cargo fmt --all`
- [x] `cargo clippy --all --all-targets -- -D warnings`
- [x] `cargo test`

### Task 10: Commit Changes
- [x] Stage all changes
- [x] Commit with message: `feat(diagnostics): add JSON export for diagnostic reports [Story 1.6]`

---

## Dev Notes

### Critical: `Instant` is NOT Serializable

`std::time::Instant` does not implement `Serialize` because it's a monotonic clock with no absolute meaning. The solution is to:

1. Store `collection_started_at: Instant` when collector is created
2. Convert event timestamps to **relative milliseconds** since collection start
3. Use `SerializableEvent` wrapper for JSON output

```rust
// Conversion pattern
let relative_ms = event.timestamp
    .duration_since(collection_started_at)
    .as_millis() as u64;
```

### Existing Serde Implementations

These types already have `Serialize`/`Deserialize`:
- `DiagnosticEventKind` ✓
- `UserAction` ✓
- `ResourceMetrics` ✓

These need to be added in this story:
- `SerializableEvent` (new)
- `DiagnosticReport` (new)
- `ReportMetadata` (new)
- `SystemInfo` (new)

### Version and Timestamp Helpers

```rust
// Version from Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

// ISO 8601 timestamp
use chrono::Utc;
let timestamp = Utc::now().to_rfc3339();
```

### JSON Output Example

```json
{
  "metadata": {
    "report_id": "a1b2c3d4",
    "generated_at": "2026-01-13T15:30:00Z",
    "iced_lens_version": "0.1.0",
    "collection_started_at": "2026-01-13T15:25:00Z",
    "collection_duration_ms": 300000,
    "event_count": 42
  },
  "system_info": {
    "os": "linux",
    "os_version": "6.14.0",
    "cpu_cores": 8,
    "ram_total_mb": 16384
  },
  "events": [
    {
      "timestamp_ms": 0,
      "type": "resource_snapshot",
      "metrics": { ... }
    },
    {
      "timestamp_ms": 1000,
      "type": "user_action",
      "action": "navigate_next"
    }
  ]
}
```

### No Anonymization

This story exports raw data. Anonymization (paths, IPs, usernames) is handled in **Epic 2**.

---

## Testing

### Unit Tests
| Test File | Coverage |
|-----------|----------|
| `report.rs` | ReportMetadata, SystemInfo, SerializableEvent creation and serde |

### Integration Tests
| Test | Verification |
|------|--------------|
| Full pipeline | Create collector → add events → export → parse JSON → verify structure |
| Empty buffer | Export with no events produces valid JSON with event_count: 0 |
| Event types | All event kinds serialize correctly |

---

## QA Results

### Review Date: 2026-01-13

### Reviewed By: Quinn (Test Architect)

### Code Quality Assessment

Clean report structure. SerializableEvent handles Instant→ms conversion properly. UUID report IDs and ISO 8601 timestamps. Pretty-printed output for debugging.

### Refactoring Performed

None required.

### Compliance Check

- Coding Standards: ✓
- Project Structure: ✓
- Testing Strategy: ✓ 10 report tests
- All ACs Met: ✓ All 12 acceptance criteria verified

### Gate Status

Gate: PASS → docs/qa/gates/1.6-basic-json-export.yml

### Recommended Status

[✓ Ready for Done]

---

## Dev Agent Record

### Agent Model Used
Claude Opus 4.5 (claude-opus-4-5-20251101)

### Completion Notes
- Created `src/diagnostics/report.rs` with all required structs (`ReportMetadata`, `SystemInfo`, `SerializableEvent`, `DiagnosticReport`)
- Used UUID v4 for report IDs via `uuid` crate (added to dependencies)
- Added `serde_json` to main dependencies (was only in dev-dependencies)
- Added `collection_started_at` (Instant) and `collection_started_at_utc` (DateTime<Utc>) fields to `DiagnosticsCollector`
- Implemented `export_json()` and `build_report()` methods
- Added `PartialEq` derive to `DiagnosticEventKind` for test assertions
- Used `#[allow(clippy::cast_possible_truncation)]` for u128→u64 duration casts (safe as max duration fits in u64)
- All 121 diagnostics tests pass including 3 new integration tests

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2026-01-13 | Initial implementation | Claude Opus 4.5 |
| 2026-01-13 | Code review: Approved | Claude Opus 4.5 |

### File List
- `src/diagnostics/report.rs` (created)
- `src/diagnostics/mod.rs` (modified - added report module and exports)
- `src/diagnostics/collector.rs` (modified - added timestamp fields and export methods)
- `src/diagnostics/events.rs` (modified - added PartialEq to DiagnosticEventKind)
- `Cargo.toml` (modified - added serde_json to dependencies)

---
