# Story 2.3: JSON Schema Definition and Implementation

**Epic:** 2 - Anonymization & Export System
**Status:** Draft
**Priority:** High
**Estimate:** 2-3 hours
**Depends On:** Story 1.6, 2.1, 2.2

---

## Story

**As a** developer,
**I want** a well-defined JSON schema for diagnostic reports,
**So that** reports are consistent and optimized for AI analysis.

---

## Acceptance Criteria

1. JSON schema documented (can be in code comments or separate doc)
2. Schema includes: metadata section, system_info section, events array, summary statistics
3. Metadata: report_id, generated_at, iced_lens_version, collection_duration_ms, event_count
4. System info: os, cpu_model, ram_total_mb, disk_type (without identifying info)
5. Events: array of typed events with consistent timestamp format (ISO 8601)
6. Summary: event counts by type, resource usage min/max/avg
7. All fields use consistent naming (snake_case)
8. Serde attributes configured for clean JSON output

---

## Tasks

- [ ] **Task 1:** Update `DiagnosticReport` structure
  - [ ] Add `summary` field
  - [ ] Ensure all sections present

- [ ] **Task 2:** Implement `ReportSummary` struct
  - [ ] event_counts: HashMap<EventType, usize>
  - [ ] resource_stats: ResourceStats (min/max/avg for CPU, RAM)

- [ ] **Task 3:** Implement `ResourceStats` struct
  - [ ] cpu_min, cpu_max, cpu_avg
  - [ ] ram_min_mb, ram_max_mb, ram_avg_mb

- [ ] **Task 4:** Add summary calculation
  - [ ] Method to compute summary from events
  - [ ] Count events by type
  - [ ] Calculate resource statistics

- [ ] **Task 5:** Configure Serde attributes
  - [ ] `#[serde(rename_all = "snake_case")]` on all types
  - [ ] Skip None fields with `#[serde(skip_serializing_if = "Option::is_none")]`
  - [ ] Use ISO 8601 for timestamps

- [ ] **Task 6:** Document JSON schema
  - [ ] Add module-level doc comment with example JSON
  - [ ] Or create `docs/diagnostics-schema.json`

- [ ] **Task 7:** Write unit tests
  - [ ] Test summary calculation
  - [ ] Test JSON output matches expected schema
  - [ ] Verify snake_case naming

- [ ] **Task 8:** Run validation
  - [ ] `cargo fmt --all`
  - [ ] `cargo clippy --all --all-targets -- -D warnings`
  - [ ] `cargo test`

- [ ] **Task 9:** Commit changes
  - [ ] Stage all changes
  - [ ] Commit with descriptive message following conventional commits
  - [ ] Reference story number in commit message

---

## Dev Notes

- Schema should be LLM-friendly (clear field names, consistent structure)
- Summary provides quick overview without parsing all events
- Reference: `docs/architecture/detailed-design-specifications.md` for schema example

---

## Testing

### Unit Tests
- Summary calculation
- JSON serialization format

### Integration Tests
- Validate exported JSON against schema

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
