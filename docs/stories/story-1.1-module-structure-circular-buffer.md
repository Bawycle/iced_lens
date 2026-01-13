# Story 1.1: Module Structure and Circular Buffer

**Epic:** 1 - Diagnostics Core & Data Collection
**Status:** Ready
**Priority:** High
**Estimate:** 2-3 hours

---

## Story

**As a** developer,
**I want** a diagnostics module with a circular buffer for storing events,
**So that** I have a foundation for collecting and storing diagnostic data with bounded memory usage.

---

## Acceptance Criteria

1. New `src/diagnostics/` module created with proper module structure
2. `DiagnosticEvent` enum defined to represent different event types (resource snapshot, user action, app state, error)
3. `CircularBuffer<T>` implemented with configurable capacity
4. Buffer correctly overwrites oldest entries when full
5. Buffer provides iterator access to all stored events in chronological order
6. Unit tests verify buffer behavior (add, overflow, iteration)
7. Module exports public API through `mod.rs`

---

## Tasks

- [ ] **Task 0:** Create feature branch
  - [ ] Create branch `feature/diagnostics` from `dev`
  - [ ] Verify branch is active

- [ ] **Task 1:** Create `src/diagnostics/` module directory structure
  - [ ] Create `src/diagnostics/mod.rs` with module exports
  - [ ] Create `src/diagnostics/buffer.rs` for CircularBuffer
  - [ ] Create `src/diagnostics/events.rs` for DiagnosticEvent types
  - [ ] Update `src/lib.rs` to export diagnostics module

- [ ] **Task 2:** Implement `BufferCapacity` newtype
  - [ ] Define newtype with configurable min/max bounds
  - [ ] Add defaults to `src/app/config/defaults.rs`
  - [ ] Implement `new()`, `value()`, `Default`

- [ ] **Task 3:** Implement `CircularBuffer<T>`
  - [ ] Generic struct with VecDeque or custom ring buffer
  - [ ] `push()` method that overwrites oldest on overflow
  - [ ] `iter()` returning chronological iterator
  - [ ] `len()`, `capacity()`, `clear()`, `is_empty()` methods

- [ ] **Task 4:** Define `DiagnosticEvent` enum
  - [ ] `ResourceSnapshot` variant (placeholder for now)
  - [ ] `UserAction` variant (placeholder for now)
  - [ ] `AppState` variant (placeholder for now)
  - [ ] `Warning` and `Error` variants (placeholder for now)
  - [ ] Each variant includes timestamp

- [ ] **Task 5:** Write unit tests
  - [ ] Test buffer push and retrieval
  - [ ] Test overflow behavior (oldest evicted)
  - [ ] Test iterator chronological order
  - [ ] Test capacity bounds via newtype

- [ ] **Task 6:** Run validation
  - [ ] `cargo fmt --all`
  - [ ] `cargo clippy --all --all-targets -- -D warnings`
  - [ ] `cargo test`

- [ ] **Task 7:** Commit changes
  - [ ] Stage all changes
  - [ ] Commit with descriptive message following conventional commits
  - [ ] Reference story number in commit message

---

## Dev Notes

- Follow existing newtype pattern from `src/video_player/frame_cache_size.rs`
- Use `std::collections::VecDeque` for simplicity
- Events will be expanded in subsequent stories
- Reference architecture: `docs/architecture/component-architecture.md`

---

## Testing

### Unit Tests
- `buffer.rs`: add, overflow, iteration, clear, capacity
- `events.rs`: timestamp creation

### Integration Tests
- None for this story (foundation only)

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
