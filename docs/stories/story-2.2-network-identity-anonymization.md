# Story 2.2: Network and Identity Anonymization

**Epic:** 2 - Anonymization & Export System
**Status:** Approved
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
5. `anonymize_string()` method applies all anonymizations to any input string
6. Original values cannot be recovered from hashes
7. Unit tests with various input patterns verify detection and anonymization

**Note:** This story implements the anonymization tools. Integration with `events.rs` fields will be done in Story 2.4 (export).

---

## Tasks

### Task 1: Add `IdentityAnonymizer` to `anonymizer.rs` (AC: 4)
- [ ] Add struct with session salt (reuse pattern from `PathAnonymizer`)
- [ ] Add constructor `new()` and `with_seed(u64)` for deterministic tests
- [ ] Export from `mod.rs`

### Task 2: Implement IPv4 detection and hashing (AC: 1, 6)
- [ ] Use `LazyLock<Regex>` pattern (like `sanitizer.rs`)
- [ ] Pattern: `\b(?:\d{1,3}\.){3}\d{1,3}\b`
- [ ] Replace with `<ip:a1b2c3d4>` format
- [ ] Hash the full IP address

### Task 3: Implement IPv6 detection and hashing (AC: 1, 6)
- [ ] Use `LazyLock<Regex>` for compiled pattern
- [ ] Pattern provided in Dev Notes (complex, handles all forms)
- [ ] Replace with `<ip:a1b2c3d4>` format (same as IPv4)

### Task 4: Implement domain detection and hashing (AC: 2, 6)
- [ ] Pattern: `\b(?:[a-zA-Z0-9-]+\.)+[a-zA-Z]{2,}\b`
- [ ] Preserve TLD (e.g., `example.com` → `<domain:a1b2>.com`)
- [ ] Skip common non-domains: `file.txt`, `image.jpg` (check against known TLDs)

### Task 5: Implement username detection (AC: 3, 6)
- [ ] Get system username at construction time (cross-platform)
- [ ] Store as field in `IdentityAnonymizer`
- [ ] Detect in strings (case-insensitive) and replace with `<user:hash>`

### Task 6: Create `anonymize_string()` method (AC: 5)
- [ ] Apply in order: username → IP → domain (username first to avoid partial matches)
- [ ] Return fully anonymized string
- [ ] Handle empty/None inputs gracefully

### Task 7: Write unit tests (AC: 7)
- [ ] Test IPv4 detection: `192.168.1.1`, `10.0.0.1`, `255.255.255.255`
- [ ] Test IPv6 detection: `::1`, `fe80::1`, `2001:db8::1`
- [ ] Test domain detection: `example.com`, `sub.domain.org`
- [ ] Test username detection with actual system username
- [ ] Test combined: string with multiple PII types
- [ ] Test consistency: same input = same output
- [ ] Test edge cases: no matches, overlapping patterns

### Task 8: Run validation
- [ ] `cargo fmt --all`
- [ ] `cargo clippy --all --all-targets -- -D warnings`
- [ ] `cargo test`

### Task 9: Commit changes
- [ ] Stage all changes
- [ ] Commit: `feat(diagnostics): add identity anonymization [Story 2.2]`

---

## Dev Notes

### Source Tree

```
src/diagnostics/
├── mod.rs              # MODIFY: export IdentityAnonymizer
├── anonymizer.rs       # MODIFY: add IdentityAnonymizer (alongside PathAnonymizer)
├── sanitizer.rs        # EXISTING: reference for LazyLock<Regex> pattern
└── ...
```

### Technical Context

- **LazyLock pattern**: Use `std::sync::LazyLock` (not lazy_static/once_cell) - see `sanitizer.rs` for example
- **blake3**: Already in project, reuse for hashing
- **regex**: Already in `Cargo.toml`

### IPv6 Regex Pattern

```rust
// Simplified pattern that catches most IPv6 addresses
// Full RFC 5952 compliance is overkill for diagnostic anonymization
static IPV6_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)\b(",
        r"(?:[0-9a-f]{1,4}:){7}[0-9a-f]{1,4}|",           // Full form
        r"(?:[0-9a-f]{1,4}:){1,7}:|",                      // Trailing ::
        r"(?:[0-9a-f]{1,4}:){1,6}:[0-9a-f]{1,4}|",        // :: in middle
        r"(?:[0-9a-f]{1,4}:){1,5}(?::[0-9a-f]{1,4}){1,2}|",
        r"(?:[0-9a-f]{1,4}:){1,4}(?::[0-9a-f]{1,4}){1,3}|",
        r"(?:[0-9a-f]{1,4}:){1,3}(?::[0-9a-f]{1,4}){1,4}|",
        r"(?:[0-9a-f]{1,4}:){1,2}(?::[0-9a-f]{1,4}){1,5}|",
        r"[0-9a-f]{1,4}:(?::[0-9a-f]{1,4}){1,6}|",
        r":(?::[0-9a-f]{1,4}){1,7}|",                      // Leading ::
        r"::(?:[fF]{4}:)?(?:\d{1,3}\.){3}\d{1,3}|",       // IPv4-mapped
        r"::1|::",                                          // Loopback, unspecified
        r")\b"
    )).expect("IPv6 regex should compile")
});
```

### Cross-Platform Username Detection

```rust
fn get_system_username() -> Option<String> {
    // Try Unix-style first, then Windows
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .ok()
}
```

### Anonymization Output Format

| Input | Output |
|-------|--------|
| `192.168.1.1` | `<ip:a1b2c3d4>` |
| `::1` | `<ip:e5f6g7h8>` |
| `example.com` | `<domain:i9j0k1l2>.com` |
| `john` (if username) | `<user:m3n4o5p6>` |
| `Error at /home/john/file.txt from 192.168.1.1` | `Error at <path> from <ip:a1b2c3d4>` |

**Note:** Path anonymization uses `PathAnonymizer` (Story 2.1), not this story.

### Relationship with PathAnonymizer

| Aspect | Decision |
|--------|----------|
| Same file | Yes, both in `anonymizer.rs` |
| Shared salt | No, each has own salt (different lifetime) |
| Shared trait | Optional, not required |
| Combined struct | Future consideration for Story 2.4 |

### Known TLDs (for domain detection)

Common TLDs to recognize: `com`, `org`, `net`, `edu`, `gov`, `io`, `dev`, `app`, country codes (2 letters).

Skip file extensions that look like TLDs: check if preceded by common filename patterns.

---

## Testing

### Unit Tests

| Test | Input | Expected Output |
|------|-------|-----------------|
| `ipv4_detection` | `Connect to 192.168.1.1` | `Connect to <ip:hash>` |
| `ipv4_multiple` | `From 10.0.0.1 to 10.0.0.2` | `From <ip:h1> to <ip:h2>` |
| `ipv6_loopback` | `Listening on ::1` | `Listening on <ip:hash>` |
| `ipv6_full` | `2001:db8::1` | `<ip:hash>` |
| `domain_simple` | `Fetched from example.com` | `Fetched from <domain:hash>.com` |
| `domain_subdomain` | `api.github.com` | `<domain:hash>.com` |
| `username_detected` | `User: {systemuser}` | `User: <user:hash>` |
| `combined_all` | String with IP + domain + username | All replaced |
| `no_matches` | `Hello world` | `Hello world` (unchanged) |
| `consistency` | Same input twice | Same output |

### Integration Tests

- Deferred to Story 2.4 (export with full anonymization pipeline)

---

## Dev Agent Record

### Agent Model Used
<!-- Record which AI model completed this story -->

### Completion Notes
<!-- Dev agent adds notes here during implementation -->

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2026-01-13 | Story created | PO |
| 2026-01-13 | PO Validation: Fixed LazyLock reference, added IPv6 pattern, cross-platform username, Task-AC mapping, source tree | Sarah (PO) |

### File List
<!-- Files created or modified -->

---
