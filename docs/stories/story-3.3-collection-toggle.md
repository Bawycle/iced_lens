# Story 3.3: Collection Toggle Control

**Epic:** 3 - UI Integration
**Status:** Draft
**Priority:** High
**Estimate:** 2-3 hours
**Depends On:** Story 3.1, 3.2

---

## Story

**As a** developer,
**I want** to enable/disable diagnostic collection from the UI,
**So that** I can control when data is being collected.

---

## Acceptance Criteria

1. Toggle switch component for enabling/disabling collection
2. Toggle follows existing IcedLens toggle style
3. Toggling sends appropriate message to DiagnosticsCollector
4. UI reflects state change immediately
5. Toggle state persists across screen navigation (but not app restart for MVP)
6. Clear label indicates toggle purpose
7. Keyboard accessible (Space to toggle when focused)

---

## Tasks

- [ ] **Task 1:** Add toggle state to App state
  - [ ] `diagnostics_enabled: bool` field
  - [ ] Initialize as false (disabled by default)

- [ ] **Task 2:** Implement toggle widget in diagnostics_screen
  - [ ] Use iced::widget::toggler or custom
  - [ ] Match existing IcedLens toggle style
  - [ ] Label: "Enable Collection" (i18n)

- [ ] **Task 3:** Add toggle message
  - [ ] `Message::ToggleCollection` in diagnostics_screen
  - [ ] Handle in update()
  - [ ] Emit event to parent

- [ ] **Task 4:** Connect to DiagnosticsCollector
  - [ ] Start collection when enabled
  - [ ] Stop collection when disabled
  - [ ] Clear buffer on disable (optional)

- [ ] **Task 5:** Persist state during session
  - [ ] State survives screen navigation
  - [ ] State resets on app restart (MVP)

- [ ] **Task 6:** Add keyboard accessibility
  - [ ] Space key toggles when focused
  - [ ] Tab navigation includes toggle

- [ ] **Task 7:** Add i18n keys
  - [ ] "diagnostics-toggle-label"
  - [ ] "diagnostics-enable" / "diagnostics-disable"

- [ ] **Task 8:** Write unit tests
  - [ ] Toggle state changes
  - [ ] Message handling

- [ ] **Task 9:** Run validation
  - [ ] `cargo fmt --all`
  - [ ] `cargo clippy --all --all-targets -- -D warnings`
  - [ ] `cargo test`

- [ ] **Task 10:** Commit changes
  - [ ] Stage all changes
  - [ ] Commit with descriptive message following conventional commits
  - [ ] Reference story number in commit message

---

## Dev Notes

- Iced provides `toggler` widget
- Style should match existing toggles in Settings
- Consider confirmation dialog for disable (clear buffer?)

---

## Testing

### Unit Tests
- Toggle message handling
- State persistence

### Manual Tests
- Toggle visual feedback
- Keyboard accessibility
- State survives navigation

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
