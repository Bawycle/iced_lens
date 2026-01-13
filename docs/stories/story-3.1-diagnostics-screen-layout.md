# Story 3.1: Diagnostics Screen Layout and Navigation

**Epic:** 3 - UI Integration
**Status:** Draft
**Priority:** High
**Estimate:** 3-4 hours
**Depends On:** Epic 1, Epic 2

---

## Story

**As a** developer,
**I want** a Diagnostics screen accessible from the hamburger menu,
**So that** I can access diagnostic controls when needed.

---

## Acceptance Criteria

1. New `DiagnosticsScreen` component created in `src/ui/`
2. Screen added to hamburger menu alongside Settings, Help, About
3. Menu entry uses appropriate icon from action_icons
4. Navigation to/from Diagnostics screen works correctly
5. Screen follows existing IcedLens layout patterns (header, content area)
6. Back navigation returns to previous screen
7. Keyboard navigation works (Tab, Enter, Escape)

---

## Tasks

- [ ] **Task 1:** Add `Screen::Diagnostics` variant
  - [ ] Update `src/app/screen.rs`
  - [ ] Add Diagnostics to Screen enum

- [ ] **Task 2:** Create `src/ui/diagnostics_screen.rs`
  - [ ] Define `Message` enum for screen
  - [ ] Define `Event` enum for parent communication
  - [ ] Define `ViewContext` struct
  - [ ] Follow pattern from `src/ui/about.rs`

- [ ] **Task 3:** Implement basic `view()` function
  - [ ] Back button (like About screen)
  - [ ] Title "Diagnostics"
  - [ ] Placeholder content area
  - [ ] Use design tokens for styling

- [ ] **Task 4:** Implement `update()` function
  - [ ] Handle BackToViewer message
  - [ ] Return appropriate Event

- [ ] **Task 5:** Update `src/ui/mod.rs`
  - [ ] Add `pub mod diagnostics_screen;`
  - [ ] Export module

- [ ] **Task 6:** Add to hamburger menu
  - [ ] Update `src/ui/navbar.rs`
  - [ ] Add Diagnostics menu item
  - [ ] Add icon (use existing or create in action_icons)

- [ ] **Task 7:** Integrate in App
  - [ ] Update `src/app/message.rs` - add `Diagnostics(diagnostics_screen::Message)`
  - [ ] Update `src/app/update.rs` - handle Diagnostics messages
  - [ ] Update `src/app/view.rs` - render screen when active

- [ ] **Task 8:** Add i18n keys
  - [ ] Add keys for Diagnostics screen title, menu item
  - [ ] Update `.ftl` files

- [ ] **Task 9:** Test navigation
  - [ ] Menu → Diagnostics works
  - [ ] Back → Viewer works
  - [ ] Keyboard navigation works

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

- Follow existing screen patterns (About, Help, Settings)
- Use `typography::TITLE_LG` for title
- Back button same style as other screens
- Icon suggestion: chart/graph icon for diagnostics

---

## Testing

### Unit Tests
- Message/Event handling

### Manual Tests
- Navigation flow
- Keyboard accessibility
- Visual consistency

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
