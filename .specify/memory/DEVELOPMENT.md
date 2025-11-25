# Development Status

This document tracks ongoing development work for IcedLens. It serves as a reference for features in progress and implementation notes.

**Last Updated:** 2025-11-25 (Session 6 - Crop Base Image System & Button Logic)

---

## 📊 Quick Status Summary

**Overall Progress:** Infrastructure 100% | Features 65%

- ✅ **Infrastructure Complete** - Module, UI, App integration, translations
- ✅ **Sidebar Complete** - Retractable, all controls, intelligent button states
- ✅ **Button Logic** - "← Retour" to exit, "Annuler" to discard, Save/SaveAs, all conditional
- ✅ **Rotate Tool** - Working image pipeline wired, icons corrected, history tracked
- ✅ **Editor Preview** - Canvas now renders the edited image with fit containment
- ✅ **Resize Tool** - Slider, presets, numeric inputs, aspect lock, live preview + auto-commit
- ✅ **Crop Tool** - Aspect ratio presets (Free, Square, 16:9, 9:16, 4:3, 3:4), immediate commit
- ✅ **Crop Base System** - Sequential crops calculated from same base image, prevents distortion
- ⏳ **Remaining** - Undo/Redo wiring, Save implementation, Keyboard shortcuts

**Next Immediate Steps:**
1. Wire Save/Save As file persistence for edited images
2. Hook Undo/Redo stack into toolbar shortcuts (Ctrl+Z/Ctrl+Y)
3. Add keyboard shortcuts (E, Ctrl+S, Esc)

---

## 🚧 In Progress: Image Editor Mode

**Goal:** Add rotate, crop, and resize capabilities to IcedLens while maintaining the clean, minimal UX philosophy.

**User Request:** Save As feature added (create new file vs overwrite original)

### Architecture Overview

**Design Pattern:** Separate editor mode following the existing Settings pattern
- `AppMode::Editor` alongside `Viewer` and `Settings`
- State-down, messages-up architecture
- Non-destructive editing until explicit save

**UI Layout:** Retractable side toolbar with top navigation
```
┌───────────────────────────────────────────────────────┐
│ [← Retour]                                            │
├────────────┬──────────────────────────────────────────┤
│ [☰] Toggle │                                          │
│            │                                          │
│   Tools    │                                          │
│  ┌──────┐  │                                          │
│  │  ↻   │  │         Image preview                    │
│  └──────┘  │         (transformed)                    │
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
│  [◀] [▶]   │  (Navigation - disabled if unsaved)      │
│            │                                          │
│ [Annuler]  │  (Cancel - disabled if no changes)       │
│  [Save]    │  (Disabled if no changes)                │
│ [Save As]  │  (Disabled if no changes)                │
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
   - **Top toolbar**: "← Retour" button to exit editor mode (disabled if unsaved changes)
   - **Retractable sidebar** (hamburger toggle ☰)
   - Width: 180px expanded, 60px collapsed
   - Tool buttons:
     - Rotate Left (↻) / Rotate Right (↺) - instant actions
     - Crop (selectable tool, highlights when active)
     - Resize (selectable tool, highlights when active)
   - Navigation arrows (◀ ▶) for browsing images in editor mode (disabled if unsaved changes)
   - Action buttons (all conditional based on `has_unsaved_changes()`):
     - Annuler (secondary) - discards changes, stays in editor, disabled if no changes
     - Save (primary blue) - overwrites original file, disabled if no changes
     - Save As... (secondary) - creates new file, disabled if no changes
   - Visual hierarchy with horizontal rules
   - Spacer pushing bottom controls to bottom
   - Gray background (#F2F2F2) for contrast

5. **Transformation Module** (`src/image_handler/transform.rs`)
   - Image operations now live under `image_handler` for reuse outside the UI
   - Functions: `rotate_left()`, `rotate_right()`, `resize()` plus `dynamic_to_image_data()` helper
   - Shared tests confirm basic behavior

6. **State Architecture for Transformations**
   - `State` now includes `working_image: DynamicImage`
   - Dual representation:
     - `working_image` (DynamicImage) - for transformations
     - `current_image` (ImageData) - for display in UI
   - After each transformation: working_image → ImageData → update display
   - Custom Debug impl (DynamicImage is not Debug)
   - **Crop base image fields** (added for relative crop calculations):
     - `crop_base_image: Option<DynamicImage>` - Image state when crop tool opened
     - `crop_base_width: u32` - Base image width for ratio calculations
     - `crop_base_height: u32` - Base image height for ratio calculations
     - Updated only when crop tool is selected, preserved across ratio changes

7. **Resize Tool & Preview Pipeline**
   - Slider (10–200%), presets, width/height inputs, and aspect locking wired to a shared preview flow
   - Changes update `preview_image` in real time; toggling away commits via `commit_active_tool_changes()`
   - Tests updated to cover resize application semantics

8. **Sidebar Overflow Handling**
   - Tool list sits inside a vertical scrollable so navigation/save controls remain accessible on short viewports

9. **Crop Tool Implementation**
   - `crop()` function in transform.rs with boundary clamping and validation
   - Tests for crop within bounds, clamping, origin crop, and entire image crop (all passing)
   - CropState structure tracking x, y, width, height, and selected ratio
   - UI panel with 6 aspect ratio buttons (Free, Square 1:1, Landscape 16:9, Portrait 9:16, Photo 4:3, Photo Portrait 3:4)
   - `adjust_crop_to_ratio()` calculates optimal crop dimensions for selected ratio relative to base image
   - `apply_crop_from_base()` applies transformation from the base image saved when crop tool opened
   - **Base Image System**: When crop tool opens, current image is saved as base reference
     - All crop ratio calculations use base image dimensions
     - Sequential crops (e.g., Square then Landscape) both calculated from same base
     - Base only updates when crop tool is closed and reopened
     - Prevents cumulative crop artifacts from chained operations
   - Immediate auto-commit on ratio selection (no preview)
   - Auto-commit on tool change (same pattern as resize)
   - Transformation recorded in history for future undo/redo
   - Translations added for en-US and fr
   - "Crop libre" button is placeholder for future interactive overlay feature

10. **Navigation & Button State Management**
   - **Top toolbar "← Retour" button**: Returns to viewer mode
     - Only enabled when NO unsaved changes exist
     - Prevents accidental loss of work
   - **Sidebar "Annuler" button**: Discards changes but stays in editor
     - Calls `discard_changes()`: reloads original image, clears history, resets states
     - Only enabled when unsaved changes exist
     - Does NOT exit editor mode
   - **Save / Save As buttons**: Write changes to disk
     - Only enabled when unsaved changes exist
     - Prevents redundant save operations
   - **Navigation buttons (◀ ▶)**: Browse to next/previous image
     - Disabled when unsaved changes exist
     - Navigation messages return Event::None when changes are unsaved
     - User MUST save or cancel before navigating
   - All button states driven by `has_unsaved_changes()` based on transformation history

11. **Crop Base Image System** (COMPLETE)
   - Added three fields to State: `crop_base_image`, `crop_base_width`, `crop_base_height`
   - When crop tool opens (Message::SelectTool(EditorTool::Crop)):
     - Current `working_image` is cloned and stored as `crop_base_image`
     - Current dimensions stored in `crop_base_width` and `crop_base_height`
   - All crop ratio calculations use base dimensions via `adjust_crop_to_ratio()`
   - New method `apply_crop_from_base()` applies crop from base image
   - Base image is NOT updated after each crop application
   - Sequential crops always reference the same base image
   - Base only updates when user closes and reopens crop tool
   - Message routing: SetCropRatio → adjust → apply_crop_from_base → immediate commit
   - Prevents cumulative distortion from chained crop operations
   - Tests passing: 98 total (92 lib + 4 main + 2 integration)

#### 🔄 In Progress
Undo/redo wiring and Save/Save As implementation are next priorities.

#### ⏳ To Do
1. **Crop Tool Enhancement (Optional)**
   - Interactive rectangle overlay on image with visual feedback
   - 8 draggable handles (4 corners + 4 edges) for manual resizing
   - Rule-of-thirds grid overlay during crop adjustment
   - Live dimension display during handle dragging

2. **Undo/Redo System (High Priority)**
   - Wire existing transformation history to Undo/Redo buttons
   - Implement undo: replay transformations from original image up to history_index - 1
   - Implement redo: apply transformation at history_index
   - Add Ctrl+Z / Ctrl+Y keyboard shortcuts
   - Update button states based on can_undo() / can_redo()

3. **Save/Save As Implementation (High Priority)**
   - Save: write working_image to original path
   - Save As: file picker dialog → new path
   - Preserve format (JPEG→JPEG, PNG→PNG)
   - Confirmation dialogs
   - Handle errors (write permissions, disk space)

4. **Keyboard Shortcuts**
   - `E` = Enter edit mode (from viewer)
   - `Ctrl+S` = Save
   - `Esc` = Cancel/exit editor
   - `R` = Select rotate tool (if needed)
   - `C` = Select crop tool
   - `Ctrl+Z` / `Ctrl+Y` = Undo/Redo

5. **Navigation in Editor**
   - Wire up NavigateNext/NavigatePrevious events in App
   - Load new image in editor when navigating
   - Prompt to save if unsaved changes exist

6. **Tests** - Unit tests for transformation logic
   - rotate_left/rotate_right dimension swaps
   - crop boundaries
   - transformation history

7. **README** - Document editing features

### Technical Notes

**Image Transformation Architecture:**

The editor uses a dual-representation approach:
```rust
pub struct State {
    working_image: DynamicImage,  // For transformations (image_rs)
    current_image: ImageData,     // For display (iced)
    // ...
}
```

**Flow:**
1. Load file → DynamicImage (working_image)
2. Convert to ImageData (current_image) for display
3. User applies transformation → modify working_image
4. Convert modified working_image → ImageData
5. Update current_image for preview
6. On Save: write working_image to disk

**Why two representations?**
- Iced's `ImageData` uses `Handle` (opaque, can't extract pixels)
- `image_rs::DynamicImage` provides rich transformation API
- Solution: keep both, sync them after each operation

**Resize UX:** Hybrid modern approach (implemented)
- Slider (10-200% scale) for intuitive adjustments
- Width/Height pixel inputs for precision
- Lock aspect ratio toggle
- Preset buttons: 50%, 75%, 150%, 200%
- Real-time preview + auto-commit when leaving the tool

**Crop UX:** Aspect ratio presets with base image system (implemented)
- 6 aspect ratio buttons: Free, Square (1:1), Landscape (16:9), Portrait (9:16), Photo (4:3), Photo Portrait (3:4)
- Immediate application on ratio selection (no preview step)
- **Base image anchoring**: When crop tool opens, current image is saved as base
  - All ratio calculations use base dimensions, not previous crop result
  - Example: 1000×800 image → Crop Square (800×800) → Crop Landscape = 800×450 from original 1000×800, NOT from 800×800
  - Prevents cumulative distortion from sequential crops
  - Base resets only when crop tool is closed and reopened
- Future enhancement: Interactive rectangle overlay with draggable handles

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
