# Story 2.1: Path Anonymization

**Epic:** 2 - Anonymization & Export System
**Status:** Draft
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
2. File paths hashed using blake3 or sha256 (fast, secure)
3. File extensions preserved (e.g., `/home/user/photos/image.jpg` → `a1b2c3d4.jpg`)
4. Directory structure depth preserved but names hashed (e.g., `hash1/hash2/hash3.jpg`)
5. Consistent hashing within a session (same path = same hash)
6. Hash is one-way (cannot reverse to original path)
7. Unit tests verify anonymization correctness and consistency

---

## Tasks

- [ ] **Task 1:** Create `src/diagnostics/anonymizer.rs`
  - [ ] Define `PathAnonymizer` struct
  - [ ] Store session salt for consistent hashing

- [ ] **Task 2:** Implement path hashing
  - [ ] Use blake3 (already in project)
  - [ ] Hash each path segment separately
  - [ ] Preserve file extension
  - [ ] Return first 8 chars of hash per segment

- [ ] **Task 3:** Implement `anonymize_path()` method
  - [ ] Split path into segments
  - [ ] Hash each segment except extension
  - [ ] Reconstruct path with hashed segments
  - [ ] Handle edge cases (empty path, no extension, etc.)

- [ ] **Task 4:** Add consistency mechanism
  - [ ] Same path → same hash within session
  - [ ] Use internal HashMap cache or deterministic salt

- [ ] **Task 5:** Write unit tests
  - [ ] Test extension preservation
  - [ ] Test segment hashing
  - [ ] Test consistency (same input = same output)
  - [ ] Test different paths produce different hashes
  - [ ] Test edge cases

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

- blake3 already in Cargo.toml
- 8 char hash prefix is sufficient for diagnostics
- Session salt prevents cross-session correlation
- Example: `/home/user/photos/vacation/beach.jpg` → `a1b2/c3d4/e5f6/g7h8/beach.jpg` (preserve filename hash + extension)

---

## Testing

### Unit Tests
- `anonymizer.rs`: path hashing, extension preservation, consistency

### Integration Tests
- Verify anonymized paths in exported JSON

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
