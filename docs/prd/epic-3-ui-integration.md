# Epic 3: UI Integration

**Goal:** Add the Diagnostics screen to IcedLens, providing users with a clean interface to view collection status, toggle collection, and export reports.

## Story 3.1: Diagnostics Screen Layout and Navigation

**As a** developer,
**I want** a Diagnostics screen accessible from the hamburger menu,
**So that** I can access diagnostic controls when needed.

**Acceptance Criteria:**
1. New `DiagnosticsScreen` component created in `src/ui/`
2. Screen added to hamburger menu alongside Settings, Help, About
3. Menu entry uses appropriate icon from action_icons
4. Navigation to/from Diagnostics screen works correctly
5. Screen follows existing IcedLens layout patterns (header, content area)
6. Back navigation returns to previous screen
7. Keyboard navigation works (Tab, Enter, Escape)

## Story 3.2: Collection Status Display

**As a** developer,
**I want** to see the current collection status on the Diagnostics screen,
**So that** I know whether diagnostics are active and collecting data.

**Acceptance Criteria:**
1. Status indicator shows: Disabled, Enabled (Collecting), or Error state
2. Visual indicator uses appropriate colors (following design tokens)
3. Status text describes current state clearly
4. Shows collection duration if active (e.g., "Collecting for 5m 32s")
5. Shows buffer fill level (e.g., "Buffer: 847 events")
6. Status updates in real-time (subscription to collector state)
7. Accessible: status is screen-reader friendly

## Story 3.3: Collection Toggle Control

**As a** developer,
**I want** to enable/disable diagnostic collection from the UI,
**So that** I can control when data is being collected.

**Acceptance Criteria:**
1. Toggle switch component for enabling/disabling collection
2. Toggle follows existing IcedLens toggle style
3. Toggling sends appropriate message to DiagnosticsCollector
4. UI reflects state change immediately
5. Toggle state persists across screen navigation (but not app restart for MVP)
6. Clear label indicates toggle purpose
7. Keyboard accessible (Space to toggle when focused)

## Story 3.4: Export Buttons and Feedback

**As a** developer,
**I want** buttons to export reports to file and clipboard,
**So that** I can easily get diagnostic data out of the application.

**Acceptance Criteria:**
1. "Export to File" button implemented with appropriate icon
2. "Copy to Clipboard" button implemented with appropriate icon
3. Buttons disabled when collection is disabled or buffer is empty
4. Clicking triggers respective export function
5. Success feedback shown via toast notification ("Report exported" / "Copied to clipboard")
6. Error feedback shown via toast notification with error description
7. Buttons follow existing IcedLens button styles
8. Buttons are keyboard accessible

## Story 3.5: Information and Help Content

**As a** developer,
**I want** brief explanatory content on the Diagnostics screen,
**So that** users understand what the tool does and what data is collected.

**Acceptance Criteria:**
1. Brief description of the Diagnostics tool purpose (2-3 sentences)
2. Summary of what data is collected (bullet list)
3. Privacy assurance statement (data is anonymized, never sent automatically)
4. Link or reference to documentation for more details (if docs exist)
5. Content is concise and doesn't clutter the interface
6. Text follows existing typography styles
7. Content is translatable (uses i18n system)

---
