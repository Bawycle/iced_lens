# Story 2.5: Clipboard Export Implementation

**Epic:** 2 - Anonymization & Export System
**Status:** Draft
**Priority:** High
**Estimate:** 1-2 hours
**Depends On:** Story 2.1, 2.2, 2.3

---

## Story

**As a** developer,
**I want** to copy diagnostic reports to the clipboard,
**So that** I can quickly paste them into Claude Code or other tools.

---

## Acceptance Criteria

1. `export_to_clipboard()` function implemented using `arboard` crate
2. Export applies full anonymization pipeline before copying
3. Clipboard contains the same JSON as file export
4. Works on Linux (X11/Wayland), Windows, and macOS
5. Graceful error handling if clipboard access fails
6. Success/failure feedback provided (returns Result)
7. Manual testing on all three platforms

---

## Tasks

- [ ] **Task 1:** Add `arboard` dependency
  - [ ] Add `arboard = "3.4"` to Cargo.toml
  - [ ] Verify it compiles

- [ ] **Task 2:** Implement `export_to_clipboard()` in `export.rs`
  - [ ] Get report from collector
  - [ ] Apply full anonymization
  - [ ] Serialize to JSON string
  - [ ] Copy to clipboard via arboard

- [ ] **Task 3:** Handle clipboard errors gracefully
  - [ ] Add `ClipboardError` to ExportError enum
  - [ ] Return descriptive error message
  - [ ] Don't panic on clipboard failure

- [ ] **Task 4:** Add Linux-specific handling
  - [ ] arboard handles X11/Wayland differences
  - [ ] Test on both if possible

- [ ] **Task 5:** Write unit test (mock clipboard if needed)
  - [ ] Test function returns success
  - [ ] Test error handling

- [ ] **Task 6:** Document manual testing requirements
  - [ ] Add note about cross-platform testing
  - [ ] List platforms to verify

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

- arboard handles cross-platform differences
- Clipboard may fail on headless systems - handle gracefully
- JSON should be the same as file export

---

## Testing

### Unit Tests
- export_to_clipboard returns Ok
- Error handling for failures

### Manual Tests
- Test paste on Linux, Windows, macOS
- Verify JSON is complete and valid

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
