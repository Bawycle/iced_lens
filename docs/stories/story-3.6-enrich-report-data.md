# Story 3.6: Enrich Diagnostic Report Data

**Epic:** 3 - UI Integration
**Status:** Approved
**Priority:** High
**Estimate:** 1-2 days
**Depends On:** Story 2.1 (PathAnonymizer), Story 1.4 (Events)

---

## Story

**As a** developer analyzing diagnostic reports,
**I want** enriched metadata in media events and system information,
**So that** I can better diagnose format-specific issues, network-related slowdowns, and hardware compatibility problems.

---

## Background

This story addresses implementation gaps identified during post-implementation review of Story 3.4:

1. **Media events** (`MediaLoadingStarted`, `MediaLoaded`, `MediaFailed`) lack file extension, storage type, and path correlation data
2. **SystemInfo** is missing `cpu_arch`, `cpu_brand`, and `disk_type` as specified in Story 2.3

See: `docs/prd/report-media-metadata-gap.md` for full analysis and stakeholder validations.

---

## Acceptance Criteria

### Media Event Enrichment

1. `MediaLoadingStarted`, `MediaLoaded`, `MediaFailed` events include new optional fields:
   - `extension: Option<String>` — file extension (e.g., "jpg", "mp4", "heic")
   - `storage_type: StorageType` — enum with `Local`, `Network`, `Unknown` variants
   - `path_hash: Option<String>` — 8-char blake3 hash via existing `PathAnonymizer`

2. `StorageType` enum created in `src/diagnostics/events.rs`:
   ```rust
   pub enum StorageType {
       Local,    // /home, /Users, C:\Users, obvious local paths
       Network,  // UNC paths (\\), detected NFS/SMB mounts
       Unknown,  // Default when detection is uncertain
   }
   ```

3. Storage type detection uses simple heuristics (cross-platform):
   - `Local`: Paths starting with `/home`, `/Users`, `C:\Users`, etc.
   - `Network`: UNC paths (`\\server\share`), known network mount patterns
   - `Unknown`: Default for ambiguous cases (acceptable)

4. `PathAnonymizer` reused from `src/diagnostics/anonymizer.rs` for `path_hash`

### SystemInfo Enrichment

5. `SystemInfo` struct updated with new fields:
   ```rust
   pub struct SystemInfo {
       pub os: String,
       pub os_version: String,
       pub cpu_arch: String,      // "x86_64", "aarch64" via std::env::consts::ARCH
       pub cpu_brand: String,     // Full brand via sysinfo::Cpu::brand()
       pub cpu_cores: usize,      // Existing field preserved
       pub ram_total_mb: u64,
       pub disk_type: Option<DiskType>,
   }
   ```

6. `DiskType` enum created:
   ```rust
   pub enum DiskType {
       Ssd,
       Hdd,
       Unknown,
   }
   ```

7. `disk_type` detected via `sysinfo::Disks::kind()` for the disk containing user's home directory

### Integration Requirements

8. Existing event logging calls updated to pass new metadata where available
9. All new fields are `Option` or have sensible defaults — no breaking changes
10. JSON serialization uses `snake_case` and `skip_serializing_if = "Option::is_none"`
11. Existing tests continue to pass
12. New unit tests cover:
    - `StorageType` detection heuristics
    - `DiskType` mapping from `sysinfo::DiskKind`
    - Serialization of enriched events
    - `SystemInfo::collect()` with new fields

---

## Tasks

### Part A: New Types (AC: 2, 6)

- [ ] **Task 1:** Create `StorageType` enum in `events.rs`
  - [ ] Define enum with `Local`, `Network`, `Unknown` variants
  - [ ] Add serde attributes for snake_case serialization
  - [ ] Implement `StorageType::detect(path: &Path) -> Self` method
  - [ ] Add unit tests for detection heuristics

- [ ] **Task 2:** Create `DiskType` enum in `report.rs`
  - [ ] Define enum with `Ssd`, `Hdd`, `Unknown` variants
  - [ ] Add serde attributes
  - [ ] Implement `From<sysinfo::DiskKind>` conversion
  - [ ] Add unit tests

### Part B: Media Event Enrichment (AC: 1, 3, 4)

- [ ] **Task 3:** Update `AppStateEvent` media variants
  - [ ] Add `extension`, `storage_type`, `path_hash` fields to `MediaLoadingStarted`
  - [ ] Add same fields to `MediaLoaded`
  - [ ] Add `extension`, `storage_type`, `path_hash` to `MediaFailed`
  - [ ] Update serde attributes with `skip_serializing_if`

- [ ] **Task 4:** Create helper for media metadata extraction
  - [ ] Create `MediaMetadata` struct with extraction logic
  - [ ] Implement `MediaMetadata::from_path(path: &Path, anonymizer: &PathAnonymizer) -> Self`
  - [ ] Handle edge cases (no extension, empty path)
  - [ ] Add unit tests

- [ ] **Task 5:** Update event logging call sites in `src/ui/viewer/component.rs`
  - [ ] Update `MediaLoadingStarted` call at line ~648
  - [ ] Update `MediaLoaded` call at line ~831
  - [ ] Update `MediaFailed` call at line ~905
  - [ ] Pass `MediaMetadata` to enriched event constructors
  - [ ] Ensure `PathAnonymizer` instance is available (add to Viewer struct if needed)

### Part C: SystemInfo Enrichment (AC: 5, 7)

- [ ] **Task 6:** Update `SystemInfo` struct
  - [ ] Add `cpu_arch: String` field
  - [ ] Add `cpu_brand: String` field
  - [ ] Add `disk_type: Option<DiskType>` field
  - [ ] Update `SystemInfo::collect()` implementation

- [ ] **Task 7:** Implement disk type detection
  - [ ] Get home directory path
  - [ ] Find disk containing home directory
  - [ ] Map `sysinfo::DiskKind` to `DiskType`
  - [ ] Handle cases where detection fails (return `None`)

### Part D: Testing & Validation (AC: 11, 12)

- [ ] **Task 8:** Add comprehensive tests
  - [ ] Test `StorageType::detect()` with various paths
  - [ ] Test `DiskType` conversion
  - [ ] Test enriched event serialization
  - [ ] Test `SystemInfo::collect()` returns valid data
  - [ ] Verify existing tests still pass

- [ ] **Task 9:** Integration verification
  - [ ] Run full test suite: `cargo test`
  - [ ] Run clippy: `cargo clippy --all --all-targets -- -D warnings`
  - [ ] Verify JSON output format with manual export test

---

## Dev Notes

### Relevant Source Tree

```
src/
├── diagnostics/                    # Target module for new types
│   ├── mod.rs                      # Re-exports public API
│   ├── events.rs                   # ADD: StorageType enum here
│   │                               # MODIFY: MediaLoadingStarted, MediaLoaded, MediaFailed
│   ├── report.rs                   # ADD: DiskType enum here
│   │                               # MODIFY: SystemInfo struct
│   ├── anonymizer.rs               # REUSE: PathAnonymizer for path_hash
│   └── collector.rs                # May need PathAnonymizer instance
├── ui/
│   └── viewer/
│       └── component.rs            # MODIFY: log_state() calls at lines 648, 831, 905
│                                   # ADD: PathAnonymizer to Viewer if needed
└── lib.rs                          # May need to export new types
```

### Key Integration Points

| Location | Line | Event | Action |
|----------|------|-------|--------|
| `src/ui/viewer/component.rs` | ~648 | `MediaLoadingStarted` | Add extension, storage_type, path_hash |
| `src/ui/viewer/component.rs` | ~831 | `MediaLoaded` | Add extension, storage_type, path_hash |
| `src/ui/viewer/component.rs` | ~905 | `MediaFailed` | Add extension, storage_type, path_hash |

### PathAnonymizer Instance Strategy

The Viewer component already has access to `self.diagnostics: Option<DiagnosticsHandle>`. Options:
1. Add `PathAnonymizer` to `DiagnosticsHandle` (recommended)
2. Create `PathAnonymizer` in Viewer and pass to events
3. Create per-event anonymizer (less ideal - different hashes)

**Recommendation:** Option 1 — extend `DiagnosticsHandle` or `DiagnosticsCollector` to hold a shared `PathAnonymizer`.

### Existing Tests Reference

Tests in `src/diagnostics/events.rs` follow pattern:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        // ...
    }
}
```

Tests in `src/diagnostics/report.rs` include `SystemInfo::collect()` verification.

---

## Testing

### Testing Standards

| Aspect | Standard |
|--------|----------|
| **Test Location** | Inline `#[cfg(test)] mod tests` in same file |
| **Framework** | Standard Rust `#[test]` attribute |
| **Assertions** | Use `assert!`, `assert_eq!`, `assert_ne!` |
| **Naming** | `snake_case`, descriptive: `test_storage_type_detects_unc_as_network` |

### Required Test Cases

**StorageType tests (`events.rs`):**
- `test_storage_type_detects_home_as_local`
- `test_storage_type_detects_users_as_local` (macOS)
- `test_storage_type_detects_unc_as_network`
- `test_storage_type_defaults_to_unknown`
- `test_storage_type_serializes_snake_case`

**DiskType tests (`report.rs`):**
- `test_disk_type_from_sysinfo_ssd`
- `test_disk_type_from_sysinfo_hdd`
- `test_disk_type_from_sysinfo_unknown`
- `test_disk_type_serializes_snake_case`

**MediaMetadata tests (`events.rs`):**
- `test_media_metadata_extracts_extension`
- `test_media_metadata_handles_no_extension`
- `test_media_metadata_generates_path_hash`

**SystemInfo tests (`report.rs`):**
- `test_system_info_includes_cpu_arch`
- `test_system_info_includes_cpu_brand`
- `test_system_info_includes_disk_type`

### Validation Commands

```bash
# Run all tests
cargo test

# Run only diagnostics tests
cargo test --lib diagnostics

# Run with verbose output
cargo test -- --nocapture

# Clippy check
cargo clippy --all --all-targets -- -D warnings
```

---

## Technical Notes

### Storage Type Detection Heuristics

```rust
impl StorageType {
    pub fn detect(path: &Path) -> Self {
        let path_str = path.to_string_lossy();

        // Network detection (high confidence)
        if path_str.starts_with("\\\\") {
            return Self::Network;
        }

        // Local detection (high confidence)
        #[cfg(unix)]
        if path_str.starts_with("/home/") || path_str.starts_with("/Users/") {
            return Self::Local;
        }

        #[cfg(windows)]
        if path_str.contains(":\\Users\\") {
            return Self::Local;
        }

        // Default to Unknown for ambiguous paths
        Self::Unknown
    }
}
```

### PathAnonymizer Integration

The `PathAnonymizer` is already instantiated in the anonymization pipeline. For media events:
- Pass the anonymizer to the logging context
- Or create a dedicated instance per session (consistent hashes within session)

### Existing Pattern Reference

- Events pattern: `src/diagnostics/events.rs` (existing media events)
- Anonymizer pattern: `src/diagnostics/anonymizer.rs` (PathAnonymizer)
- SystemInfo pattern: `src/diagnostics/report.rs` (SystemInfo::collect)

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| `storage_type` often `Unknown` | High | Low | Acceptable, refine heuristics later |
| Cross-platform path detection complexity | Medium | Low | Simple heuristics, default to Unknown |
| Breaking change to event structs | Low | Medium | All new fields are Option with defaults |

---

## Definition of Done

- [ ] All acceptance criteria met
- [ ] New `StorageType` and `DiskType` enums implemented
- [ ] Media events enriched with extension, storage_type, path_hash
- [ ] `SystemInfo` enriched with cpu_arch, cpu_brand, disk_type
- [ ] Unit tests pass for all new functionality
- [ ] Existing tests continue to pass
- [ ] `cargo clippy` passes with no warnings
- [ ] JSON export produces valid enriched output
- [ ] Documentation updated (code comments)

---

## References

- Analysis report: `docs/prd/report-media-metadata-gap.md`
- PathAnonymizer: `src/diagnostics/anonymizer.rs`
- Current events: `src/diagnostics/events.rs`
- Current SystemInfo: `src/diagnostics/report.rs`
- Story 2.1: Path Anonymization (PathAnonymizer implementation)
- Story 2.3: JSON Schema Definition (SystemInfo specification)

---

## Change Log

| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2026-01-14 | 1.0 | Initial draft created by PM (John) | John |
| 2026-01-14 | 1.1 | Added Dev Notes, Testing sections; Fixed Task 5 file path | John |
| 2026-01-14 | 1.2 | Story APPROVED after PO validation | Sarah |
