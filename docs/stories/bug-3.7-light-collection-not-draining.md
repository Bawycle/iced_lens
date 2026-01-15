# Bug 3.7: Light Collection Events Not Draining to Buffer

**Epic:** 3 - UI Integration
**Status:** Ready for Review
**Priority:** High
**Estimate:** 1 hour
**Depends On:** Story 3.2

---

## Bug Description

**Reported by:** User
**Found in:** Epic 3 implementation

Diagnostic events logged via `DiagnosticsHandle` (light collection) are not being drained from the channel to the buffer. The "Buffer: X events" counter on the Diagnostics screen stays at 0 even after user navigation actions that should log events.

---

## Root Cause Analysis

### Expected Flow
1. Events logged via `DiagnosticsHandle.log_action()` → sent to `event_tx` channel
2. `DiagnosticsCollector.process_pending()` drains `event_rx` channel → buffer
3. Buffer count displayed on Diagnostics screen

### Actual Flow
- Step 1 works correctly
- Step 2 **never happens** in normal usage
- Buffer stays empty

### Technical Details

`process_pending()` is only called in `Message::Tick` handler (`src/app/mod.rs:663`).

However, `Message::Tick` is only emitted when specific conditions are met (`src/app/subscription.rs:154-164`):

```rust
pub fn create_tick_subscription(
    fullscreen: bool,
    is_loading: bool,
    has_notifications: bool,
) -> Subscription<Message> {
    if fullscreen || is_loading || has_notifications {
        time::every(std::time::Duration::from_millis(100)).map(Message::Tick)
    } else {
        Subscription::none()  // <-- No tick when all conditions are false
    }
}
```

When `fullscreen`, `is_loading`, and `has_notifications` are all false:
- `Subscription::none()` is returned
- `Message::Tick` is never emitted
- `process_pending()` is never called
- Events remain stuck in the channel

---

## Acceptance Criteria

1. Light collection events (UserAction, AppState, Warning, Error) are drained to buffer
2. Buffer count updates when user navigates to Diagnostics screen
3. Events collected before opening Diagnostics screen are visible
4. No performance regression (don't add unnecessary polling)

---

## Proposed Fix

Call `process_pending()` when handling Diagnostics messages, specifically when `RefreshStatus` fires (every 1 second on Diagnostics screen).

### Location
`src/app/mod.rs`, in the `Message::Diagnostics` handling block (around line 536)

### Change
```rust
// Handle diagnostics exports before creating context (need access to self.notifications)
if let Message::Diagnostics(ref diagnostics_message) = message {
    // Always drain pending events when handling diagnostics messages.
    // This ensures the buffer is up-to-date even when Tick isn't firing
    // (Tick only fires when fullscreen/loading/notifications are active).
    self.diagnostics.process_pending();

    match diagnostics_message {
        // ... existing code
    }
}
```

### Why This Fix
- `RefreshStatus` fires every 1 second when on Diagnostics screen
- Events are drained right before displaying the updated count
- No polling added when not on Diagnostics screen
- Minimal code change, low risk

---

## Testing

### Manual Test
1. Start app fresh
2. Navigate several images (Next/Previous)
3. Open hamburger menu → Settings → back
4. Open hamburger menu → Diagnostics
5. **Expected:** Buffer count > 0
6. **Before fix:** Buffer count = 0

### Unit Test (optional)
Could add a test that verifies `process_pending()` is called during Diagnostics message handling, but existing tests cover the individual components.

---

## Files to Modify

- `src/app/mod.rs` - Add `process_pending()` call in Diagnostics message handling

---

## Dev Notes

### Alternative Fixes Considered

1. **Always run Tick** - Would work but adds unnecessary CPU usage when idle
2. **Add condition to Tick subscription** - More complex, would need to track "has pending events"
3. **Call in RefreshStatus handler only** - Current fix does this implicitly since all Diagnostics messages go through the same block

---

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2026-01-15 | Bug identified and documented | Sarah (PO) |
| 2026-01-15 | Bug fix implemented | James (Dev) |

---

## Dev Agent Record

### Agent Model Used
Claude Opus 4.5 (claude-opus-4-5-20251101)

### Debug Log References
N/A - No debug log entries needed for this fix.

### Completion Notes
- Added `self.diagnostics.process_pending()` call at the start of `Message::Diagnostics` handling block in `src/app/mod.rs`
- This ensures events are drained from the channel to the buffer whenever any Diagnostics message is handled
- `RefreshStatus` fires every 1 second on the Diagnostics screen, so buffer will be updated regularly
- All 922 unit tests + 26 doc tests pass
- Clippy lint check passes with no warnings

### File List

| File | Action |
|------|--------|
| `src/app/mod.rs` | Modified - Added `process_pending()` call in Diagnostics message handler |

---

## QA Results

### Review Date: 2026-01-15

### Reviewed By: Quinn (Test Architect)

### Risk Assessment

**Risk Level: LOW**
- Minimal code change (~5 lines including comment)
- Non-critical path (diagnostics feature, not core functionality)
- No security/auth/payment changes
- No new dependencies
- Existing comprehensive tests for affected components

### Code Quality Assessment

**Rating: EXCELLENT**

The implementation is clean, minimal, and precisely addresses the root cause:

1. **Correct Placement**: `process_pending()` is called at the start of the `Message::Diagnostics` block (line 540), ensuring events are drained before any diagnostic operation.

2. **Clear Documentation**: The comment explains WHY the fix is needed (Tick only fires conditionally), making future maintenance easier.

3. **Design Choice**: Calling `process_pending()` on ALL Diagnostics messages (not just `RefreshStatus`) is actually better - ensures fresh buffer before exports, toggles, etc.

4. **Idempotency**: Multiple calls to `process_pending()` are safe (non-blocking `try_recv` loop).

### Refactoring Performed

None required - implementation is clean and follows existing patterns.

### Compliance Check

- Coding Standards: ✓ Follows existing patterns, proper comment style
- Project Structure: ✓ Change in correct location per story spec
- Testing Strategy: ✓ Existing unit tests cover `process_pending()` behavior
- All ACs Met: ✓ See validation below

### Acceptance Criteria Validation

| AC# | Criteria | Status | Evidence |
|-----|----------|--------|----------|
| 1 | Light collection events drained to buffer | ✓ PASS | `process_pending()` drains channel to buffer |
| 2 | Buffer count updates on Diagnostics screen | ✓ PASS | Called before any display update |
| 3 | Events before opening visible | ✓ PASS | Drained on first Diagnostics message |
| 4 | No performance regression | ✓ PASS | No polling added, only on-demand draining |

### Test Architecture Assessment

- **Existing Coverage**: Comprehensive unit tests for `process_pending()` in `collector.rs` (25+ test cases)
- **Test Levels**: Appropriate - unit tests for component, manual test for integration
- **Gap Analysis**: Story correctly identifies unit test as "optional" - component behavior is well-tested

### Improvements Checklist

- [x] Implementation matches proposed fix exactly
- [x] Comment explains the rationale clearly
- [x] No unnecessary complexity introduced
- [ ] (Optional) Add integration test for message handler calling `process_pending()`

### Security Review

N/A - No user input handling, no data exposure, no auth changes.

### Performance Considerations

- `process_pending()` is non-blocking (`try_recv` loop)
- Only called when on Diagnostics screen (via message handling)
- No polling or timers added
- Zero overhead when not using Diagnostics feature

### Files Modified During Review

None - no refactoring required.

### Gate Status

Gate: **PASS** → `docs/qa/gates/3.7-light-collection-not-draining.yml`

### Recommended Status

✓ **Ready for Done** - All acceptance criteria met, clean implementation, tests passing.

