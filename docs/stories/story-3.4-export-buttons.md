# Story 3.4: Export Buttons and Feedback

**Epic:** 3 - UI Integration
**Status:** Done
**Priority:** High
**Estimate:** 2-3 hours
**Depends On:** Story 2.4, Story 2.5, Story 3.1

---

## Story

**As a** developer,
**I want** buttons to export diagnostic reports to file and clipboard,
**So that** I can easily get diagnostic data out of the application.

---

## Acceptance Criteria

1. "Export to File" button implemented with arrow-down-bar icon
2. "Copy to Clipboard" button implemented with clipboard icon
3. New icons created: `arrow_down_bar` and `clipboard` in icons.rs (visual naming convention)
4. Buttons disabled when buffer is empty (no events to export)
5. Clicking triggers respective export function from Epic 2
6. Success feedback shown via toast notification ("Report exported to {path}" / "Copied to clipboard")
7. Error feedback shown via toast notification with error description
8. Buttons follow existing IcedLens button styles
9. Buttons are keyboard accessible (Tab + Enter)

---

## Tasks

- [ ] **Task 1:** Create arrow_down_bar icon (AC: 1, 3)
  - [ ] Create SVG `assets/icons/source/arrow_down_bar.svg` (32x32, fill="white")
  - [ ] Add dark variant to `src/ui/icons.rs` using `define_icon!` macro
  - [ ] Add `"arrow_down_bar"` to `build.rs::needs_light_variant()` for dark theme support
  - [ ] Add light variant to `src/ui/icons.rs` in `light` module
  - [ ] Run `cargo build` to generate PNGs (dark + light)

- [ ] **Task 2:** Create clipboard icon (AC: 2, 3)
  - [ ] Create SVG `assets/icons/source/clipboard.svg` (32x32, fill="white")
  - [ ] Add dark variant to `src/ui/icons.rs` using `define_icon!` macro
  - [ ] Add `"clipboard"` to `build.rs::needs_light_variant()` for dark theme support
  - [ ] Add light variant to `src/ui/icons.rs` in `light` module
  - [ ] Run `cargo build` to generate PNGs (dark + light)

- [ ] **Task 3:** Add action_icons mappings (AC: 1, 2)
  - [ ] Add `diagnostics` module to `action_icons.rs`
  - [ ] Add `diagnostics::export_file(is_dark_theme: bool)` → theme-aware icon selection
  - [ ] Add `diagnostics::export_clipboard(is_dark_theme: bool)` → theme-aware icon selection
  - [ ] UI code MUST use `action_icons::diagnostics::*` (not `icons::*` directly)
  - [ ] Add `is_dark_theme` to `ViewContext` struct

- [ ] **Task 4:** Add export buttons section to `diagnostics_screen.rs` (AC: 1, 2, 8)
  - [ ] Create `build_export_section()` function
  - [ ] "Export to File" button with icon + text
  - [ ] "Copy to Clipboard" button with icon + text
  - [ ] Use `Row` layout with `spacing::SM`

- [ ] **Task 5:** Implement button disabled state (AC: 4)
  - [ ] Add `event_count` to `ViewContext` (from Story 3.2)
  - [ ] Disable buttons when `event_count == 0`
  - [ ] Conditional `.on_press()` based on count

- [ ] **Task 6:** Add export messages and events (AC: 5)
  - [ ] Add `Message::ExportToFile` to `diagnostics_screen`
  - [ ] Add `Message::ExportToClipboard` to `diagnostics_screen`
  - [ ] Add `Event::ExportToFile` and `Event::ExportToClipboard` for parent
  - [ ] Handle in `update()` - emit events to App

- [ ] **Task 7:** Handle export in App (AC: 5, 6, 7)
  - [ ] Handle `diagnostics_screen::Event::ExportToFile`:
    - [ ] Call `diagnostics.export_with_dialog()`
    - [ ] On success: push success notification with path
    - [ ] On error: push error notification
    - [ ] On cancel: no notification
  - [ ] Handle `diagnostics_screen::Event::ExportToClipboard`:
    - [ ] Call `diagnostics.export_to_clipboard()`
    - [ ] On success: push success notification
    - [ ] On error: push error notification

- [ ] **Task 8:** Add i18n keys (AC: 6, 7)
  - [ ] Button labels (EN + FR)
  - [ ] Success/error notification messages (EN + FR)

- [ ] **Task 9:** Write unit tests (AC: 4, 5)
  - [ ] Test disabled state logic
  - [ ] Test message/event handling

- [ ] **Task 10:** Run validation
  - [ ] `cargo fmt --all`
  - [ ] `cargo clippy --all --all-targets -- -D warnings`
  - [ ] `cargo test`

- [ ] **Task 11:** Commit changes
  - [ ] Stage all changes
  - [ ] Commit: `feat(ui): add diagnostic export buttons [Story 3.4]`

---

## Dev Notes

### Source Tree

```
assets/icons/source/
├── arrow_down_bar.svg      # NEW: Arrow pointing down with bar (32x32, fill="white")
└── clipboard.svg           # NEW: Clipboard shape (32x32, fill="white")

src/
├── build.rs                # MODIFY: Add icons to needs_light_variant()
├── diagnostics/
│   └── collector.rs        # REFERENCE: export_with_dialog(), export_to_clipboard()
├── app/
│   └── mod.rs              # MODIFY: Handle export events, push notifications
├── ui/
│   ├── icons.rs            # MODIFY: Add arrow_down_bar and clipboard icons (dark + light)
│   ├── action_icons.rs     # MODIFY: Add diagnostics module with semantic mappings
│   ├── diagnostics_screen.rs  # MODIFY: Add export section, messages, events
│   └── notifications/      # REFERENCE: Notification::success(), ::error()
└── assets/i18n/
    ├── en-US.ftl           # MODIFY: Add export-related keys
    └── fr.ftl              # MODIFY: Add export-related keys
```

### Icon Creation (CONTRIBUTING.md process)

**Important:** Icons in `icons.rs` use **visual names** (what you see), not semantic names (what it does). The semantic mapping is done in `action_icons.rs`.

#### Step 1: Create SVG files

```xml
<!-- assets/icons/source/arrow_down_bar.svg -->
<!-- Visual: Arrow pointing down with horizontal bar at bottom -->
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" fill="white">
  <path d="M16 4v16m0 0l-6-6m6 6l6-6M6 26h20"
        stroke="white" stroke-width="2" stroke-linecap="round"
        stroke-linejoin="round" fill="none"/>
</svg>

<!-- assets/icons/source/clipboard.svg -->
<!-- Visual: Clipboard shape with clip at top -->
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" fill="white">
  <path d="M10 6H8a2 2 0 00-2 2v16a2 2 0 002 2h16a2 2 0 002-2V8a2 2 0 00-2-2h-2
           M12 4h8a1 1 0 011 1v2a1 1 0 01-1 1h-8a1 1 0 01-1-1V5a1 1 0 011-1z"
        stroke="white" stroke-width="2" fill="none"/>
</svg>
```

**Note:** These are example SVGs. The Dev Agent should create clean 32x32 icons following the existing icon style in `assets/icons/source/`.

#### Step 2: Add to build.rs

```rust
// In build.rs, add to needs_light_variant() match arms:

fn needs_light_variant(name: &str) -> bool {
    matches!(
        name,
        // ... existing entries ...
            // Diagnostics screen
            | "arrow_down_bar"
            | "clipboard"
    )
}
```

#### Step 3: Add to icons.rs

```rust
// In src/ui/icons.rs - Add dark variants in main section
// NOTE: Names describe visual appearance, not function (per CONTRIBUTING.md)

define_icon!(
    arrow_down_bar,
    dark,
    "arrow_down_bar.png",
    "Arrow down bar icon: arrow pointing down with horizontal line at bottom."
);

define_icon!(
    clipboard,
    dark,
    "clipboard.png",
    "Clipboard icon: clipboard shape with clip at top."
);

// In src/ui/icons.rs - Add light variants in `pub mod light` section

pub mod light {
    // ... existing light icons ...

    define_icon!(
        arrow_down_bar,
        light,
        "arrow_down_bar.png",
        "Arrow down bar icon (white): arrow pointing down with horizontal line at bottom."
    );

    define_icon!(
        clipboard,
        light,
        "clipboard.png",
        "Clipboard icon (white): clipboard shape with clip at top."
    );
}
```

#### Step 4: Add to action_icons.rs

```rust
// In src/ui/action_icons.rs
// Semantic layer: maps actions to visual icons
// UI code MUST use these functions, not icons::* directly

/// Diagnostics screen action icons.
pub mod diagnostics {
    use super::icons;
    use iced::widget::image::{Handle, Image};

    /// Export to file action.
    /// Returns dark icon for light theme, light icon for dark theme.
    #[must_use]
    pub fn export_file(is_dark_theme: bool) -> Image<Handle> {
        if is_dark_theme {
            icons::light::arrow_down_bar()
        } else {
            icons::arrow_down_bar()
        }
    }

    /// Copy to clipboard action.
    /// Returns dark icon for light theme, light icon for dark theme.
    #[must_use]
    pub fn export_clipboard(is_dark_theme: bool) -> Image<Handle> {
        if is_dark_theme {
            icons::light::clipboard()
        } else {
            icons::clipboard()
        }
    }
}
```

#### Step 5: Build to generate PNGs

```bash
cargo build
```

The `build.rs` script automatically converts SVGs to dark/light PNG variants based on `needs_light_variant()`.

### Export Section UI

```rust
// In src/ui/diagnostics_screen.rs

use crate::ui::action_icons;
use crate::ui::design_tokens::{sizing, spacing, typography};

/// Contextual data needed to render the diagnostics screen.
pub struct ViewContext<'a> {
    pub i18n: &'a I18n,
    pub status: CollectionStatus,
    pub event_count: usize,
    pub collection_duration: Duration,
    pub is_dark_theme: bool,  // For theme-aware icons
}

fn build_export_section<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let can_export = ctx.event_count > 0;

    // Export to File button (icon + text)
    // Pass is_dark_theme to get correct icon variant
    let file_content = Row::new()
        .spacing(spacing::XS)
        .align_y(Vertical::Center)
        .push(action_icons::diagnostics::export_file(ctx.is_dark_theme)
            .width(sizing::ICON_SM))
        .push(text(ctx.i18n.tr("diagnostics-export-file")).size(typography::BODY));

    let file_button = button(file_content)
        .padding([spacing::XS, spacing::SM]);

    let file_button = if can_export {
        file_button.on_press(Message::ExportToFile)
    } else {
        file_button  // Disabled - no on_press
    };

    // Copy to Clipboard button (icon + text)
    let clipboard_content = Row::new()
        .spacing(spacing::XS)
        .align_y(Vertical::Center)
        .push(action_icons::diagnostics::export_clipboard(ctx.is_dark_theme)
            .width(sizing::ICON_SM))
        .push(text(ctx.i18n.tr("diagnostics-export-clipboard")).size(typography::BODY));

    let clipboard_button = button(clipboard_content)
        .padding([spacing::XS, spacing::SM]);

    let clipboard_button = if can_export {
        clipboard_button.on_press(Message::ExportToClipboard)
    } else {
        clipboard_button  // Disabled - no on_press
    };

    // Layout
    Row::new()
        .spacing(spacing::SM)
        .push(file_button)
        .push(clipboard_button)
        .into()
}
```

### Messages and Events

```rust
// In src/ui/diagnostics_screen.rs

#[derive(Debug, Clone)]
pub enum Message {
    BackToViewer,
    RefreshStatus,
    ToggleResourceCollection(bool),
    ExportToFile,      // NEW
    ExportToClipboard, // NEW
}

#[derive(Debug, Clone)]
pub enum Event {
    None,
    BackToViewer,
    ToggleResourceCollection(bool),
    ExportToFile,      // NEW
    ExportToClipboard, // NEW
}

pub fn update(message: &Message) -> Event {
    match message {
        Message::BackToViewer => Event::BackToViewer,
        Message::RefreshStatus => Event::None,
        Message::ToggleResourceCollection(enabled) => {
            Event::ToggleResourceCollection(*enabled)
        }
        Message::ExportToFile => Event::ExportToFile,
        Message::ExportToClipboard => Event::ExportToClipboard,
    }
}
```

### App Event Handling

```rust
// In src/app/mod.rs

use crate::ui::notifications::Notification;

// Handle export events from diagnostics screen
diagnostics_screen::Event::ExportToFile => {
    match self.diagnostics.export_with_dialog() {
        Ok(path) => {
            // Success notification with file path
            let filename = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            self.notification_manager.push(
                Notification::success("diagnostics-export-success")
                    .with_arg("filename", filename)
            );
        }
        Err(ExportError::Cancelled) => {
            // User cancelled - no notification needed
        }
        Err(e) => {
            self.notification_manager.push(
                Notification::error("diagnostics-export-error")
                    .with_arg("error", e.to_string())
            );
        }
    }
    Task::none()
}

diagnostics_screen::Event::ExportToClipboard => {
    match self.diagnostics.export_to_clipboard() {
        Ok(()) => {
            self.notification_manager.push(
                Notification::success("diagnostics-clipboard-success")
            );
        }
        Err(e) => {
            self.notification_manager.push(
                Notification::error("diagnostics-clipboard-error")
                    .with_arg("error", e.to_string())
            );
        }
    }
    Task::none()
}
```

### I18n Keys

```ftl
# English (assets/i18n/en-US.ftl)
diagnostics-export-file = Export to File
diagnostics-export-clipboard = Copy to Clipboard
diagnostics-export-success = Report exported to { $filename }
diagnostics-clipboard-success = Report copied to clipboard
diagnostics-export-error = Export failed: { $error }
diagnostics-clipboard-error = Clipboard error: { $error }

# French (assets/i18n/fr.ftl)
diagnostics-export-file = Exporter vers fichier
diagnostics-export-clipboard = Copier dans le presse-papiers
diagnostics-export-success = Rapport exporté vers { $filename }
diagnostics-clipboard-success = Rapport copié dans le presse-papiers
diagnostics-export-error = Échec de l'export : { $error }
diagnostics-clipboard-error = Erreur presse-papiers : { $error }
```

### Note on Blocking Dialog

`export_with_dialog()` opens a native file dialog which is blocking. This is acceptable for MVP since:
- File dialogs are typically quick interactions
- The user expects the UI to wait while they choose a location

**Future enhancement:** Use `rfd::AsyncFileDialog` for non-blocking dialog.

### Design Decision: Disable Only When Empty

The export buttons are disabled **only when buffer is empty**, not when resource collection is disabled. Rationale:
- Event collection (user actions, errors) is always active
- Buffer may contain valuable diagnostic data even without resource metrics
- Users should be able to export whatever has been collected

---

## Testing

### Unit Tests (in diagnostics_screen.rs)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_to_file_emits_event() {
        let event = update(&Message::ExportToFile);
        assert!(matches!(event, Event::ExportToFile));
    }

    #[test]
    fn export_to_clipboard_emits_event() {
        let event = update(&Message::ExportToClipboard);
        assert!(matches!(event, Event::ExportToClipboard));
    }
}
```

### Manual Tests

| Test | Steps | Expected Result |
|------|-------|-----------------|
| Export to file | Click "Export to File", choose location | File saved, success toast appears |
| Export cancel | Click "Export to File", cancel dialog | No toast, no action |
| Copy to clipboard | Click "Copy to Clipboard" | Success toast, JSON in clipboard |
| Paste verification | After clipboard copy, paste in text editor | Valid JSON report |
| Disabled state | Open Diagnostics with empty buffer | Both buttons visually disabled |
| Empty buffer click | Try clicking disabled button | Nothing happens |
| Export error | Export to read-only location | Error toast with message |
| Clipboard error | (Headless test) Export to clipboard | Error toast with clipboard message |
| Keyboard access | Tab to button, press Enter | Export triggers |

---

## Dev Agent Record

### Agent Model Used
<!-- Record which AI model completed this story -->

### Completion Notes
<!-- Dev agent adds notes here during implementation -->

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2026-01-14 | Story created | PM |
| 2026-01-14 | PO Validation: Added Source Tree, code examples, fixed AC3 (disable only when empty), noted no icons available, added i18n keys | Sarah (PO) |
| 2026-01-14 | PO Validation: Fixed icon naming (download → arrow_down_bar) per CONTRIBUTING.md visual naming rule, fixed i18n paths (en-US.ftl, fr.ftl) | Sarah (PO) |
| 2026-01-14 | PO Validation: Added light variants in build.rs + icons.rs, clarified action_icons.rs usage requirement | Sarah (PO) |
| 2026-01-14 | PO Validation: Added theme-aware function signature `(is_dark_theme: bool)` following navigation::edit() pattern, added is_dark_theme to ViewContext | Sarah (PO) |
| 2026-01-15 | PO: Status corrected to Ready for Review - implementation confirmed via commit b020f91, story file was not updated by dev | Sarah (PO) |

### File List
<!-- Files created or modified -->

---

## QA Results

### Review Date: 2026-01-15
### Reviewed By: Quinn (Test Architect)
### Gate Decision: PASS

**Note:** Story file tasks were not updated by dev, but implementation confirmed via commit `b020f91` and code review.

#### Code Quality Assessment
Export buttons properly implemented with theme-aware icons via action_icons module. Disabled state correctly checks event_count. Message/Event flow to App handles export calls with appropriate notifications. i18n keys added for 5 languages.

#### AC Traceability

| AC | Description | Status |
|----|-------------|--------|
| 1 | Export to File button | ✓ build_export_section() |
| 2 | Copy to Clipboard button | ✓ build_export_section() |
| 3 | Icons created (arrow_down_bar, clipboard) | ✓ icons.rs + action_icons.rs |
| 4 | Disabled when buffer empty | ✓ `can_export = ctx.event_count > 0` |
| 5 | Triggers Epic 2 export functions | ✓ App handles events |
| 6 | Success toast notification | ✓ notification-diagnostics-*-success |
| 7 | Error toast notification | ✓ notification-diagnostics-*-error |
| 8 | Follows button styles | ✓ Design tokens, padding |
| 9 | Keyboard accessible | ✓ Native button focus |

#### Test Coverage
- `export_to_file_emits_event` - Event emission
- `export_to_clipboard_emits_event` - Event emission
- 922 total tests pass (current)

#### NFR Assessment
- Security: N/A - Export uses existing Epic 2 functions
- Performance: N/A - Button click triggers export
- Reliability: PASS - Proper error handling with notifications

#### Administrative Note
Story file tasks should be marked [x] by dev. Implementation is complete per git commit.

**Recommendation:** Ready for Done

---
