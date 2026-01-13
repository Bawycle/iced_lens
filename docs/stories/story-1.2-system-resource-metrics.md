# Story 1.2: System Resource Metrics Collection

**Epic:** 1 - Diagnostics Core & Data Collection
**Status:** Draft
**Priority:** High
**Estimate:** 3-4 hours
**Depends On:** Story 1.1

---

## Story

**As a** developer,
**I want** to collect system resource metrics (CPU, RAM, disk I/O) at regular intervals,
**So that** I can correlate performance issues with resource usage.

---

## Acceptance Criteria

1. `ResourceCollector` struct implemented using `sysinfo` crate (or similar)
2. Collects: CPU usage %, RAM usage (used/total), disk read/write bytes
3. Sampling runs on a separate thread to avoid blocking UI
4. Configurable sampling interval (default: 1 second)
5. Each sample stored as `DiagnosticEvent::ResourceSnapshot` with timestamp
6. Collector can be started/stopped via channel commands
7. Cross-platform compatibility verified (Linux, Windows, macOS)
8. Unit tests verify metric collection and thread safety

---

## Tasks

- [ ] **Task 1:** Add `sysinfo` dependency
  - [ ] Add `sysinfo = "0.32"` to Cargo.toml
  - [ ] Verify it compiles on target platform

- [ ] **Task 2:** Create `src/diagnostics/resource_collector.rs`
  - [ ] Define `ResourceMetrics` struct (cpu_percent, ram_used, ram_total, disk_read, disk_write)
  - [ ] Implement `SamplingInterval` newtype
  - [ ] Add defaults to config

- [ ] **Task 3:** Expand `DiagnosticEvent::ResourceSnapshot`
  - [ ] Update events.rs with full ResourceMetrics data
  - [ ] Add timestamp to snapshot

- [ ] **Task 4:** Implement `ResourceCollector`
  - [ ] Use `sysinfo::System` for metrics
  - [ ] Spawn sampling thread with configurable interval
  - [ ] Use `crossbeam-channel` for start/stop commands
  - [ ] Send snapshots to main collector via channel

- [ ] **Task 5:** Add `crossbeam-channel` dependency
  - [ ] Add `crossbeam-channel = "0.5"` to Cargo.toml

- [ ] **Task 6:** Write unit tests
  - [ ] Test ResourceMetrics creation
  - [ ] Test sampling interval newtype
  - [ ] Test collector start/stop

- [ ] **Task 7:** Run validation
  - [ ] `cargo fmt --all`
  - [ ] `cargo clippy --all --all-targets -- -D warnings`
  - [ ] `cargo test`

- [ ] **Task 8:** Commit changes
  - [ ] Stage all changes
  - [ ] Commit with descriptive message following conventional commits
  - [ ] Reference story number in commit message

---

## Dev Notes

- `sysinfo` crate handles cross-platform differences
- Sampling thread should be lightweight
- Channel bounded to prevent memory issues
- CPU usage may need initial delay for accurate reading

---

## Testing

### Unit Tests
- `resource_collector.rs`: metrics creation, interval newtype, start/stop

### Integration Tests
- Verify metrics are non-zero on real system

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
