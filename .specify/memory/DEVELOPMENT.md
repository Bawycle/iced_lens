# Development Status

This document tracks ongoing development work for IcedLens. It serves as a reference for features in progress and implementation notes.

**Last Updated:** 2025-11-24

---

## 🚧 In Progress: Image Editor Mode

**Goal:** Add rotate, crop, and resize capabilities to IcedLens while maintaining the clean, minimal UX philosophy.

### Architecture Overview

**Design Pattern:** Separate editor mode following the existing Settings pattern
- `AppMode::Editor` alongside `Viewer` and `Settings`
- State-down, messages-up architecture
- Non-destructive editing until explicit save

**UI Layout:** Retractable side toolbar
```
┌────────────┬──────────────────────────────────────────┐
│ [☰] Toggle │                                          │
│            │                                          │
│   Tools    │                                          │
│  ┌──────┐  │                                          │
│  │  ↻   │  │         Image with overlays              │
│  └──────┘  │         (crop handles, etc.)             │
│  ┌──────┐  │                                          │
│  │  ↺   │  │                                          │
│  └──────┘  │                                          │
│  ┌──────┐  │                                          │
│  │ Crop │  │                                          │
│  └──────┘  │                                          │
│  ┌──────┐  │                                          │
│  │Resize│  │                                          │
│  └──────┘  │                                          │
│            │                                          │
│  [◀] [▶]   │  (Navigation arrows - edit other images) │
│            │                                          │
│  [Cancel]  │                                          │
│  [ Save ]  │                                          │
└────────────┴──────────────────────────────────────────┘
```

### Implementation Status

#### ✅ Completed
1. **Module Structure** (`src/ui/editor/mod.rs`)
   - `State` struct with image data, transformation history, tool state
   - `EditorTool` enum: `Rotate`, `Crop`, `Resize`
   - `Transformation` enum for undo/redo support
   - `CropRatio` enum: Free, Square, Landscape (16:9), Portrait (9:16), Photo (4:3)
   - `ResizeState` with scale slider, pixel inputs, lock aspect ratio
   - `Message` and `Event` types following project patterns
   - Basic tests (4 passing)

2. **App Integration** (COMPLETE)
   - `AppMode::Editor` variant added
   - `editor: Option<EditorState>` field in App
   - `Message::Editor` routing fully implemented
   - `handle_editor_message()` with event handling
   - Mode switching: Viewer ↔ Editor ↔ Settings
   - "✏ Edit" button in viewer toolbar (appears when image loaded)
   - View rendering for all modes

3. **Translations** (en-US, fr)
   - Basic strings: title, cancel, save, tool names
   - Rotate left/right labels
   - Undo/redo labels

4. **Sidebar UI** (COMPLETE)
   - Retractable sidebar (hamburger toggle ☰)
   - Width: 180px expanded, 60px collapsed
   - Tool buttons:
     - Rotate Left (↻) / Rotate Right (↺)
     - Crop (selectable tool)
     - Resize (selectable tool)
   - Navigation arrows (◀ ▶) for browsing images
   - Action buttons: Cancel, Save (primary button)
   - Visual hierarchy with horizontal rules
   - Spacer pushing bottom controls to bottom
   - Gray background (#F2F2F2) for contrast

#### 🔄 In Progress
- (None - infrastructure complete)

#### ⏳ To Do
1. **Rotate Tool Implementation** - Actual 90° rotation logic using `image` crate
2. **Crop Tool Implementation** - Interactive rectangle with handles, aspect ratio constraints
3. **Resize Tool Implementation** - Slider (10-200%) + pixel inputs + presets (50%, 75%, 150%, 200%)
4. **Undo/Redo System** - Transformation history with Ctrl+Z/Ctrl+Y
5. **Save Dialog** - Overwrite vs Save As confirmation
6. **Keyboard Shortcuts**
   - `E` = Enter edit mode (from viewer)
   - `Ctrl+S` = Save
   - `Esc` = Cancel/exit editor
   - `R` = Select rotate tool
   - `C` = Select crop tool
   - `Ctrl+Z` / `Ctrl+Y` = Undo/Redo
7. **Image Display** - Show current_image in editor canvas area
8. **Preview System** - Apply transformations and update preview
9. **Image Saving** - Write transformed image to disk (preserve format)
10. **Tests** - Unit tests for transformation logic
11. **README** - Document editing features

### Technical Notes

**Resize UX:** Hybrid modern approach
- Primary control: Slider (10-200% scale) for intuitive adjustments
- Secondary: Width/Height pixel inputs for precision
- Lock aspect ratio toggle
- Preset buttons: 50%, 75%, 150%, 200%
- Real-time preview (performance permitting)

**Crop UX:** Interactive selection
- Rectangle overlay with 8 handles (4 corners + 4 edges)
- Rule-of-thirds grid overlay
- Aspect ratio constraints with presets
- Live dimension display during adjustment

**Transformation Pipeline:**
```
Original Image → [Apply Transformations] → Current Preview
                        ↓
                  History Stack
                  (for undo/redo)
                        ↓
                  Save to Disk
```

**File Format Preservation:** Save in original format when possible
- JPEG → JPEG
- PNG → PNG
- Fallback to PNG for unsupported formats

### Dependencies

**Current:** No new dependencies yet

**Likely Needed:**
- `image` crate already in use (sufficient for rotate, crop, resize)
- Possible: `imageproc` for advanced operations (future)

### Testing Strategy

**TDD Compliance:** Following project constitution
1. Write tests first for transformation functions
2. Implement transformations to pass tests
3. Integration tests for save/load workflow
4. Manual testing for UX/UI polish

### Open Questions

None currently - design decisions confirmed with user:
- ✅ Option A (separate mode) chosen
- ✅ Side toolbar with retractable option
- ✅ Resize: Real-time preview (Option A)
- ✅ Crop: Free + preset ratios, direct image manipulation
- ✅ Keyboard shortcuts: E, Ctrl+S, Esc, R, C confirmed

---

## Future Editor Features (Post-MVP)

These are potential enhancements beyond the initial rotate/crop/resize:
- Flip (horizontal/vertical)
- Adjust brightness/contrast/saturation
- Filters (grayscale, sepia, blur, sharpen)
- Text overlay
- Drawing tools (arrows, rectangles, annotations)
- Batch operations
- Export to different formats

---

## Notes for Maintainers

**Code Style:**
- Follow existing patterns (Settings, Viewer modules)
- Keep editor logic in `src/ui/editor/`
- Transformation functions should be pure and testable
- Use `#[cfg(test)]` for unit tests

**Localization:**
- All UI strings must have en-US and fr translations
- Keys follow pattern: `editor-{component}-{element}`

**Performance:**
- Large images may need async preview generation
- Consider caching transformed previews
- Monitor memory usage with transformation history

**Security:**
- Validate all user inputs (dimensions, percentages)
- Prevent path traversal in Save As dialog
- Sanitize file names
