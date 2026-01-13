# Story 3.2: Collection Status Display

**Epic:** 3 - UI Integration
**Status:** Draft
**Priority:** High
**Estimate:** 2-3 hours
**Depends On:** Story 3.1

---

## Story

**As a** developer,
**I want** to see the current collection status on the Diagnostics screen,
**So that** I know whether diagnostics are active and collecting data.

---

## Acceptance Criteria

1. Status indicator shows: Disabled, Enabled (Collecting), or Error state
2. Visual indicator uses appropriate colors (following design tokens)
3. Status text describes current state clearly
4. Shows collection duration if active (e.g., "Collecting for 5m 32s")
5. Shows buffer fill level (e.g., "Buffer: 847 events")
6. Status updates in real-time (subscription to collector state)
7. Accessible: status is screen-reader friendly

---

## Tasks

- [ ] **Task 1:** Define `CollectionStatus` enum
  - [ ] Disabled, Enabled, Error variants
  - [ ] Include started_at timestamp for Enabled
  - [ ] Include error message for Error

- [ ] **Task 2:** Add status query to DiagnosticsCollector
  - [ ] `get_status()` method
  - [ ] Returns CollectionStatus
  - [ ] Returns buffer event count

- [ ] **Task 3:** Implement status indicator widget
  - [ ] Colored dot/badge (green=enabled, gray=disabled, red=error)
  - [ ] Use `palette::SUCCESS_500`, `palette::GRAY_400`, `palette::ERROR_500`

- [ ] **Task 4:** Implement status text
  - [ ] "Collection: Enabled" / "Collection: Disabled" / "Collection: Error"
  - [ ] Duration format: "Xh Ym Zs" or "Xm Zs"
  - [ ] Use i18n for text

- [ ] **Task 5:** Implement buffer fill display
  - [ ] "Buffer: X events"
  - [ ] Update in real-time

- [ ] **Task 6:** Add subscription for real-time updates
  - [ ] Use Iced subscription pattern
  - [ ] Poll collector status periodically (every 1s)
  - [ ] Or use channel-based notification

- [ ] **Task 7:** Add accessibility attributes
  - [ ] Aria labels for status
  - [ ] Screen reader friendly text

- [ ] **Task 8:** Write unit tests
  - [ ] Status formatting
  - [ ] Duration calculation

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

- Status should update without user interaction
- Duration calculation: `started_at.elapsed()`
- Use existing color tokens for consistency

---

## Testing

### Unit Tests
- Duration formatting
- Status display logic

### Manual Tests
- Real-time status updates
- Color accuracy
- Screen reader testing (if possible)

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
