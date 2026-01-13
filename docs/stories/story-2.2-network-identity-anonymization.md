# Story 2.2: Network and Identity Anonymization

**Epic:** 2 - Anonymization & Export System
**Status:** Draft
**Priority:** High
**Estimate:** 2-3 hours
**Depends On:** Story 2.1

---

## Story

**As a** developer,
**I want** IP addresses, domain names, and usernames anonymized,
**So that** no personally identifiable information appears in reports.

---

## Acceptance Criteria

1. IP address detection and hashing implemented (IPv4 and IPv6 patterns)
2. Domain name detection and hashing implemented
3. Username detection based on common patterns and system username
4. `IdentityAnonymizer` struct with methods for each type
5. Anonymization applied to all string fields in events
6. Original values cannot be recovered from hashes
7. Unit tests with various input patterns verify detection and anonymization

---

## Tasks

- [ ] **Task 1:** Add `IdentityAnonymizer` to `anonymizer.rs`
  - [ ] Struct with session salt
  - [ ] Method for each anonymization type

- [ ] **Task 2:** Implement IPv4 detection and hashing
  - [ ] Regex pattern for IPv4 (e.g., `\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}`)
  - [ ] Replace with hash (e.g., `<ip:a1b2c3d4>`)

- [ ] **Task 3:** Implement IPv6 detection and hashing
  - [ ] Regex pattern for IPv6
  - [ ] Replace with hash

- [ ] **Task 4:** Implement domain detection and hashing
  - [ ] Regex for domain patterns
  - [ ] Exclude common TLDs from hash (preserve `.com`, `.org` etc.)
  - [ ] Replace domain name with hash

- [ ] **Task 5:** Implement username detection
  - [ ] Get system username via `whoami` or env var
  - [ ] Detect in strings and replace with `<user:hash>`

- [ ] **Task 6:** Create `anonymize_string()` method
  - [ ] Apply all anonymizations to a string
  - [ ] Run path, IP, domain, username detection
  - [ ] Return fully anonymized string

- [ ] **Task 7:** Write unit tests
  - [ ] Test IPv4 detection and replacement
  - [ ] Test IPv6 detection and replacement
  - [ ] Test domain detection
  - [ ] Test username detection
  - [ ] Test combined anonymization

- [ ] **Task 8:** Run validation
  - [ ] `cargo fmt --all`
  - [ ] `cargo clippy --all --all-targets -- -D warnings`
  - [ ] `cargo test`

- [ ] **Task 9:** Commit changes
  - [ ] Stage all changes
  - [ ] Commit with descriptive message following conventional commits
  - [ ] Reference story number in commit message

---

## Dev Notes

- Use lazy_static or once_cell for compiled regexes
- System username: `std::env::var("USER")` or `whoami` crate
- Hash format: `<type:hash8chars>` for clarity in reports

---

## Testing

### Unit Tests
- IPv4, IPv6, domain, username detection
- Combined anonymization

### Integration Tests
- Verify no PII in exported JSON

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
