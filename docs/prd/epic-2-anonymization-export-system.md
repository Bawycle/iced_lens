# Epic 2: Anonymization & Export System

**Goal:** Implement comprehensive anonymization to ensure privacy compliance, and deliver polished export functionality with a well-defined JSON schema optimized for AI analysis.

## Story 2.1: Path Anonymization

**As a** developer,
**I want** file paths to be anonymized in diagnostic reports,
**So that** user privacy is protected while preserving diagnostic value.

**Acceptance Criteria:**
1. `PathAnonymizer` struct implemented with hash-based anonymization
2. File paths hashed using blake3 or sha256 (fast, secure)
3. File extensions preserved (e.g., `/home/user/photos/image.jpg` → `a1b2c3d4.jpg`)
4. Directory structure depth preserved but names hashed (e.g., `hash1/hash2/hash3.jpg`)
5. Consistent hashing within a session (same path = same hash)
6. Hash is one-way (cannot reverse to original path)
7. Unit tests verify anonymization correctness and consistency

## Story 2.2: Network and Identity Anonymization

**As a** developer,
**I want** IP addresses, domain names, and usernames anonymized,
**So that** no personally identifiable information appears in reports.

**Acceptance Criteria:**
1. IP address detection and hashing implemented (IPv4 and IPv6 patterns)
2. Domain name detection and hashing implemented
3. Username detection based on common patterns and system username
4. `IdentityAnonymizer` struct with methods for each type
5. Anonymization applied to all string fields in events
6. Original values cannot be recovered from hashes
7. Unit tests with various input patterns verify detection and anonymization

## Story 2.3: JSON Schema Definition and Implementation

**As a** developer,
**I want** a well-defined JSON schema for diagnostic reports,
**So that** reports are consistent and optimized for AI analysis.

**Acceptance Criteria:**
1. JSON schema documented (can be in code comments or separate doc)
2. Schema includes: metadata section, system_info section, events array, summary statistics
3. Metadata: report_id, generated_at, iced_lens_version, collection_duration_ms, event_count
4. System info: os, cpu_model, ram_total_mb, disk_type (without identifying info)
5. Events: array of typed events with consistent timestamp format (ISO 8601)
6. Summary: event counts by type, resource usage min/max/avg
7. All fields use consistent naming (snake_case)
8. Serde attributes configured for clean JSON output

## Story 2.4: File Export Implementation

**As a** developer,
**I want** to export diagnostic reports to a file,
**So that** I can save reports for later analysis or sharing.

**Acceptance Criteria:**
1. `export_to_file()` function implemented
2. Default filename format: `iced_lens_diagnostics_YYYYMMDD_HHMMSS.json`
3. User can choose save location via native file dialog (if available) or default to user's documents/downloads
4. Export applies full anonymization pipeline before writing
5. File is written atomically (temp file + rename) to prevent corruption
6. Success/failure feedback provided (returns Result)
7. Integration test verifies file creation and content validity

## Story 2.5: Clipboard Export Implementation

**As a** developer,
**I want** to copy diagnostic reports to the clipboard,
**So that** I can quickly paste them into Claude Code or other tools.

**Acceptance Criteria:**
1. `export_to_clipboard()` function implemented using `arboard` crate
2. Export applies full anonymization pipeline before copying
3. Clipboard contains the same JSON as file export
4. Works on Linux (X11/Wayland), Windows, and macOS
5. Graceful error handling if clipboard access fails
6. Success/failure feedback provided (returns Result)
7. Manual testing on all three platforms

---
