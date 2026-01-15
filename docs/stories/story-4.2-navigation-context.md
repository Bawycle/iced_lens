# Story 4.2: Viewer/Editor Navigation Context

**Epic:** 4 - Diagnostics Collection Completeness
**Status:** Ready
**Priority:** P1
**Estimate:** 2-3 hours
**Depends On:** Story 4.0

---

## Story

**As a** developer analyzing navigation patterns,
**I want** navigation events to include Viewer/Editor context,
**So that** I can distinguish navigation behavior between modes.

---

## Acceptance Criteria

1. `NavigationContext` enum defined with `Viewer` and `Editor` variants
2. `NavigateNext` UserAction modified to include: `context: NavigationContext`, `filter_active: bool`, `position_in_filtered: Option<usize>`, `position_in_total: usize`
3. `NavigatePrevious` UserAction modified with same fields
4. Viewer navigation events explicitly marked with `NavigationContext::Viewer`
5. Editor navigation events explicitly marked with `NavigationContext::Editor`
6. Integration tests verify context is correctly captured in both modes
7. Existing navigation tests updated to expect new event structure

---

## Tasks

- [ ] **Task 1:** Define `NavigationContext` enum (AC: 1)
  - [ ] Add to `src/diagnostics/events.rs`
  - [ ] Variants: `Viewer`, `Editor`
  - [ ] Derive `Debug, Clone, Serialize, PartialEq`

- [ ] **Task 2:** Modify `NavigateNext` UserAction (AC: 2)
  - [ ] Change from unit variant to struct variant
  - [ ] Add field: `context: NavigationContext`
  - [ ] Add field: `filter_active: bool`
  - [ ] Add field: `position_in_filtered: Option<usize>`
  - [ ] Add field: `position_in_total: usize`

- [ ] **Task 3:** Modify `NavigatePrevious` UserAction (AC: 3)
  - [ ] Change from unit variant to struct variant
  - [ ] Add same fields as `NavigateNext`

- [ ] **Task 4:** Update Viewer navigation logging (AC: 4)
  - [ ] Location: `handle_navigate_next()` at `src/app/update.rs:1273`
  - [ ] Location: `handle_navigate_previous()` at `src/app/update.rs:1288`
  - [ ] Update existing `log_action()` calls to use struct variant with `NavigationContext::Viewer`
  - [ ] Use `ctx.media_navigator.navigation_info()` for filter and position fields

- [ ] **Task 5:** Add Editor navigation logging (AC: 5)
  - [ ] Location: `handle_editor_navigate_next()` at `src/app/update.rs:915`
  - [ ] Location: `handle_editor_navigate_previous()` at `src/app/update.rs:927`
  - [ ] **Note:** These handlers currently have NO logging - add new `log_action()` calls
  - [ ] Use `NavigationContext::Editor`, `filter_active: false`, `position_in_filtered: None`
  - [ ] Use `ctx.media_navigator.navigation_info()` for position_in_total

- [ ] **Task 6:** Add integration tests for Viewer context (AC: 6)
  - [ ] Test navigation in Viewer captures `NavigationContext::Viewer`
  - [ ] Test filter_active reflects actual filter state
  - [ ] Test positions are correct

- [ ] **Task 7:** Add integration tests for Editor context (AC: 6)
  - [ ] Test navigation in Editor captures `NavigationContext::Editor`
  - [ ] Test filter_active is false or position_in_filtered is None

- [ ] **Task 8:** Update existing navigation tests (AC: 7)
  - [ ] Find tests that assert on NavigateNext/NavigatePrevious
  - [ ] Update assertions to match new struct variant format

- [ ] **Task 9:** Run validation
  - [ ] `cargo fmt --all`
  - [ ] `cargo clippy --all --all-targets -- -D warnings`
  - [ ] `cargo test`

---

## Dev Notes

### Source Tree

```
src/
├── diagnostics/
│   └── events.rs           # MODIFY: Add NavigationContext, modify NavigateNext/Previous
├── app/
│   └── update.rs           # MODIFY: Update navigation logging calls (lines 915-936, 1273-1301)
├── media/
│   └── navigator.rs        # REFERENCE: Position and filter info source (NavigationInfo struct)
```

**Note:** Tests are in-file using `#[cfg(test)]` modules per coding standards.

### Target Handler Locations

| Context | Handler | Line | Current Logging |
|---------|---------|------|-----------------|
| Viewer | `handle_navigate_next` | 1273 | ✅ Logs `UserAction::NavigateNext` |
| Viewer | `handle_navigate_previous` | 1288 | ✅ Logs `UserAction::NavigatePrevious` |
| Editor | `handle_editor_navigate_next` | 915 | ❌ No logging (add in this story) |
| Editor | `handle_editor_navigate_previous` | 927 | ❌ No logging (add in this story) |

### MediaNavigator API

Use `navigation_info()` to get all required data:

```rust
// In navigator.rs - NavigationInfo struct
pub struct NavigationInfo {
    pub has_next: bool,
    pub has_previous: bool,
    pub at_first: bool,
    pub at_last: bool,
    pub current_index: Option<usize>,
    pub total_count: usize,
    pub filtered_count: usize,
    pub filter_active: bool,
}

// Usage:
let info = ctx.media_navigator.navigation_info();
let filter_active = info.filter_active;
let position_in_total = info.current_index.unwrap_or(0);
let position_in_filtered = if filter_active { info.current_index } else { None };
```

### Current Event Structure (Before)

```rust
pub enum UserAction {
    NavigateNext,
    NavigatePrevious,
    // ...
}
```

### Proposed Event Structure (After)

```rust
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum NavigationContext {
    Viewer,
    Editor,
}

pub enum UserAction {
    NavigateNext {
        context: NavigationContext,
        filter_active: bool,
        position_in_filtered: Option<usize>,
        position_in_total: usize,
    },
    NavigatePrevious {
        context: NavigationContext,
        filter_active: bool,
        position_in_filtered: Option<usize>,
        position_in_total: usize,
    },
    // ...
}
```

### Navigation Context Logic

**Viewer Mode:**
- Uses `MediaNavigator` for navigation
- Respects active filters
- `filter_active`: true if any filter is active
- `position_in_filtered`: Some(index) if filter active, None otherwise
- `position_in_total`: Always populated from navigator

**Editor Mode:**
- Images only (videos not editable)
- Ignores filters (shows all images)
- `filter_active`: false
- `position_in_filtered`: None
- `position_in_total`: Position in full image list

### Collection Point Updates

```rust
// In Viewer navigation handler (update.rs:1273):
pub fn handle_navigate_next(ctx: &mut UpdateContext<'_>) -> Task<Message> {
    let info = ctx.media_navigator.navigation_info();

    ctx.diagnostics.log_action(UserAction::NavigateNext {
        context: NavigationContext::Viewer,
        filter_active: info.filter_active,
        position_in_filtered: if info.filter_active { info.current_index } else { None },
        position_in_total: info.current_index.unwrap_or(0),
    });

    // ... rest of existing navigation logic
}

// In Editor navigation handler (update.rs:915):
fn handle_editor_navigate_next(ctx: &mut UpdateContext<'_>) -> Task<Message> {
    let info = ctx.media_navigator.navigation_info();

    // ADD: Editor currently has no logging - add it
    ctx.diagnostics.log_action(UserAction::NavigateNext {
        context: NavigationContext::Editor,
        filter_active: false,  // Editor ignores filters
        position_in_filtered: None,
        position_in_total: info.current_index.unwrap_or(0),
    });

    // ... rest of existing navigation logic
}
```

---

## Testing

### Integration Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn viewer_navigation_includes_viewer_context() {
        // Setup: Initialize app in Viewer mode
        // Action: Navigate next
        // Assert: Event has context: NavigationContext::Viewer
    }

    #[test]
    fn viewer_navigation_reflects_filter_state() {
        // Setup: Initialize with active filter
        // Action: Navigate next
        // Assert: filter_active: true, position_in_filtered: Some(_)
    }

    #[test]
    fn editor_navigation_includes_editor_context() {
        // Setup: Initialize app, enter Editor mode
        // Action: Navigate next in editor
        // Assert: Event has context: NavigationContext::Editor
    }

    #[test]
    fn editor_navigation_ignores_filters() {
        // Setup: Initialize with active filter, enter Editor
        // Action: Navigate in editor
        // Assert: filter_active: false, position_in_filtered: None
    }
}
```

---

## Change Log

| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2025-01-15 | 1.0 | Story created from architecture review | Sarah (PO) |
| 2025-01-15 | 1.1 | PO Validation: Fixed Source Tree (tests in-file), added Target Handler Locations, corrected MediaNavigator API (use navigation_info()), noted Editor handlers have no logging, added exact line numbers for all handlers | Sarah (PO) |

---

## Dev Agent Record

### Agent Model Used
_To be filled by Dev Agent_

### Debug Log References
_To be filled by Dev Agent_

### Completion Notes
_To be filled by Dev Agent_

### File List
_To be filled by Dev Agent_

---

## QA Results

_To be filled by QA Agent_

---
