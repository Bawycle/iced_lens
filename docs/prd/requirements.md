# Requirements

## Functional Requirements

- **FR1:** The system shall continuously collect system resource metrics (CPU usage, RAM usage, disk I/O) in lightweight mode with configurable sampling interval
- **FR2:** The system shall log user actions (navigation, media loading, video controls, settings changes) with timestamps
- **FR3:** The system shall capture application state changes and internal operations (media loading states, video player states, editor states)
- **FR4:** The system shall capture warnings and errors from notifications and console output
- **FR5:** The system shall store collected data in a circular buffer with a default capacity of 10 minutes of history
- **FR6:** The system shall anonymize file paths by hashing while preserving file extensions
- **FR7:** The system shall anonymize IP addresses, domain names, and usernames in collected data
- **FR8:** The system shall export collected data as structured JSON with a well-defined schema
- **FR9:** The system shall provide file export functionality to save JSON reports to disk
- **FR10:** The system shall provide clipboard export functionality to copy JSON reports
- **FR11:** The system shall provide a dedicated Diagnostics screen accessible from the hamburger menu
- **FR12:** The Diagnostics screen shall display current collection status (enabled/disabled, lightweight mode active)
- **FR13:** The Diagnostics screen shall provide a toggle to enable/disable data collection
- **FR14:** The Diagnostics screen shall provide buttons to export reports (file and clipboard)
- **FR15:** The system shall preserve hardware configuration data (OS, CPU model, RAM size, disk type) without anonymization for diagnostic value

## Non-Functional Requirements

- **NFR1:** Collection overhead shall not exceed 1% CPU usage and 1% additional RAM consumption during lightweight mode
- **NFR2:** The circular buffer shall use no more than 10MB of memory by default
- **NFR3:** Export operation shall complete in under 2 seconds for a full buffer
- **NFR4:** All anonymization shall be one-way (non-reversible) using cryptographic hashing
- **NFR5:** The system shall work consistently across Linux, Windows, and macOS platforms
- **NFR6:** The Diagnostics screen shall follow existing IcedLens UI patterns and design tokens
- **NFR7:** Collection shall run on a separate thread to avoid blocking the UI
- **NFR8:** JSON output shall be valid, parseable, and optimized for LLM consumption
- **NFR9:** No data shall be transmitted over the network; all exports are local and user-initiated
- **NFR10:** The system shall gracefully degrade if system metrics are unavailable on a platform

---
