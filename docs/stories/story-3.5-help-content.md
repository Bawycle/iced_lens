# Story 3.5: Information and Help Content

**Epic:** 3 - UI Integration
**Status:** Draft
**Priority:** Medium
**Estimate:** 1-2 hours
**Depends On:** Story 3.1

---

## Story

**As a** developer,
**I want** brief explanatory content on the Diagnostics screen,
**So that** users understand what the tool does and what data is collected.

---

## Acceptance Criteria

1. Brief description of the Diagnostics tool purpose (2-3 sentences)
2. Summary of what data is collected (bullet list)
3. Privacy assurance statement (data is anonymized, never sent automatically)
4. Link or reference to documentation for more details (if docs exist)
5. Content is concise and doesn't clutter the interface
6. Text follows existing typography styles
7. Content is translatable (uses i18n system)

---

## Tasks

- [ ] **Task 1:** Add description section to diagnostics_screen
  - [ ] 2-3 sentences explaining purpose
  - [ ] Place below status, above controls

- [ ] **Task 2:** Add data collection summary
  - [ ] Bullet list of what's collected:
    - System resources (CPU, RAM, disk)
    - User actions
    - Application states
    - Warnings and errors
  - [ ] Use `typography::BODY` for text

- [ ] **Task 3:** Add privacy assurance
  - [ ] "All data is anonymized before export"
  - [ ] "Data is never sent automatically"
  - [ ] Use subtle styling (secondary text color)

- [ ] **Task 4:** Add i18n keys
  - [ ] diagnostics-description
  - [ ] diagnostics-data-collected-title
  - [ ] diagnostics-data-item-resources
  - [ ] diagnostics-data-item-actions
  - [ ] diagnostics-data-item-states
  - [ ] diagnostics-data-item-errors
  - [ ] diagnostics-privacy-notice

- [ ] **Task 5:** Update all .ftl translation files
  - [ ] English (en.ftl)
  - [ ] French (fr.ftl) if exists
  - [ ] Other languages as needed

- [ ] **Task 6:** Style content appropriately
  - [ ] Use design tokens for typography
  - [ ] Adequate spacing between sections
  - [ ] Don't overwhelm the interface

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

- Keep content minimal - users are developers who need quick access
- Privacy notice is important for trust
- Follow existing help/about content patterns

---

## Testing

### Unit Tests
- None (static content)

### Manual Tests
- Content displays correctly
- Translations work
- Doesn't clutter interface

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
