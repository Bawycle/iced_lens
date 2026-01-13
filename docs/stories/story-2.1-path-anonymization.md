# Story 2.1: Path Anonymization

**Epic:** 2 - Anonymization & Export System
**Status:** Done
**Priority:** High
**Estimate:** 2-3 hours
**Depends On:** Epic 1 complete

---

## Story

**As a** developer,
**I want** file paths to be anonymized in diagnostic reports,
**So that** user privacy is protected while preserving diagnostic value.

---

## Acceptance Criteria

1. `PathAnonymizer` struct implemented with hash-based anonymization
2. File paths hashed using blake3 (fast, secure)
3. File extensions preserved (e.g., `/home/user/photos/image.jpg` → `a1b2c3d4.jpg`)
4. Directory structure depth preserved but names hashed (e.g., `hash1/hash2/hash3.jpg`)
5. Consistent hashing within a session (same path = same hash)
6. Hash is one-way (cannot reverse to original path)
7. Unit tests verify anonymization correctness and consistency

---

## Tasks

### Task 1: Create `src/diagnostics/anonymizer.rs` (AC: 1)
- [x] Define `PathAnonymizer` struct with session salt field
- [x] Store salt as `[u8; 32]` generated via `rand` or fixed seed
- [x] Export from `mod.rs`

### Task 2: Implement path hashing (AC: 2, 3, 4)
- [x] Use blake3 (already in `Cargo.toml`)
- [x] Hash each path segment separately (directory names + filename)
- [x] Preserve file extension only (not filename)
- [x] Return first 8 chars of hash per segment for readability

### Task 3: Implement `anonymize_path()` method (AC: 3, 4, 6)
- [x] Split path into segments using `std::path::Path::components()`
- [x] Hash each segment (including filename stem)
- [x] Append original extension to hashed filename
- [x] Reconstruct path with hashed segments
- [x] Handle edge cases: empty path, no extension, root paths, Windows paths

### Task 4: Add consistency mechanism (AC: 5)
- [x] Use deterministic salt (created once per `PathAnonymizer` instance)
- [x] Same salt + same path = same hash (no need for HashMap cache)
- [x] Document that new instance = new salt = different hashes

### Task 5: Write unit tests (AC: 7)
- [x] Test extension preservation: `image.jpg` → `a1b2c3d4.jpg`
- [x] Test directory hashing: `/home/user/` → `hash1/hash2/`
- [x] Test full path: `/home/user/photo.png` → `h1/h2/h3.png`
- [x] Test consistency: same input = same output with same instance
- [x] Test different instances produce different hashes
- [x] Test edge cases: no extension, hidden files (`.bashrc`), empty path

### Task 6: Run validation
- [x] `cargo fmt --all`
- [x] `cargo clippy --all --all-targets -- -D warnings`
- [x] `cargo test`

### Task 7: Commit changes
- [x] Stage all changes
- [x] Commit: `feat(diagnostics): add path anonymization [Story 2.1]`

---

## Dev Notes

### Source Tree

```
src/diagnostics/
├── mod.rs              # MODIFY: export PathAnonymizer
├── anonymizer.rs       # NEW: PathAnonymizer implementation
├── sanitizer.rs        # EXISTING: message sanitization (separate concern)
├── report.rs           # EXISTING: will use PathAnonymizer in Story 2.4
└── ...
```

### Technical Context

- **blake3** already in `Cargo.toml` (`blake3 = "1.5"`)
- 8 char hash prefix is sufficient for diagnostics (collision unlikely)
- Session salt prevents cross-session correlation (privacy enhancement)

### Anonymization Format

**Input:** `/home/user/photos/vacation/beach.jpg`
**Output:** `a1b2c3d4/e5f6g7h8/i9j0k1l2/m3n4o5p6/q7r8s9t0.jpg`

Each segment is hashed:
- `/home` → `a1b2c3d4`
- `/user` → `e5f6g7h8`
- `/photos` → `i9j0k1l2`
- `/vacation` → `m3n4o5p6`
- `beach` → `q7r8s9t0`
- `.jpg` → preserved

### Relationship with Existing Modules

| Module | Relationship |
|--------|--------------|
| `sanitizer.rs` | Different concern: removes paths from messages. `anonymizer.rs` hashes paths for structured fields. |
| `report.rs` | Will call `PathAnonymizer::anonymize_path()` during export (Story 2.4) |
| `events.rs` | Events may contain paths that need anonymization before export |

### Public API

```rust
pub struct PathAnonymizer {
    salt: [u8; 32],
}

impl PathAnonymizer {
    pub fn new() -> Self;                           // Random salt
    pub fn with_seed(seed: u64) -> Self;            // Deterministic (for tests)
    pub fn anonymize_path(&self, path: &Path) -> PathBuf;
}
```

---

## Testing

### Unit Tests

| Test | Verification |
|------|--------------|
| `extension_preserved` | `photo.jpg` → `*.jpg` |
| `directory_segments_hashed` | `/a/b/c` → 3 hash segments |
| `full_path_anonymized` | Complete path transformation |
| `consistency_same_instance` | Same path, same hash |
| `different_instances_differ` | New instance, different hash |
| `edge_case_no_extension` | `Makefile` → `hash` (no extension) |
| `edge_case_hidden_file` | `.bashrc` → `hash` (dot is part of name) |
| `edge_case_empty_path` | Empty → Empty or error |
| `edge_case_windows_path` | `C:\Users\...` handled correctly |

### Integration Tests

- Deferred to Story 2.4 (export functionality)

---

## Dev Agent Record

### Agent Model Used
Claude Opus 4.5 (claude-opus-4-5-20251101)

### Completion Notes
- Implemented `PathAnonymizer` struct with blake3 keyed hashing
- Salt generated from system time + process ID (no external RNG crate needed)
- `with_seed(u64)` constructor for deterministic testing
- 18 unit tests covering all acceptance criteria
- All tests pass, clippy clean, formatted

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2026-01-13 | Story created | PO |
| 2026-01-13 | PO Validation: Fixed AC/example contradiction, added Task-AC mapping, source tree, integration notes | Sarah (PO) |
| 2026-01-13 | Implementation complete | James (Dev) |

### File List
| File | Action | Description |
|------|--------|-------------|
| `src/diagnostics/anonymizer.rs` | Created | PathAnonymizer implementation with 18 tests |
| `src/diagnostics/mod.rs` | Modified | Export PathAnonymizer |

---
