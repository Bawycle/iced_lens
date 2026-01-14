# Story 3.1: Diagnostics Screen Layout and Navigation

**Epic:** 3 - UI Integration
**Status:** Ready for Review
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
3. Menu entry uses appropriate icon from `icons` module
4. Navigation to/from Diagnostics screen works correctly
5. Screen follows existing IcedLens layout patterns (header, content area)
6. Back navigation returns to previous screen (Viewer)
7. Keyboard navigation works:
   - **Tab**: Navigate between focusable elements
   - **Enter**: Activate focused element (buttons)
   - **Escape**: Not required for MVP (back button suffices)

---

## Tasks

- [x] **Task 1:** Add `Screen::Diagnostics` variant (AC: 1)
  - [x] Update `src/app/screen.rs`
  - [x] Add `Diagnostics` to Screen enum after `About`

- [x] **Task 2:** Create `src/ui/diagnostics_screen.rs` (AC: 1, 5)
  - [x] Add module file
  - [x] Define `ViewContext<'a>` struct with `i18n: &'a I18n`
  - [x] Define `Message` enum with `BackToViewer`
  - [x] Define `Event` enum with `None`, `BackToViewer`
  - [x] Follow pattern from `src/ui/about.rs`

- [x] **Task 3:** Implement `view()` function (AC: 5, 6)
  - [x] Back button with "← {i18n key}" format
  - [x] Title using `typography::TITLE_LG`
  - [x] Placeholder content area (text: "Diagnostics controls will appear here")
  - [x] Use `scrollable()` wrapper like about.rs
  - [x] Use design tokens for all styling

- [x] **Task 4:** Implement `update()` function (AC: 4, 6)
  - [x] Handle `Message::BackToViewer` → return `Event::BackToViewer`
  - [x] Pattern: `pub fn update(message: &Message) -> Event`

- [x] **Task 5:** Update `src/ui/mod.rs` (AC: 1)
  - [x] Add `pub mod diagnostics_screen;`

- [x] **Task 6:** Add to hamburger menu (AC: 2, 3)
  - [x] Update `src/ui/navbar.rs`:
    - [x] Add `Message::OpenDiagnostics` variant
    - [x] Add `Event::OpenDiagnostics` variant
    - [x] Add handler in `update()`: close menu, return `Event::OpenDiagnostics`
    - [x] Add menu item in `build_dropdown()` using `icons::cog()` (or `icons::info()`)
  - [x] Add i18n key `menu-diagnostics` usage

- [x] **Task 7:** Integrate in App (AC: 4)
  - [x] Update `src/app/message.rs`:
    - [x] Add import: `use crate::ui::diagnostics_screen;`
    - [x] Add variant: `Diagnostics(diagnostics_screen::Message)`
  - [x] Update `src/app/update.rs`:
    - [x] Handle `Message::Diagnostics(msg)` → call `diagnostics_screen::update(&msg)`
    - [x] Handle `navbar::Event::OpenDiagnostics` → switch to `Screen::Diagnostics`
    - [x] Handle `diagnostics_screen::Event::BackToViewer` → switch to `Screen::Viewer`
  - [x] Update `src/app/view.rs`:
    - [x] Add match arm for `Screen::Diagnostics` → render `diagnostics_screen::view()`

- [x] **Task 8:** Add i18n keys (AC: 2)
  - [x] Update `assets/i18n/en-US.ftl`:
    - [x] `menu-diagnostics = Diagnostics`
    - [x] `diagnostics-title = Diagnostics`
    - [x] `diagnostics-back-button = Back to Viewer`
  - [x] Update `assets/i18n/fr.ftl`:
    - [x] `menu-diagnostics = Diagnostics`
    - [x] `diagnostics-title = Diagnostics`
    - [x] `diagnostics-back-button = Retour`

- [ ] **Task 9:** Test navigation (AC: 4, 6, 7)
  - [ ] Manual: Menu → Diagnostics works
  - [ ] Manual: Back button → Viewer works
  - [ ] Manual: Tab navigation highlights elements
  - [ ] Manual: Enter on back button returns to Viewer

- [x] **Task 10:** Run validation
  - [x] `cargo fmt --all`
  - [x] `cargo clippy --all --all-targets -- -D warnings`
  - [x] `cargo test`

- [x] **Task 11:** Commit changes
  - [x] Stage all changes
  - [x] Commit: `feat(ui): add diagnostics screen layout [Story 3.1]`

---

## Dev Notes

### Source Tree

```
src/
├── app/
│   ├── screen.rs           # MODIFY: Add Screen::Diagnostics
│   ├── message.rs          # MODIFY: Add Diagnostics(diagnostics_screen::Message)
│   ├── update.rs           # MODIFY: Handle Diagnostics messages + navbar event
│   └── view.rs             # MODIFY: Render DiagnosticsScreen when Screen::Diagnostics
├── ui/
│   ├── mod.rs              # MODIFY: pub mod diagnostics_screen;
│   ├── diagnostics_screen.rs  # NEW: Screen implementation
│   ├── navbar.rs           # MODIFY: Add OpenDiagnostics Message/Event + menu item
│   ├── about.rs            # REFERENCE: Pattern to follow (DO NOT MODIFY)
│   ├── icons.rs            # REFERENCE: Available icons (DO NOT MODIFY)
│   └── design_tokens.rs    # REFERENCE: Styling tokens (DO NOT MODIFY)
└── assets/i18n/
    ├── en/main.ftl         # MODIFY: Add English keys
    └── fr/main.ftl         # MODIFY: Add French keys
```

### Screen Pattern (Reference: about.rs)

Each screen follows this pattern:

```rust
// Imports needed
use crate::i18n::fluent::I18n;
use crate::ui::design_tokens::{spacing, typography};
use iced::{
    alignment::Horizontal,
    widget::{button, scrollable, text, Column, Text},
    Element, Length,
};

/// Contextual data needed to render the screen.
pub struct ViewContext<'a> {
    pub i18n: &'a I18n,
}

/// Messages emitted by the screen.
#[derive(Debug, Clone)]
pub enum Message {
    BackToViewer,
}

/// Events propagated to the parent application.
#[derive(Debug, Clone)]
pub enum Event {
    None,
    BackToViewer,
}

/// Process a screen message and return the corresponding event.
#[must_use]
pub fn update(message: &Message) -> Event {
    match message {
        Message::BackToViewer => Event::BackToViewer,
    }
}

/// Render the screen.
#[must_use]
pub fn view(ctx: ViewContext<'_>) -> Element<'_, Message> {
    let back_button = button(
        text(format!("← {}", ctx.i18n.tr("diagnostics-back-button")))
            .size(typography::BODY),
    )
    .on_press(Message::BackToViewer);

    let title = Text::new(ctx.i18n.tr("diagnostics-title"))
        .size(typography::TITLE_LG);

    let content = Column::new()
        .width(Length::Fill)
        .spacing(spacing::LG)
        .align_x(Horizontal::Left)
        .padding(spacing::MD)
        .push(back_button)
        .push(title)
        .push(Text::new("Diagnostics controls will appear here."));

    scrollable(content).into()
}
```

### Navbar Integration (navbar.rs)

Add to existing enums and handlers:

```rust
// In Message enum:
pub enum Message {
    // ... existing variants ...
    OpenDiagnostics,  // NEW
}

// In Event enum:
pub enum Event {
    // ... existing variants ...
    OpenDiagnostics,  // NEW
}

// In update() function, add match arm:
Message::OpenDiagnostics => {
    *menu_open = false;
    Event::OpenDiagnostics
}

// In build_dropdown(), add menu item after about_item:
let diagnostics_item = build_menu_item(
    icons::cog(),  // Or icons::info() - consistent technical icon
    ctx.i18n.tr("menu-diagnostics"),
    Message::OpenDiagnostics,
);

// Add to menu_column:
let menu_column = Column::new()
    .spacing(spacing::XXS)
    .push(settings_item)
    .push(help_item)
    .push(about_item)
    .push(diagnostics_item);  // NEW
```

### App Integration (update.rs)

```rust
// Handle navbar event:
navbar::Event::OpenDiagnostics => {
    self.screen = Screen::Diagnostics;
    Task::none()
}

// Handle diagnostics screen message:
Message::Diagnostics(msg) => {
    let event = diagnostics_screen::update(&msg);
    match event {
        diagnostics_screen::Event::BackToViewer => {
            self.screen = Screen::Viewer;
        }
        diagnostics_screen::Event::None => {}
    }
    Task::none()
}
```

### Icon Selection

Available menu-appropriate icons:
- `icons::cog()` - Gear (technical/settings-adjacent)
- `icons::info()` - Info circle (already used for About)

Recommendation: Use `icons::cog()` to differentiate from About. Future: Create dedicated diagnostics icon.

---

## Testing

### Unit Tests (in diagnostics_screen.rs)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::fluent::I18n;

    #[test]
    fn diagnostics_view_renders() {
        let i18n = I18n::default();
        let ctx = ViewContext { i18n: &i18n };
        let _element = view(ctx);
        // Test passes if no panic
    }

    #[test]
    fn back_to_viewer_emits_event() {
        let event = update(&Message::BackToViewer);
        assert!(matches!(event, Event::BackToViewer));
    }
}
```

### Manual Tests

| Test | Steps | Expected Result |
|------|-------|-----------------|
| Menu navigation | 1. Click hamburger menu<br>2. Click "Diagnostics" | Diagnostics screen displayed |
| Back button | 1. On Diagnostics screen<br>2. Click "← Back to viewer" | Returns to Viewer screen |
| Tab navigation | 1. On Diagnostics screen<br>2. Press Tab repeatedly | Focus moves between back button and other elements |
| Enter activation | 1. Focus back button with Tab<br>2. Press Enter | Returns to Viewer screen |
| Visual consistency | Compare with About screen | Same layout structure, spacing, typography |

---

## Dev Agent Record

### Agent Model Used
Claude Opus 4.5

### Completion Notes
- Implemented Diagnostics screen following about.rs pattern
- Added Screen::Diagnostics variant, Message/Event handling
- Integrated into hamburger menu with cog icon
- Added i18n keys for all 5 languages (en-US, fr, de, es, it)
- Added UserAction::OpenDiagnostics to diagnostics events for logging
- All validations pass (fmt, clippy, test - 881 tests)
- Task 9 (manual testing) left for QA validation

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2026-01-14 | Story created | PM |
| 2026-01-14 | PO Validation: Added Source Tree, patterns, Task-AC mapping, testing details | Sarah (PO) |
| 2026-01-14 | Implementation completed | James (Dev) |

### File List
- `src/app/screen.rs` - MODIFIED: Added Screen::Diagnostics variant
- `src/app/message.rs` - MODIFIED: Added Diagnostics message import and variant
- `src/app/mod.rs` - MODIFIED: Handle Diagnostics message
- `src/app/update.rs` - MODIFIED: Added handle_diagnostics_message, OpenDiagnostics navbar event
- `src/app/view.rs` - MODIFIED: Added Screen::Diagnostics rendering
- `src/app/subscription.rs` - MODIFIED: Added Diagnostics to screen pattern match
- `src/ui/mod.rs` - MODIFIED: Added pub mod diagnostics_screen
- `src/ui/diagnostics_screen.rs` - NEW: Screen implementation with view/update
- `src/ui/navbar.rs` - MODIFIED: Added OpenDiagnostics Message/Event, menu item
- `src/diagnostics/events.rs` - MODIFIED: Added UserAction::OpenDiagnostics
- `assets/i18n/en-US.ftl` - MODIFIED: Added diagnostics i18n keys
- `assets/i18n/fr.ftl` - MODIFIED: Added diagnostics i18n keys
- `assets/i18n/de.ftl` - MODIFIED: Added diagnostics i18n keys
- `assets/i18n/es.ftl` - MODIFIED: Added diagnostics i18n keys
- `assets/i18n/it.ftl` - MODIFIED: Added diagnostics i18n keys

---

## QA Results

<!-- QA agent adds results here after review -->

---
