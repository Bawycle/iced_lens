# Story 1.6: Basic JSON Export (Debug)

**Epic:** 1 - Diagnostics Core & Data Collection
**Status:** Draft
**Priority:** High
**Estimate:** 2-3 hours
**Depends On:** Story 1.1, 1.2, 1.3, 1.4, 1.5

---

## Story

**As a** developer,
**I want** to export collected data as JSON for validation,
**So that** I can verify the collection pipeline works before adding anonymization.

---

## Acceptance Criteria

1. `DiagnosticReport` struct defined containing: metadata, events, system info
2. Serde serialization implemented for all diagnostic types
3. `export_json()` function generates valid JSON from buffer contents
4. JSON includes: report timestamp, IcedLens version, collection duration, event count
5. Export accessible via debug command or test
6. JSON output is valid and parseable
7. Integration test verifies full pipeline: collect → buffer → export

---

## Tasks

- [ ] **Task 1:** Create `src/diagnostics/report.rs`
  - [ ] Define `ReportMetadata` struct
  - [ ] Define `SystemInfo` struct
  - [ ] Define `DiagnosticReport` struct

- [ ] **Task 2:** Implement `ReportMetadata`
  - [ ] report_id (UUID or random string)
  - [ ] generated_at (ISO 8601 timestamp)
  - [ ] iced_lens_version (from Cargo.toml)
  - [ ] collection_duration_ms
  - [ ] event_count

- [ ] **Task 3:** Implement `SystemInfo`
  - [ ] os, os_version
  - [ ] cpu_model, cpu_cores
  - [ ] ram_total_mb
  - [ ] disk_type (if detectable)

- [ ] **Task 4:** Add Serde derives to all event types
  - [ ] `#[derive(Serialize, Deserialize)]` on all structs/enums
  - [ ] `#[serde(rename_all = "snake_case")]` for consistent naming

- [ ] **Task 5:** Implement `export_json()` in collector
  - [ ] Build DiagnosticReport from buffer
  - [ ] Serialize to JSON string
  - [ ] Return Result<String, Error>

- [ ] **Task 6:** Write integration test
  - [ ] Create collector
  - [ ] Add sample events
  - [ ] Export JSON
  - [ ] Verify JSON is valid and contains expected fields

- [ ] **Task 7:** Run validation
  - [ ] `cargo fmt --all`
  - [ ] `cargo clippy --all --all-targets -- -D warnings`
  - [ ] `cargo test`

- [ ] **Task 8:** Commit changes
  - [ ] Stage all changes
  - [ ] Commit with descriptive message following conventional commits
  - [ ] Reference story number in commit message

---

## Dev Notes

- No anonymization yet - that's Epic 2
- JSON should be pretty-printed for debugging
- Use `env!("CARGO_PKG_VERSION")` for version
- `chrono` crate already in project for timestamps

---

## Testing

### Unit Tests
- `report.rs`: struct creation, serialization

### Integration Tests
- Full pipeline: collect → buffer → export → validate JSON

---

## Dev Agent Record

### Agent Model Used
<!-- Record which AI model completed this story -->

### Completion Notes
<!-- Dev agent adds notes here during implementation -->

### Change Log
| Date | Change | Author |
|------|--------|--------|

### File List
<!-- Files created or modified -->

---
