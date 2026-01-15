# Architecture Review: Diagnostics Data Enrichment

**Date:** 2026-01-15
**Author:** Winston (Architect)
**Related Story:** Bug 3.7
**Status:** Recommendation for PO Review

---

## Executive Summary

Following user feedback on diagnostic reports, two areas have been identified where the collected data lacks precision and context. This document provides architectural recommendations for enriching the diagnostic system.

---

## Issue 1: Incomplete OS Identification on Linux

### Current State

```json
"system_info": {
  "os": "linux",
  "os_version": "22.3"
}
```

### Problem

- `os_version` shows "22.3" but without distribution name, this is meaningless
- User running Linux Mint 22.3 cannot be distinguished from other distros with similar version numbers
- Kernel version is not captured, which is relevant for driver-related issues

### Technical Analysis

The `sysinfo` crate (v0.37) already provides the necessary data:

| Method | Returns | Example (Linux Mint) |
|--------|---------|---------------------|
| `System::name()` | Distribution name | `"Linux Mint"` |
| `System::kernel_version()` | Kernel version | `"6.14.0-37-generic"` |
| `System::os_version()` | OS version | `"22.3"` |

**Cross-platform behavior:**

| OS | `name()` | `kernel_version()` | `os_version()` |
|----|----------|-------------------|----------------|
| Linux Mint | "Linux Mint" | "6.14.0-37-generic" | "22.3" |
| Ubuntu | "Ubuntu" | "6.8.0-45-generic" | "24.04" |
| Windows 11 | "Windows 11" | "22631" | "23H2" |
| macOS | "macOS" | "24.0.0" | "15.0" |

### Recommendation

**Add two new fields to `SystemInfo` struct:**

```rust
// src/diagnostics/report.rs
pub struct SystemInfo {
    pub os: String,
    pub os_name: String,         // NEW: Human-readable OS name
    pub os_version: String,
    pub kernel_version: String,  // NEW: Kernel/build version
    pub cpu_arch: String,
    pub cpu_brand: String,
    pub cpu_cores: usize,
    pub ram_total_mb: u64,
    pub disk_type: Option<DiskType>,
}
```

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

### Impact Assessment

| Aspect | Impact |
|--------|--------|
| Code changes | ~10 lines in `report.rs` |
| Breaking change | No (additive fields) |
| Performance | Negligible (single call at report generation) |
| Dependencies | None (already using sysinfo) |

---

## Issue 2: Vague Media Size Categorization

### Current State

```json
{
  "type": "app_state",
  "state": "media_loaded",
  "media_type": "image",
  "size_category": "small"
}
```

### Problem

1. **"small" is ambiguous** — Could be 100 KB or 900 KB
2. **Same thresholds for all media types** — A 5 MB image is large, a 5 MB video is tiny
3. **Missing dimensions** — File size alone doesn't indicate memory/GPU load

### Technical Analysis

**Current thresholds (same for images and videos):**

| Category | File Size |
|----------|-----------|
| Small | < 1 MB |
| Medium | 1 MB – 10 MB |
| Large | 10 MB – 100 MB |
| VeryLarge | >= 100 MB |

**Why this matters for diagnostics:**

| Metric | Indicates | Useful For |
|--------|-----------|------------|
| File size | Disk I/O load | Loading time analysis |
| Dimensions | Memory/GPU load | Rendering issues, OOM errors |

A compressed 4K image (3840×2160) might be only 2 MB ("medium") but requires 33 MB in memory when decoded (4 bytes/pixel RGBA).

### Recommendation

**Option A (Preferred): Replace category with raw data**

```json
{
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

**Advantages:**
- Objective data, no interpretation
- Enables post-hoc analysis with adjustable thresholds
- Dimensions reveal true memory impact
- Future-proof (thresholds can be adjusted without schema change)

**Option B (Minimal): Keep category but make it explicit**

```json
{
  "size_category": "small",
  "file_size_bytes": 524288
}
```

**Option C: Type-specific thresholds**

| Category | Image | Video |
|----------|-------|-------|
| Small | < 500 KB | < 10 MB |
| Medium | 500 KB – 5 MB | 10 MB – 100 MB |
| Large | 5 MB – 50 MB | 100 MB – 1 GB |
| VeryLarge | >= 50 MB | >= 1 GB |

### Impact Assessment

| Aspect | Option A | Option B | Option C |
|--------|----------|----------|----------|
| Code changes | Moderate | Minimal | Moderate |
| Schema change | Yes (additive) | Yes (additive) | No |
| Diagnostic value | High | Medium | Medium |
| Implementation effort | ~2 hours | ~30 min | ~1 hour |

---

## Implementation Recommendations

### Priority Matrix

| Enhancement | Diagnostic Value | Effort | Priority |
|-------------|-----------------|--------|----------|
| Add `os_name` | High | Low | **P1** |
| Add `kernel_version` | Medium | Low | **P2** |
| Add `file_size_bytes` | High | Low | **P1** |
| Add `dimensions` | High | Medium | **P2** |
| Remove `size_category` | Low | Low | Optional |

### Suggested Approach

**Minimal viable enhancement (can be added to Story 3.7):**
1. Add `os_name` field
2. Add `file_size_bytes` field (keep `size_category` for backward compatibility)

**Full enhancement (separate Story 3.8):**
1. Add `os_name`, `kernel_version`
2. Add `file_size_bytes`, `dimensions`
3. Deprecate `size_category` (keep for one version, then remove)

---

## Files to Modify

| File | Changes |
|------|---------|
| `src/diagnostics/report.rs` | Add `os_name`, `kernel_version` to `SystemInfo` |
| `src/diagnostics/events.rs` | Add `file_size_bytes`, `dimensions` to media events |
| `src/ui/viewer/component.rs` | Pass dimensions when logging media events |
| `src/app/mod.rs` | Pass dimensions for AI operation events |

---

## Acceptance Criteria (Proposed)

### For OS Information

- [ ] Diagnostic reports include `os_name` showing distribution/OS name
- [ ] Diagnostic reports include `kernel_version`
- [ ] All platforms (Linux, Windows, macOS) populate these fields correctly
- [ ] Falls back to "unknown" if data unavailable

### For Media Size

- [ ] Diagnostic events include `file_size_bytes` as exact value
- [ ] Image events include `dimensions` object with `width` and `height`
- [ ] Video events include `dimensions` when available from metadata
- [ ] Existing `size_category` field remains for backward compatibility

---

## Questions for PO

1. Should these enhancements be added to Story 3.7 or created as a separate Story 3.8?
2. Is backward compatibility with existing diagnostic reports required?
3. For media dimensions: should we include video dimensions (requires reading metadata)?
4. Priority preference between OS info enrichment vs media size enrichment?

---

*Report generated by Winston (Architect Agent)*
