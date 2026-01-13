# Tech Stack

## Existing Technology Stack

| Category | Technology | Version | Usage in Enhancement | Notes |
|----------|------------|---------|---------------------|-------|
| Language | Rust | 1.92+ | All new code | Required |
| UI Framework | Iced | 0.14.0 | Diagnostics screen | Existing patterns |
| Serialization | serde, serde_json | 1.0 | JSON export | Already in project |
| Hashing | blake3 | 1.5 | Path anonymization | Already in project |
| Async Runtime | tokio | 1.48 | Collector thread | Already in project |
| File Dialog | rfd | 0.16 | File export location | Already in project |

## New Technology Additions

| Technology | Version | Purpose | Rationale | Integration Method |
|------------|---------|---------|-----------|-------------------|
| sysinfo | 0.32+ | System metrics (CPU, RAM, disk) | Cross-platform, well-maintained, Rust-native | Add to Cargo.toml |
| arboard | 3.4+ | Clipboard access | Cross-platform clipboard, Rust-native | Add to Cargo.toml |
| crossbeam-channel | 0.5+ | Thread communication | Fast, bounded channels for collector | Add to Cargo.toml |

---
