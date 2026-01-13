# User Interface Design Goals

## Overall UX Vision

The Diagnostics tool should be unobtrusive during normal use and easily accessible when needed. The interface should feel like a natural extension of IcedLens, following the existing design language. Users (developers/contributors) should be able to understand the collection status at a glance and export reports with minimal friction.

## Key Interaction Paradigms

- **Passive by default:** Collection runs silently when enabled; no user attention required
- **On-demand interaction:** Users access the Diagnostics screen only when they need to check status or export
- **One-click exports:** Both file and clipboard exports should be single-action operations
- **Clear status indication:** Visual feedback on whether collection is active and what mode it's in

## Core Screens and Views

1. **Diagnostics Screen** (new)
   - Collection status indicator (enabled/disabled, mode)
   - Toggle switch to enable/disable collection
   - Export to file button
   - Copy to clipboard button
   - Brief explanation of what data is collected

2. **Hamburger Menu** (modification)
   - Add "Diagnostics" entry alongside Settings, Help, About

## Accessibility

**WCAG AA** - Following IcedLens's existing accessibility standards:
- Sufficient contrast for status indicators
- Keyboard navigable controls
- Clear focus states
- Screen reader compatible status announcements

## Branding

Follow existing IcedLens design system:
- Use design tokens from `design_tokens.rs`
- Match existing button and toggle styles
- Use action icons from `action_icons.rs` for new icons
- Consistent spacing and typography

## Target Device and Platforms

**Desktop Only** - Linux, Windows, macOS (same as IcedLens core application)

---
