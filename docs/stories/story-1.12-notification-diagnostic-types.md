# Story 1.12: Notification Diagnostic Types Migration

**Epic:** 1 - Diagnostics Core & Data Collection
**Status:** Done
**Priority:** Low
**Estimate:** 2 hours
**Depends On:** Story 1.7

---

## Story

**As a** developer,
**I want** all notification creation sites to use explicit diagnostic types,
**So that** warning/error categorization is reliable and maintainable rather than relying on string pattern inference.

---

## Background

Story 1.7 introduced `with_warning_type()` and `with_error_type()` builder methods to `Notification`, enabling explicit diagnostic categorization. However, only 4 sites were migrated as examples. The remaining **46 sites** (28 in mod.rs, 14 in update.rs, 4 in persistence.rs) still rely on fallback string matching inference in `manager.rs`.

While the fallback works correctly, explicit types are:
- More reliable (no pattern matching fragility)
- Self-documenting (type is visible at creation site)
- Easier to maintain (no need to update inference patterns)

---

## Acceptance Criteria

1. All `Notification::warning()` calls include `.with_warning_type()`
2. All `Notification::error()` calls include `.with_error_type()`
3. Fallback inference functions can be marked as deprecated or removed
4. No behavior change (same types as inferred before)

---

## Tasks

### Task 1: Migrate src/app/mod.rs (AC: 1, 2, 4)
**28 sites** migrated:
- [x] Config save warnings → `WarningType::ConfigurationIssue`
- [x] Model download/validation errors → `ErrorType::AIModelError`
- [x] Persistence warnings → `WarningType::ConfigurationIssue`
- [x] Scan dir warning → `WarningType::Other`
- [x] Save/export errors → `ErrorType::ExportError`
- [x] Load errors → `ErrorType::IoError`, `ErrorType::DecodeError`
- [x] Skipped files warnings → `WarningType::UnsupportedFormat`
- [x] Editor errors → `ErrorType::InternalError`

### Task 2: Migrate src/app/update.rs (AC: 1, 2, 4)
**14 sites** migrated:
- [x] Persistence warnings → `WarningType::ConfigurationIssue`
- [x] Load errors → `ErrorType::DecodeError`
- [x] Skipped files warnings → `WarningType::UnsupportedFormat`
- [x] Scan dir/editor errors → `WarningType::Other`, `ErrorType::InternalError`
- [x] Delete error → `ErrorType::IoError`
- [x] Metadata errors → `ErrorType::IoError`, `ErrorType::Other`

### Task 3: Migrate src/app/persistence.rs (AC: 1, 2, 4)
**4 sites** migrated:
- [x] All config load/save warnings → `WarningType::ConfigurationIssue`

### Task 4: Review and Clean Up (AC: 3)
- [x] Run grep command to verify 0 untyped warnings/errors remain
- [x] Remove `infer_warning_type()` and `infer_error_type()` functions (no longer needed)
- [x] Update `push()` docstring to reflect explicit typing requirement

### Task 5: Run Validation
- [x] `cargo fmt --all`
- [x] `cargo clippy --all --all-targets -- -D warnings`
- [x] `cargo test` (all 864 tests pass)

### Task 6: Commit Changes
- [x] Stage all changes
- [x] Commit with message: `refactor(notifications): migrate to explicit diagnostic types [Story 1.12]`

---

## Dev Notes

### Migration Pattern

**BEFORE:**
```rust
ctx.notifications.push(notifications::Notification::warning(&key));
```

**AFTER:**
```rust
ctx.notifications.push(
    notifications::Notification::warning(&key)
        .with_warning_type(WarningType::ConfigurationIssue)
);
```

**BEFORE (with args):**
```rust
self.notifications.push(notifications::Notification::error(
    "notification-deblur-download-error",
).with_arg("error", e));
```

**AFTER:**
```rust
self.notifications.push(
    notifications::Notification::error("notification-deblur-download-error")
        .with_error_type(ErrorType::AIModelError)
        .with_arg("error", e)
);
```

### Required Import

```rust
use crate::diagnostics::{WarningType, ErrorType};
```

### Recommended Type Mapping

| Message key pattern | Type |
|---------------------|------|
| `*-io-*`, `*-read-*`, `*-write-*` | `ErrorType::IoError` |
| `*-decode-*`, `*-load-error-*` | `ErrorType::DecodeError` |
| `*-save-*`, `*-export-*` | `ErrorType::ExportError` |
| `*-deblur-*`, `*-upscale-*`, `*-ai-*`, `*-model-*` | `ErrorType::AIModelError` |
| `*-internal-*` | `ErrorType::InternalError` |
| `*-unsupported-*`, `*-format-*`, `*-skipped-*` | `WarningType::UnsupportedFormat` |
| `*-config-*`, `*-setting-*`, `*-persist-*`, `*-preferences-*` | `WarningType::ConfigurationIssue` |
| `*-permission-*`, `*-access-*` | `WarningType::PermissionDenied` |
| `*-not-found-*`, `*-missing-*` | `WarningType::FileNotFound` |
| `*-network-*`, `*-download-*` | `WarningType::NetworkError` |
| `*-metadata-*`, `*-clip-*`, `*-delete-*` | `ErrorType::IoError` or `WarningType::Other` |

For keys that don't match any pattern, use `WarningType::Other` or `ErrorType::Other`.

### Already Migrated (from Story 1.7)

These 4 sites are already done (skip them):
- `notification-deblur-apply-error` → `ErrorType::AIModelError` (mod.rs:780)
- `notification-upscale-resize-error` → `ErrorType::AIModelError` (mod.rs:814)
- `notification-save-error` → `ErrorType::ExportError` (update.rs:726)
- `notification-video-editing-unsupported` → `WarningType::UnsupportedFormat`

### Grep Command to Find Unmigrated Sites

```bash
# Find all unmigrated sites
grep -rn "Notification::\(warning\|error\)(" src/app/ | grep -v "with_warning_type\|with_error_type"

# After migration, this should return 0 results (excluding tests)
```

### Verification Command

```bash
# Count remaining unmigrated sites (should be 0 after migration)
grep -rn "Notification::\(warning\|error\)(" src/app/ | grep -v "with_warning_type\|with_error_type\|#\[test\]" | wc -l
```

---

## Testing

### No New Tests Required

This is a pure refactoring story with no behavior change:
- Existing tests in `src/ui/notifications/` verify the builder pattern works
- Existing tests in `src/diagnostics/collector.rs` verify warning/error capture
- The `infer_*_type()` functions remain as fallback (deprecated, not removed)

### Verification Steps

1. Run full test suite: `cargo test`
2. Run grep verification command (should return 0)
3. Manual spot-check: trigger a few notifications and verify they appear correctly

---

## Dev Agent Record

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2026-01-13 | Story created from PO review of Story 1.7 | PO (Sarah) |
| 2026-01-13 | PO Validation: Updated estimates (46 sites), added migration patterns, Task-AC mappings, Testing section | PO Validation |

---
