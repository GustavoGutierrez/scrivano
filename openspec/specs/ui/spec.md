# Delta for UI

## Purpose

Define egui/eframe UI behavior for Scrivano desktop application.

## ADDED Requirements

### Requirement: Main Recording View

The system MUST display main recording interface.

The UI SHALL show: record button, elapsed time, audio level, status indicator.

#### Scenario: Main recording screen

- GIVEN app is running
- WHEN main view is displayed
- THEN show start/stop recording button
- AND show elapsed time counter
- AND show audio level meter
- AND show recording status

### Requirement: Recording List/History View

The system MUST display list of past recordings.

The UI SHALL support filtering by date, duration, title.

#### Scenario: Recording history displayed

- GIVEN user navigates to history
- WHEN view loads
- THEN show list of recordings with date, title, duration
- AND allow filtering by date range

### Requirement: Recording Detail/Review View

The system MUST display recording detail after completion.

The UI SHALL show: audio player, transcript with timestamps, highlights list, summaries.

#### Scenario: Recording detail view

- GIVEN user selects a recording
- WHEN detail view opens
- THEN show audio playback controls
- AND show transcript aligned with timestamps
- AND show highlights list with jump-to-timestamp
- AND show summaries in tabs by template

### Requirement: Settings View

The system MUST display settings configuration.

The UI SHALL allow configuring: audio devices, language, Whisper path, Ollama host/port, hotkeys.

#### Scenario: Settings displayed

- GIVEN user navigates to settings
- WHEN view loads
- THEN show audio device dropdowns
- AND show language selector (ES/EN)
- AND show Whisper model path setting
- AND show Ollama host:port setting
- AND show hotkey configuration

### Requirement: Error and Status Messaging

The system SHALL display clear error messages for common failures.

The UI MUST show: Whisper loading errors, Ollama connection errors, disk space warnings, audio permission errors.

#### Scenario: Error displayed

- GIVEN error condition occurs
- WHEN error is detected
- THEN show clear error message
- AND suggest possible solution

## MODIFIED Requirements

None - this is a new specification.

## REMOVED Requirements

None.

---

## Success Criteria

- [ ] Main recording view shows all required elements
- [ ] History view lists recordings with filtering
- [ ] Detail view shows player, transcript, highlights, summaries
- [ ] Settings view allows configuring all options
- [ ] Error messages are clear and actionable