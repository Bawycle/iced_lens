# Project Brief: IcedLens Diagnostics Tool

**Version:** 1.0
**Date:** 2026-01-13
**Author:** Business Analyst (BMAD Method)

---

## Executive Summary

**IcedLens Diagnostics** is an integrated developer tool that automatically collects anonymized performance data to help diagnose and resolve application issues. The tool serves as a bridge between user-experienced problems and technical analysis by AI assistants (like Claude Code).

**Primary Problem:** Developers lack precise, contextual data when users report vague performance issues ("it's slow", "it freezes").

**Target Users:** IcedLens developers and contributors.

**Key Value Proposition:** Zero-friction data collection that respects privacy-first principles while providing actionable diagnostic information for AI-assisted troubleshooting.

---

## Problem Statement

### Current State & Pain Points

IcedLens users experience various performance issues:
- Slow media loading during navigation
- Video seeking stuttering
- High memory consumption
- High CPU usage
- UI freezes
- Audio/video desynchronization

**The core problem:** When these issues occur, developers have no way to understand what happened. Users can only provide vague descriptions ("sometimes it's slow when changing media"), which are insufficient for diagnosis.

### Impact of the Problem

- **Debugging time wasted:** Without data, developers must guess and reproduce issues blindly
- **User frustration:** Issues remain unresolved because they cannot be properly analyzed
- **Contributor barrier:** External contributors cannot diagnose issues without deep codebase knowledge
- **AI assistance limited:** Claude Code cannot help without precise contextual data

### Why Existing Solutions Fall Short

- **Manual profiling:** Requires user action at the moment of the issue (unrealistic)
- **System monitoring tools:** Don't capture application-specific context (states, operations, user actions)
- **Log files:** Often too verbose or not detailed enough for performance analysis

### Urgency

As IcedLens grows in features (AI deblur, upscaling, video playback), performance issues become more likely and harder to diagnose. Proactive instrumentation is essential before the codebase becomes too complex.

---

## Proposed Solution

### Core Concept

An integrated diagnostic module that:
1. **Continuously collects lightweight metrics** in the background (minimal performance impact)
2. **Automatically escalates to detailed collection** when anomalies are detected
3. **Anonymizes all sensitive data** before export (privacy-first)
4. **Exports structured JSON reports** ready for AI analysis

### Key Differentiators

| Aspect | Our Approach | Typical Tools |
|--------|--------------|---------------|
| User involvement | Zero (fully automatic) | Requires manual activation |
| Privacy | Anonymized by design | Often captures sensitive paths |
| AI-readiness | JSON structured for LLM consumption | Human-oriented formats |
| Integration | Native to IcedLens | External tools |
| Performance impact | Self-measuring overhead | Unknown impact |

### Why This Will Succeed

- **Frictionless:** Users don't change their behavior
- **Privacy-respecting:** Aligned with IcedLens's core values
- **AI-optimized:** Designed for the Claude Code workflow developers already use
- **Incremental:** MVP is small and self-contained

---

## Target Users

### Primary User Segment: IcedLens Developers

**Profile:**
- Core maintainers of IcedLens
- Familiar with Rust/Iced architecture
- Use Claude Code for development assistance

**Current Behaviors:**
- Receive vague bug reports about performance
- Manually try to reproduce issues
- Add temporary logging/profiling when debugging
- Ask users for more details (often unsuccessfully)

**Needs & Pain Points:**
- Precise data about what happened before/during an issue
- Context about user actions and system state
- Correlation between resource usage and operations
- Data format compatible with AI analysis

**Goals:**
- Diagnose issues faster
- Fix performance problems systematically
- Understand real-world usage patterns (anonymized)

### Secondary User Segment: External Contributors

**Profile:**
- Open source contributors
- May not be familiar with the full codebase
- Want to help but lack context

**Needs:**
- Understanding of execution flow
- Links to relevant documentation/architecture
- Reproducible test cases

**Goals:**
- Contribute performance fixes
- Understand the codebase through real data

---

## Goals & Success Metrics

### Business Objectives

- Reduce average time to diagnose performance issues by 50%
- Enable AI-assisted debugging for 100% of performance reports
- Maintain zero impact on user privacy (no identifiable data in reports)

### User Success Metrics

- Developers can generate a useful diagnostic report in < 30 seconds
- Reports contain sufficient data for Claude Code to identify root causes
- Contributors can understand execution flow from report data

### Key Performance Indicators (KPIs)

| KPI | Definition | Target |
|-----|------------|--------|
| Report completeness | % of reports with all MVP data categories | 100% |
| Collection overhead | CPU/RAM cost of lightweight collection | < 1% |
| Anonymization coverage | % of sensitive data properly anonymized | 100% |
| Time to first diagnosis | Time from report generation to actionable insight | < 5 min |

---

## MVP Scope

### Core Features (Must Have)

| Feature | Description | Rationale |
|---------|-------------|-----------|
| **Lightweight continuous collection** | Background capture of essential metrics | Foundation of the system |
| **Resource metrics** | CPU, RAM, disk I/O history | Core performance indicators |
| **User action logging** | Record user interactions | Understand "what happened before" |
| **App state/operation logging** | Internal states and operations | Understand "what the app was doing" |
| **Warning/error capture** | Notifications and console output | Often contains the answer |
| **Basic anonymization** | Hash paths, anonymize IPs/users | Privacy requirement |
| **JSON export** | Structured output format | AI-compatible |
| **File + clipboard export** | Two export mechanisms | Flexibility |
| **Diagnostics screen** | Status, toggle, export buttons | User interface |
| **Circular buffer** | Fixed-size rolling storage | Prevent storage explosion |

### Out of Scope for MVP

- Automatic anomaly detection and mode switching
- Real-time metrics visualization
- Session history and comparison
- Configurable retention settings (use defaults)
- Benchmark mode
- Network vs local file distinction
- Unusual character detection in paths
- Self-instrumentation (collection overhead measurement)
- Execution flow traces (detailed)

### MVP Success Criteria

The MVP is successful when:
1. A developer can enable collection, use the app normally, and export a report
2. The report contains enough data for Claude Code to provide actionable insights
3. No sensitive user data (paths, usernames, IPs) is exposed in the report
4. Collection overhead is imperceptible during normal use

---

## Post-MVP Vision

### Phase 2 Features

| Feature | Value Added |
|---------|-------------|
| **Auto-triggered detailed collection** | Capture more data when anomalies occur |
| **Self-instrumentation** | Measure collection overhead separately |
| **Local vs network distinction** | Better I/O diagnosis |
| **Configurable retention** | User control over data storage |
| **Real-time preview** | Visual feedback during collection |
| **Session history** | Compare reports over time |

### Long-term Vision

A comprehensive performance observatory that:
- Automatically detects and captures performance anomalies
- Correlates user descriptions with technical data
- Provides built-in visualization for quick diagnosis
- Includes benchmark suite for regression detection
- Generates AI-ready reports with architectural context

### Expansion Opportunities

- **Benchmark mode:** Reproducible performance tests
- **Correlation engine:** Link "it was slow" to specific time windows
- **Architecture integration:** Include relevant doc links in reports
- **Trend analysis:** Detect performance regressions across versions

---

## Technical Considerations

### Platform Requirements

| Aspect | Requirement |
|--------|-------------|
| **Target Platforms** | Linux, Windows, macOS (same as IcedLens) |
| **Performance Budget** | < 1% CPU/RAM overhead for lightweight collection |
| **Storage** | Circular buffer, configurable size (default ~10MB) |

### Technology Preferences

| Component | Preference | Rationale |
|-----------|------------|-----------|
| **Metrics collection** | `sysinfo` crate or similar | Rust-native, cross-platform |
| **Serialization** | `serde` + `serde_json` | Already in project |
| **Hashing** | `blake3` or `sha256` | Fast, secure anonymization |
| **Clipboard** | `arboard` crate | Cross-platform clipboard |
| **Buffer structure** | `VecDeque` or ring buffer | Efficient circular storage |

### Architecture Considerations

| Aspect | Approach |
|--------|----------|
| **Module location** | New `src/diagnostics/` module |
| **Integration points** | Message handlers, media loading, video player |
| **Data flow** | Events → Collector → Buffer → Anonymizer → Export |
| **UI integration** | New screen in hamburger menu |
| **Thread model** | Collection on separate thread to avoid UI blocking |

### Security/Compliance

- All data anonymized before leaving the application
- No network transmission (local export only)
- User must explicitly export (no automatic sending)
- Hash functions are one-way (cannot reverse to original data)

---

## Constraints & Assumptions

### Constraints

| Type | Constraint |
|------|------------|
| **Privacy** | Must be privacy-first; no identifiable data in exports |
| **Infrastructure** | No server/backend; everything runs locally |
| **Sharing** | Manual, voluntary sharing only |
| **Performance** | Collection must not noticeably impact app performance |
| **Complexity** | MVP must be implementable without major architecture changes |

### Key Assumptions

- Developers will use Claude Code (or similar AI) to analyze reports
- JSON is the optimal format for AI consumption
- Users are willing to enable diagnostics when debugging
- The circular buffer approach provides sufficient historical data
- Existing message/event system can be instrumented without major refactoring
- Cross-platform clipboard access is reliable enough for the use case

---

## Risks & Open Questions

### Key Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Collection overhead too high** | Defeats purpose of performance tool | Measure early, optimize aggressively |
| **Insufficient data captured** | Reports not useful for diagnosis | Iterative refinement based on real usage |
| **Anonymization incomplete** | Privacy breach | Thorough review, test with real paths |
| **Buffer size wrong** | Too small = missing data, too large = storage issues | Make configurable in v2 |
| **JSON too verbose** | Hard to paste in Claude Code | Implement compression or summary mode |

### Open Questions

1. **Sampling frequency:** What's the optimal interval for resource metrics? (100ms? 1s? event-driven?)
2. **Buffer duration:** How many minutes of history to keep by default?
3. **JSON schema:** What's the exact structure for maximum AI usefulness?
4. **Instrumentation points:** Which handlers/functions need instrumentation first?
5. **Threshold values:** What constitutes "normal" vs "anomalous" behavior?

### Areas Needing Further Research

- Optimal JSON schema for LLM consumption (test with Claude Code)
- Existing Rust crates for system metrics collection
- Cross-platform clipboard reliability
- Impact of continuous metrics collection on battery (laptops)
- Best practices for circular buffer sizing

---

## Appendices

### A. Research Summary

**Source:** Brainstorming session (2026-01-13)

**Key Findings:**
- User wants zero involvement in data collection
- Privacy is non-negotiable (app is privacy-first)
- AI-assisted diagnosis is the primary use case
- Automatic anomaly detection is desirable but complex (v2)
- Self-measuring collection overhead is important for accuracy

### B. Related Documents

- Brainstorming results: `docs/brainstorming-session-results.md`
- Architecture documentation: `docs/architecture/`
- Coding standards: `docs/architecture/coding-standards.md`

### C. References

- IcedLens CONTRIBUTING.md (patterns and conventions)
- Iced framework documentation
- `sysinfo` crate documentation
- `serde_json` serialization patterns

---

## Next Steps

### Immediate Actions

1. **Review this brief** and validate assumptions
2. **Explore codebase** to identify instrumentation points
3. **Define JSON schema** for the diagnostic report
4. **Create PRD** with detailed technical specifications
5. **Prototype collector** to validate performance impact

### PM Handoff

This Project Brief provides the full context for **IcedLens Diagnostics Tool**. Please start in 'PRD Generation Mode', review the brief thoroughly to work with the user to create the PRD section by section as the template indicates, asking for any necessary clarification or suggesting improvements.

---

*Generated using the BMAD-METHOD Project Brief Template v2.0*
