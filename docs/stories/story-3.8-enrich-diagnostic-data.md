# Story 3.8: Enrich Diagnostic Report Data

**Epic:** 3 - UI Integration
**Status:** Approved
**Priority:** Medium
**Estimate:** 2 hours
**Depends On:** Story 3.6 (Diagnostic Events)

---

## User Story

As a **developer or support person** analyzing diagnostic reports,
I want **precise system and media information** in the reports,
So that **I can accurately identify environment-specific issues and performance bottlenecks**.

---

## Story Context

**Existing System Integration:**

- Integrates with: `src/diagnostics/` module (report.rs, events.rs)
- Technology: Rust, sysinfo crate (v0.37)
- Follows pattern: Existing `SystemInfo::collect()` and `SizeCategory` patterns
- Touch points: `report.rs`, `events.rs`, `viewer/component.rs`, `app/mod.rs`

**Background:**

The diagnostic system was introduced in Epic 3 (Stories 3.2-3.6). User feedback identified two data gaps:

1. **OS identification on Linux** — Version "22.3" without distribution name is meaningless
2. **Media size categorization** — "small" category is ambiguous; dimensions are missing

---

## Acceptance Criteria

### OS Information Enrichment

- [ ] `SystemInfo` includes `os_name` field populated via `System::name()`
- [ ] `SystemInfo` includes `kernel_version` field populated via `System::kernel_version()`
- [ ] All platforms (Linux, Windows, macOS) populate these fields correctly
- [ ] Fields fall back to `"unknown"` if data unavailable

**Expected output:**

```json
"system_info": {
  "os": "linux",
  "os_name": "Linux Mint",
  "os_version": "22.3",
  "kernel_version": "6.14.0-37-generic",
  "cpu_arch": "x86_64",
  ...
}
```

### Media Size Data Enrichment

- [ ] `SizeCategory` enum is removed from `events.rs`
- [ ] Media events include `file_size_bytes: u64` with exact file size
- [ ] Image events include `dimensions: { width: u32, height: u32 }` when available
- [ ] Video events include `dimensions` when available from loaded media (not from metadata parsing)
- [ ] Events where dimensions are unavailable use `dimensions: null`

**Expected output:**

```json
{
  "timestamp_ms": 3938,
  "type": "app_state",
  "state": "media_loaded",
  "media_type": "image",
  "file_size_bytes": 524288,
  "dimensions": { "width": 1920, "height": 1080 },
  "extension": "png",
  "storage_type": "local",
  "path_hash": "d19b0e31"
}
```

### Code Cleanup

- [ ] All usages of `SizeCategory` are replaced with new fields
- [ ] `SizeCategory::from_bytes()` function is removed
- [ ] No dead code remains

---

## Tasks

### OS Information Enrichment
- [ ] Add `os_name: String` field to `SystemInfo` struct (AC: 1)
- [ ] Add `kernel_version: String` field to `SystemInfo` struct (AC: 2)
- [ ] Update `SystemInfo::collect()` to populate new fields via `System::name()` and `System::kernel_version()`
- [ ] Add fallback to `"unknown"` for both fields (AC: 4)

### Media Size Enrichment
- [ ] Create `Dimensions` struct in `events.rs` with `width: u32`, `height: u32`
- [ ] Add `file_size_bytes: u64` to media event structs (AC: 6)
- [ ] Add `dimensions: Option<Dimensions>` to media event structs (AC: 7, 8, 9)
- [ ] Remove `SizeCategory` enum entirely (AC: 5)
- [ ] Update `viewer/component.rs` to pass `file_size_bytes` and `dimensions` when logging events
- [ ] Update `app/mod.rs` AI operations to pass `file_size_bytes` and `dimensions`
- [ ] Update `diagnostics/mod.rs` exports

### Testing & Cleanup
- [ ] Update/add unit tests for `SystemInfo::collect()` (AC: 3)
- [ ] Update/add unit tests for `Dimensions` serialization
- [ ] Update/add unit tests for media event serialization with new fields
- [ ] Fix all compiler errors from `SizeCategory` removal (AC: 10, 11)
- [ ] Run `cargo test` — all tests pass
- [ ] Run `cargo clippy --all --all-targets -- -D warnings` — no warnings (AC: 12)
- [ ] Manual test: export diagnostic report, verify new fields present

---

## Technical Notes

### Integration Approach

**For SystemInfo (`report.rs`):**

```rust
pub struct SystemInfo {
    pub os: String,
    pub os_name: String,         // NEW: System::name()
    pub os_version: String,
    pub kernel_version: String,  // NEW: System::kernel_version()
    // ... existing fields unchanged
}
```

**For Media Events (`events.rs`):**

```rust
// NEW: Dimensions struct
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

// REMOVE: SizeCategory enum entirely

// UPDATE: Event structs to use new fields
pub struct MediaLoadedEvent {
    pub file_size_bytes: u64,              // Replaces size_category
    pub dimensions: Option<Dimensions>,     // NEW
    // ... other fields
}
```

### Files to Modify

| File | Changes |
|------|---------|
| `src/diagnostics/report.rs` | Add `os_name`, `kernel_version` to `SystemInfo` |
| `src/diagnostics/events.rs` | Remove `SizeCategory`, add `Dimensions`, update event structs |
| `src/diagnostics/mod.rs` | Update exports if needed |
| `src/ui/viewer/component.rs` | Pass `file_size_bytes` and `dimensions` when logging events |
| `src/app/mod.rs` | Update AI operation event logging |

### Existing Pattern Reference

- `SystemInfo::collect()` at `report.rs:113-134` — extend with new fields
- Media event logging at `viewer/component.rs:634-640` — update to pass new data

### Key Constraints

- `sysinfo` crate already provides all needed OS data — no new dependencies
- Dimensions for images are already available when media is loaded
- Video dimensions available from `VideoState` when video is loaded
- No file metadata parsing required for dimensions (use already-loaded data)

---

## Definition of Done

- [ ] All acceptance criteria met
- [ ] `cargo test` passes
- [ ] `cargo clippy --all --all-targets -- -D warnings` passes
- [ ] Manual verification: export diagnostic report and confirm new fields present
- [ ] No regression in existing diagnostic functionality

---

## Testing

### Manual Test

1. Start app, navigate several images
2. Load a video
3. Open Diagnostics → Start Collection → perform actions → Stop → Export
4. Verify JSON report contains:
   - `system_info.os_name` (e.g., "Linux Mint", "Windows 11", "macOS")
   - `system_info.kernel_version` (e.g., "6.14.0-37-generic")
   - Media events with `file_size_bytes` (exact number, not category)
   - Image events with `dimensions` object
   - Video events with `dimensions` when available

### Unit Tests

- Test `SystemInfo::collect()` returns non-empty `os_name` and `kernel_version`
- Test `Dimensions` serialization
- Test media event serialization with new fields

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| `System::name()` returns `None` on some platforms | Low | Fallback to "unknown" |
| Dimensions unavailable for some media | Low | Use `Option<Dimensions>` with `null` serialization |
| Breaking change in event schema | None | This is a new feature, no backward compatibility needed |

---

## Out of Scope

- Video dimensions from file metadata parsing (would require additional dependency)
- Memory size estimation (width × height × 4 bytes) — can be calculated from dimensions if needed
- Type-specific size thresholds — raw data allows analysis tools to apply any thresholds

---

## References

- Architecture Review: `docs/architecture-review-diagnostics-enrichment.md`
- Story 3.6: Diagnostic event logging implementation
- sysinfo crate docs: `System::name()`, `System::kernel_version()`

---

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2026-01-15 | Story created from architecture review | Sarah (PO) |
| 2026-01-15 | Added Tasks section after validation review | Sarah (PO) |
| 2026-01-15 | Story approved for implementation | Sarah (PO) |
