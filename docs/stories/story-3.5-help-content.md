# Story 3.5: Information and Help Content

**Epic:** 3 - UI Integration
**Status:** Done
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
4. Link or reference to USER_GUIDE.md for more details
5. Content is concise and doesn't clutter the interface
6. Text follows existing typography styles (design tokens)
7. Content is translatable (uses i18n system)

---

## Tasks

- [x] **Task 1:** Add info section to diagnostics_screen (AC: 1, 5, 6)
  - [x] Create `build_info_section()` function following about.rs pattern
  - [x] Use `icons::info()` for section header
  - [x] 2-3 sentences explaining Diagnostics purpose
  - [x] Place below header, above status section

- [x] **Task 2:** Add data collection summary (AC: 2, 5, 6)
  - [x] Create `build_data_collected_section()` function
  - [x] Bullet list of what's collected:
    - System resources (CPU, RAM, disk)
    - User actions (navigation, edits)
    - Application states (screen, mode)
    - Warnings and errors
  - [x] Use `typography::BODY` for text
  - [x] Use `"• {item}"` format for bullets

- [x] **Task 3:** Add privacy assurance (AC: 3, 5, 6)
  - [x] Add privacy notice within info section
  - [x] "All data is anonymized before export"
  - [x] "Data is never sent automatically"
  - [x] Use `palette::GRAY_400` for subtle secondary text styling

- [x] **Task 4:** Add documentation reference (AC: 4, 5)
  - [x] Add link/reference to USER_GUIDE.md
  - [x] Display repository URL (docs/USER_GUIDE.md)
  - [x] Follow `build_link_item()` pattern from about.rs

- [x] **Task 5:** Add i18n keys (AC: 7)
  - [x] `diagnostics-info-title` - section title
  - [x] `diagnostics-info-description` - purpose description
  - [x] `diagnostics-data-collected-title` - data collected header
  - [x] `diagnostics-data-item-resources` - "System resources (CPU, RAM, disk)"
  - [x] `diagnostics-data-item-actions` - "User actions"
  - [x] `diagnostics-data-item-states` - "Application states"
  - [x] `diagnostics-data-item-errors` - "Warnings and errors"
  - [x] `diagnostics-privacy-notice` - privacy assurance text
  - [x] `diagnostics-docs-link` - documentation link label

- [x] **Task 6:** Update i18n translation files (AC: 7)
  - [x] English (`assets/i18n/en-US.ftl`)
  - [x] French (`assets/i18n/fr.ftl`)
  - [x] German (`assets/i18n/de.ftl`)
  - [x] Spanish (`assets/i18n/es.ftl`)
  - [x] Italian (`assets/i18n/it.ftl`)

- [x] **Task 7:** Run validation
  - [x] `cargo fmt --all`
  - [x] `cargo clippy --all --all-targets -- -D warnings`
  - [x] `cargo test`

- [x] **Task 8:** Commit changes
  - [x] Stage all changes
  - [x] Commit: `feat(ui): add diagnostics help content [Story 3.5]`

---

## Dev Notes

### Source Tree

```
src/ui/
├── diagnostics_screen.rs  # MODIFY: Add info section, data collected, privacy notice
├── about.rs               # REFERENCE: build_section() pattern, build_link_item()
├── icons.rs               # REFERENCE: icons::info()
└── design_tokens.rs       # REFERENCE: typography, spacing, palette

assets/i18n/
├── en-US.ftl              # MODIFY: Add diagnostics info keys
└── fr.ftl                 # MODIFY: Add diagnostics info keys

docs/
└── USER_GUIDE.md          # REFERENCE: Documentation to link to
```

### UI Pattern (following about.rs)

```rust
// In src/ui/diagnostics_screen.rs

use crate::ui::design_tokens::{palette, radius, sizing, spacing, typography};
use crate::ui::icons;
use iced::widget::{container, rule, text, Column, Container, Row, Text};

/// Documentation URL for diagnostics.
const DOCS_URL: &str = "https://codeberg.org/Bawycle/iced_lens/src/branch/master/docs/USER_GUIDE.md";

fn build_info_section<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    // Description
    let description = Text::new(ctx.i18n.tr("diagnostics-info-description"))
        .size(typography::BODY);

    // Data collected list
    let data_list = Column::new()
        .spacing(spacing::XS)
        .push(Text::new(ctx.i18n.tr("diagnostics-data-collected-title"))
            .size(typography::BODY_LG))
        .push(build_data_item(ctx.i18n.tr("diagnostics-data-item-resources")))
        .push(build_data_item(ctx.i18n.tr("diagnostics-data-item-actions")))
        .push(build_data_item(ctx.i18n.tr("diagnostics-data-item-states")))
        .push(build_data_item(ctx.i18n.tr("diagnostics-data-item-errors")));

    // Privacy notice (subtle styling)
    let privacy = Text::new(ctx.i18n.tr("diagnostics-privacy-notice"))
        .size(typography::BODY)
        .color(palette::GRAY_400);

    // Documentation link
    let docs_link = build_link_item(
        &ctx.i18n.tr("diagnostics-docs-link"),
        DOCS_URL,
    );

    let content = Column::new()
        .spacing(spacing::SM)
        .push(description)
        .push(data_list)
        .push(privacy)
        .push(docs_link);

    build_section(
        icons::info(),
        ctx.i18n.tr("diagnostics-info-title"),
        content.into(),
    )
}

/// Build a bullet point item.
fn build_data_item<'a>(description: &str) -> Element<'a, Message> {
    Text::new(format!("• {description}"))
        .size(typography::BODY)
        .into()
}

/// Build a link item with label and URL (same pattern as about.rs).
fn build_link_item<'a>(label: &str, url: &'a str) -> Element<'a, Message> {
    Row::new()
        .spacing(spacing::SM)
        .push(Text::new(format!("{label}:")).size(typography::BODY))
        .push(Text::new(url).size(typography::BODY))
        .into()
}

/// Build a section with icon, title, and content (same pattern as about.rs).
fn build_section(
    icon: iced::widget::Image<iced::widget::image::Handle>,
    title: String,
    content: Element<'_, Message>,
) -> Element<'_, Message> {
    let icon_sized = icons::sized(icon, sizing::ICON_MD);

    let header = Row::new()
        .spacing(spacing::SM)
        .align_y(Vertical::Center)
        .push(icon_sized)
        .push(Text::new(title).size(typography::TITLE_SM));

    let inner = Column::new()
        .spacing(spacing::SM)
        .push(header)
        .push(rule::horizontal(1))
        .push(content);

    Container::new(inner)
        .padding(spacing::MD)
        .width(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.extended_palette().background.weak.color.into()),
            border: Border {
                radius: radius::MD.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}
```

### I18n Keys

```ftl
# English (assets/i18n/en-US.ftl)
diagnostics-info-title = About Diagnostics
diagnostics-info-description = The Diagnostics tool collects runtime information to help troubleshoot issues. Generate a report to share with support or analyze application behavior.
diagnostics-data-collected-title = Data collected:
diagnostics-data-item-resources = System resources (CPU, RAM, disk usage)
diagnostics-data-item-actions = User actions (navigation, edits, commands)
diagnostics-data-item-states = Application states (screen, mode, settings)
diagnostics-data-item-errors = Warnings and errors
diagnostics-privacy-notice = All data is anonymized before export. Nothing is sent automatically.
diagnostics-docs-link = Documentation

# French (assets/i18n/fr.ftl)
diagnostics-info-title = À propos des Diagnostics
diagnostics-info-description = L'outil de diagnostic collecte des informations d'exécution pour aider à résoudre les problèmes. Générez un rapport à partager avec le support ou pour analyser le comportement de l'application.
diagnostics-data-collected-title = Données collectées :
diagnostics-data-item-resources = Ressources système (CPU, RAM, utilisation disque)
diagnostics-data-item-actions = Actions utilisateur (navigation, éditions, commandes)
diagnostics-data-item-states = États de l'application (écran, mode, paramètres)
diagnostics-data-item-errors = Avertissements et erreurs
diagnostics-privacy-notice = Toutes les données sont anonymisées avant export. Rien n'est envoyé automatiquement.
diagnostics-docs-link = Documentation
```

### Design Decisions

1. **Single info section**: Rather than multiple scattered sections, combine description, data list, and privacy notice in one cohesive block for better readability.

2. **Reuse about.rs pattern**: The `build_section()` pattern provides consistent styling with other help/about screens.

3. **Privacy prominence**: Privacy notice uses subtle gray color to indicate secondary information while still being visible.

4. **Concise content**: Keep descriptions minimal - developers want quick access to functionality, not lengthy explanations.

---

## Testing

### Unit Tests

None required - this is static informational content.

### Manual Tests

| Test | Steps | Expected Result |
|------|-------|-----------------|
| Content display | Open Diagnostics screen | Info section visible with description, data list, privacy notice |
| Typography | Check text sizes | Matches BODY/BODY_LG/TITLE_SM tokens |
| Translations | Switch language to French | All text displays in French |
| Layout | View on different window sizes | Content doesn't overflow or clip |
| Link display | Check documentation URL | URL is visible and correctly formatted |
| Visual consistency | Compare with About screen | Section styling matches (icon, header, container) |

---

## Dev Agent Record

### Agent Model Used
Claude Opus 4.5 (claude-opus-4-5-20251101)

### Completion Notes
- All 7 acceptance criteria satisfied
- Implemented info section with description, data list, privacy notice, and docs link following about.rs pattern
- Added i18n translations for all 5 languages (en-US, fr, de, es, it)
- 922 tests pass, clippy clean

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2026-01-14 | Story created | PM |
| 2026-01-14 | PO Validation: Added Source Tree, code examples following about.rs pattern, Task-AC mapping, i18n keys, fixed file paths, added QA Results section | Sarah (PO) |
| 2026-01-15 | Implementation completed, all tasks done | James (Dev Agent) |

### File List
| File | Action |
|------|--------|
| `src/ui/diagnostics_screen.rs` | Modified - Added info section functions |
| `assets/i18n/en-US.ftl` | Modified - Added 10 diagnostics info keys |
| `assets/i18n/fr.ftl` | Modified - Added 10 diagnostics info keys |
| `assets/i18n/de.ftl` | Modified - Added 10 diagnostics info keys |
| `assets/i18n/es.ftl` | Modified - Added 10 diagnostics info keys |
| `assets/i18n/it.ftl` | Modified - Added 10 diagnostics info keys |

---

## QA Results

### Review Date: 2026-01-15

### Reviewed By: Quinn (Test Architect)

### Code Quality Assessment

Implementation is clean, well-structured, and follows established project patterns. The code correctly replicates the `about.rs` section pattern for visual consistency. All i18n keys are properly defined across all 5 supported languages.

### Refactoring Performed

None required. Code quality is good and follows project conventions.

### Compliance Check

- Coding Standards: ✓ Uses design tokens, follows naming conventions
- Project Structure: ✓ All files in correct locations
- Testing Strategy: ✓ Story explicitly exempts unit tests (static content)
- All ACs Met: ✓ All 7 acceptance criteria verified

### AC Traceability

| AC | Description | Implementation | Status |
|----|-------------|----------------|--------|
| 1 | Brief description (2-3 sentences) | `diagnostics-info-description` key, 2 sentences | ✓ |
| 2 | Data collected summary (bullet list) | 4 bullet items via `build_data_item()` | ✓ |
| 3 | Privacy assurance statement | `diagnostics-privacy-notice` key with both requirements | ✓ |
| 4 | Link to USER_GUIDE.md | `DOCS_URL` constant, `build_link_item()` | ✓ |
| 5 | Content is concise | Single cohesive section, minimal text | ✓ |
| 6 | Typography uses design tokens | `typography::BODY`, `BODY_LG`, `TITLE_SM` | ✓ |
| 7 | Content is translatable | All 5 languages: en-US, fr, de, es, it | ✓ |

### Improvements Checklist

All items completed by developer:
- [x] Info section with icon header following about.rs pattern
- [x] Bullet list for data collected items
- [x] Privacy notice with subtle gray styling
- [x] Documentation URL reference
- [x] All i18n keys defined (10 keys × 5 languages = 50 translations)
- [x] View function integrates info_section in correct position

No improvements required.

### Security Review

N/A - Static informational UI content with no user input handling.

### Performance Considerations

N/A - Simple UI rendering, no computation or data processing.

### Files Modified During Review

None.

### Gate Status

Gate: **PASS** → `docs/qa/gates/3.5-help-content.yml`

### Recommended Status

✓ **Ready for Done**

The implementation satisfies all acceptance criteria with clean, maintainable code that follows project patterns.

---
