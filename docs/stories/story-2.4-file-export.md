# Story 2.4: File Export Implementation

**Epic:** 2 - Anonymization & Export System
**Status:** Draft
**Priority:** High
**Estimate:** 2-3 hours
**Depends On:** Story 2.1, 2.2, 2.3

---

## Story

**As a** developer,
**I want** to export diagnostic reports to a file,
**So that** I can save reports for later analysis or sharing.

---

## Acceptance Criteria

1. `export_to_file()` function implemented
2. Default filename format: `iced_lens_diagnostics_YYYYMMDD_HHMMSS.json`
3. User can choose save location via native file dialog (if available) or default to user's documents/downloads
4. Export applies full anonymization pipeline before writing
5. File is written atomically (temp file + rename) to prevent corruption
6. Success/failure feedback provided (returns Result)
7. Integration test verifies file creation and content validity

---

## Tasks

- [ ] **Task 1:** Create `src/diagnostics/export.rs`
  - [ ] Define export-related functions
  - [ ] Import anonymizer and report modules

- [ ] **Task 2:** Implement `generate_filename()`
  - [ ] Format: `iced_lens_diagnostics_YYYYMMDD_HHMMSS.json`
  - [ ] Use chrono for timestamp formatting

- [ ] **Task 3:** Implement `export_to_file()` function
  - [ ] Accept optional path (use default if None)
  - [ ] Get report from collector
  - [ ] Apply full anonymization
  - [ ] Serialize to JSON

- [ ] **Task 4:** Implement atomic file write
  - [ ] Write to temp file first
  - [ ] Rename to final path on success
  - [ ] Clean up temp on failure

- [ ] **Task 5:** Add file dialog integration (optional path)
  - [ ] Use `rfd` crate (already in project)
  - [ ] Return selected path or default

- [ ] **Task 6:** Define `ExportError` enum
  - [ ] IoError, SerializationError, DialogCancelled
  - [ ] Implement `std::error::Error`

- [ ] **Task 7:** Write integration test
  - [ ] Export to temp directory
  - [ ] Verify file exists
  - [ ] Verify JSON is valid
  - [ ] Verify anonymization applied

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

- `rfd` already in Cargo.toml for file dialogs
- Default location: `dirs::document_dir()` or `dirs::download_dir()`
- Atomic write prevents corrupted files on crash

---

## Testing

### Unit Tests
- Filename generation
- Atomic write logic

### Integration Tests
- Full export to temp file
- JSON validation

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
