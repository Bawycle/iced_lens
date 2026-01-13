# Story 3.4: Export Buttons and Feedback

**Epic:** 3 - UI Integration
**Status:** Draft
**Priority:** High
**Estimate:** 2-3 hours
**Depends On:** Story 2.4, 2.5, 3.1

---

## Story

**As a** developer,
**I want** buttons to export reports to file and clipboard,
**So that** I can easily get diagnostic data out of the application.

---

## Acceptance Criteria

1. "Export to File" button implemented with appropriate icon
2. "Copy to Clipboard" button implemented with appropriate icon
3. Buttons disabled when collection is disabled or buffer is empty
4. Clicking triggers respective export function
5. Success feedback shown via toast notification ("Report exported" / "Copied to clipboard")
6. Error feedback shown via toast notification with error description
7. Buttons follow existing IcedLens button styles
8. Buttons are keyboard accessible

---

## Tasks

- [ ] **Task 1:** Add export buttons to diagnostics_screen view
  - [ ] "Export to File" button
  - [ ] "Copy to Clipboard" button
  - [ ] Use Row layout with spacing

- [ ] **Task 2:** Add icons to buttons
  - [ ] Add save/download icon to action_icons.rs
  - [ ] Add clipboard icon to action_icons.rs
  - [ ] Use in buttons

- [ ] **Task 3:** Implement button disabled state
  - [ ] Disable when collection disabled
  - [ ] Disable when buffer is empty
  - [ ] Use disabled styling from design tokens

- [ ] **Task 4:** Add export messages
  - [ ] `Message::ExportToFile`
  - [ ] `Message::ExportToClipboard`
  - [ ] Handle in update()

- [ ] **Task 5:** Connect to export functions
  - [ ] Call `export_to_file()` on file button
  - [ ] Call `export_to_clipboard()` on clipboard button
  - [ ] Handle async if needed

- [ ] **Task 6:** Add success notifications
  - [ ] Use existing Notification system
  - [ ] "diagnostics-export-success" i18n key
  - [ ] "diagnostics-clipboard-success" i18n key

- [ ] **Task 7:** Add error notifications
  - [ ] "diagnostics-export-error" i18n key
  - [ ] Include error details in notification

- [ ] **Task 8:** Add i18n keys
  - [ ] Button labels
  - [ ] Success/error messages

- [ ] **Task 9:** Write unit tests
  - [ ] Button disabled logic
  - [ ] Message handling

- [ ] **Task 10:** Run validation
  - [ ] `cargo fmt --all`
  - [ ] `cargo clippy --all --all-targets -- -D warnings`
  - [ ] `cargo test`

- [ ] **Task 11:** Commit changes
  - [ ] Stage all changes
  - [ ] Commit with descriptive message following conventional commits
  - [ ] Reference story number in commit message

---

## Dev Notes

- Follow existing button patterns in IcedLens
- Notifications use existing toast system
- Disabled state should be visually clear

---

## Testing

### Unit Tests
- Disabled state logic
- Message handling

### Manual Tests
- Export to file works
- Copy to clipboard works
- Notifications appear
- Disabled state visual

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
