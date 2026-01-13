# Story 1.5: Warning and Error Capture

**Epic:** 1 - Diagnostics Core & Data Collection
**Status:** Draft
**Priority:** High
**Estimate:** 2-3 hours
**Depends On:** Story 1.1, 1.3

---

## Story

**As a** developer,
**I want** to capture warnings and errors from the application,
**So that** I can see error context in diagnostic reports.

---

## Acceptance Criteria

1. `DiagnosticEvent::Warning` and `DiagnosticEvent::Error` variants added
2. Integration with existing notification system to capture user-visible warnings/errors
3. Integration with log macros or console output capture for internal errors
4. Error events include: timestamp, error type/code, message (sanitized), source module
5. Warnings include: timestamp, warning type, message (sanitized)
6. Sensitive data in error messages is not captured (or marked for anonymization)
7. Unit tests verify error event capture

---

## Tasks

- [ ] **Task 1:** Define `DiagnosticWarning` struct
  - [ ] In `src/diagnostics/events.rs`
  - [ ] Fields: timestamp, warning_type, message, source_module
  - [ ] warning_type as enum or string

- [ ] **Task 2:** Define `DiagnosticError` struct
  - [ ] Fields: timestamp, error_type, error_code (optional), message, source_module
  - [ ] error_type as enum or string

- [ ] **Task 3:** Update `DiagnosticEvent` enum
  - [ ] `Warning(DiagnosticWarning)` variant
  - [ ] `Error(DiagnosticError)` variant

- [ ] **Task 4:** Add `log_warning()` and `log_error()` to DiagnosticsCollector
  - [ ] Accept error/warning data
  - [ ] Sanitize message (remove potential paths/PII)
  - [ ] Non-blocking via channel

- [ ] **Task 5:** Create message sanitizer helper
  - [ ] Remove file paths from messages
  - [ ] Mark for anonymization in Epic 2
  - [ ] Basic regex or string replacement

- [ ] **Task 6:** Write unit tests
  - [ ] Test warning event creation
  - [ ] Test error event creation
  - [ ] Test message sanitization

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

- Integration with notification system in later story
- Message sanitization is basic here; full anonymization in Epic 2
- error_code useful for categorization

---

## Testing

### Unit Tests
- `events.rs`: DiagnosticWarning, DiagnosticError creation
- `collector.rs`: log_warning, log_error
- Sanitizer: path removal

### Integration Tests
- None (notification integration in later story)

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
