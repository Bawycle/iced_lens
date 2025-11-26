# Editor Refactor Plan

## Goals
- Split `src/ui/editor/mod.rs` (~2.4k lines) into smaller components without changing existing features (rotate, crop, resize, overlays, undo/redo, navigation, save flows).
- Adopt the same "state down, messages up" layering used by the viewer: container orchestrates state, sub-components receive read-only view + callbacks, messages bubble back to the parent.
- Maintain parity with current UX and translation keys; avoid regressions in crop/resize math or keyboard shortcuts.

## Current Module Map
| Area | Responsibilities | Key Functions/Types |
| --- | --- | --- |
| Public API | Exports `State`, `ViewContext`, `Message`, `Event` | type defs at top of file |
| View assembly | Renders toolbar, sidebar, image area, overlays | `State::view`, `build_sidebar`, `build_resize_panel`, `build_crop_panel`, `build_crop_ratio_button` |
| Update loop | Central message handling & tool routing | `State::update` (862), helpers for undo/redo, save, navigation |
| Resize logic | State, overlay, inputs | `ResizeState`, `ResizeOverlay`, `build_resize_panel`, `set_resize_percent`, `handle_width_input_change`, `apply_resize_dimensions`, etc. |
| Crop logic | State, overlay events, canvas renderer | `CropState`, `CropOverlay`, `CropOverlayRenderer`, `handle_crop_overlay_*`, `apply_crop_from_base`, etc. |
| Geometry helpers | Base dimensions, aspect calculations | `base_width`, `base_height`, `adjust_crop_to_ratio`, `update_crop_from_handle_drag`, `apply_aspect_ratio_constraint_to_current_crop` |
| Image persistence | Save/discard/replay transformations | `save_image`, `discard_changes`, `replay_transformations_up_to_index` |
| Tests | Unit tests for state constructors and operations | bottom of file (2337+) |

The single file currently mixes UI rendering, tool state machines, and IO helpers, which makes local changes risky and slows compile times.

## Target Structure
```
src/ui/editor/
├── component.rs        # public view/update entrypoints, wires sub-modules
├── mod.rs              # re-exports + facade (thin)
├── state/
│   ├── mod.rs          # shared EditorState (image, history, routing)
│   ├── crop.rs         # CropState, overlay state machine, helpers
│   └── resize.rs       # ResizeState, overlay logic, inputs
├── view/
│   ├── toolbar.rs      # back/save buttons, shortcuts
│   ├── sidebar/
│   │   ├── mod.rs
│   │   ├── crop_panel.rs
│   │   └── resize_panel.rs
│   └── canvas.rs       # image display + overlay composition
├── overlay/
│   ├── crop.rs         # Canvas Program impl
│   └── resize.rs
└── tests/
    └── mod.rs          # move existing unit tests here (or keep in main file temporarily)
```
- `component.rs` exports `Message`/`Event` enums and routes sub-messages.
- State submodules own their data + domain-specific helper methods; editor `State` holds instances and exposes immutable views for rendering.
- View submodules take `&ViewContext` + lightweight view models; they never mutate state directly—only emit messages.

## Incremental Plan
1. **Audit & Baseline** ✅ (structure captured here; `cargo clippy`/`cargo test` were green before refactor work started).

2. **State Extraction** ✅ (`state/{crop,resize}.rs` now own tool-specific data & helpers; imports updated in `mod.rs`, tests kept passing).

3. **View Split** ✅ (toolbar/sidebar/canvas/crop+resize panels live under `view/`, driven by lightweight view models).

4. **Message Routing Cleanup** ✅ (`Message::{Toolbar,Sidebar,Canvas}` with dedicated handlers landed; state update logic now delegates per area).

5. **Overlay Modules** ✅ (`overlay/{crop,resize}.rs` host the canvas programs; `view/canvas.rs` consumes them, keeping drawing logic isolated).

6. **Testing & Regression Guarding** ✅
   - Automated: latest run `cargo fmt && cargo clippy && cargo test editor` (2025-11-26) passes.
   - Manual QA (2025-11-26): verified crop drag/ratio, resize inputs & lock toggle, rotate + undo/redo, navigation/save/back buttons, and keyboard shortcuts (undo/redo, navigate, save) in the UI.

> Continue tracking new regressions in a follow-up QA checklist if additional editor functionality changes beyond this plan.

## Risk Mitigation
- Work in small commits per step to ease review and rollback.
- Preserve existing public functions until the final step to avoid API churn.
- Keep unit tests close to the logic they cover; when moving code, move its tests to the same module.
- Use integration tests (already in `tests/integration.rs`) as final verification.

## Follow-up Opportunities
The core refactor is complete, but we identified additional improvements that keep the "state down / messages up" layering intact while reducing module size and clarifying ownership:

1. **Tool-specific handlers (in progress)**: crop/resize helper methods now live in `state/{crop,resize}.rs`, and overlay/tool activation logic is being encapsulated there. Next step is to route the sidebar/canvas message arms through these helpers so `State::update` simply dispatches per tool.
2. **History subsystem**: extract `Transformation`, undo/redo bookkeeping, and replay logic into `state/history.rs` to simplify the main state struct and enable focused tests.
3. **Scoped messages**: define toolbar/sidebar/canvas message enums next to their view modules, re-export them from the facade, and keep routing centralized—this limits cross-module imports and clarifies who can emit what.
4. **Shared helpers/services**: relocate utilities like `parse_dimension_input`, crop math, and image conversions into dedicated helpers (e.g., `state/utils.rs` or `image_ops.rs`) to avoid bloating the primary impl block.
5. **Targeted tests**: as modules split, co-locate new unit tests with each subsystem (crop ratios, resize clamping, history replay) to guard against regressions without relying solely on the large integration test.

These can land incrementally; each preserves the existing public API and continues to funnel state updates/messages through the current editor facade.

## Recent Progress (2025-11-26)
- Offloaded crop and resize helper implementations into their respective state modules, shrinking `mod.rs` while keeping the facade unchanged.
- Added state-level hooks for showing/hiding overlays and initializing crop sessions, paving the way for delegating sidebar/canvas message branches into the tool modules.
