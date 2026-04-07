# Delta for Audio Capture

## Purpose

Define audio capture behavior for system loopback and optional microphone input.

## ADDED Requirements

### Requirement: Audio Source Selection

The system MUST support selecting audio source between system loopback and microphone.

The system SHALL list available audio devices on startup.

#### Scenario: User selects system audio

- GIVEN user has system audio available
- WHEN user selects "System Audio" as input
- THEN recording captures system loopback

#### Scenario: User selects microphone

- GIVEN user has microphone available
- WHEN user selects "Microphone" as input
- THEN recording captures microphone input

### Requirement: Recording Control

The system MUST allow starting and stopping recording via UI button.

The system SHALL support global hotkey for start/stop.

#### Scenario: Start recording from UI

- GIVEN user clicks "Start Recording"
- WHEN button is pressed
- THEN audio capture begins immediately

#### Scenario: Stop recording from UI

- GIVEN recording is in progress
- WHEN user clicks "Stop Recording"
- THEN audio capture stops and file is saved

### Requirement: Recording State Display

The system SHALL show real-time recording state including elapsed time and audio level.

#### Scenario: Recording in progress

- GIVEN recording has started
- WHEN UI renders
- THEN display elapsed time in HH:MM:SS format
- AND display audio level meter (VU)
- AND show "Recording" status indicator

### Requirement: Highlights During Recording

The system MUST allow inserting highlights during active recording.

The system SHALL record timestamp when highlight is inserted.

#### Scenario: Insert highlight during recording

- GIVEN recording is in progress
- WHEN user presses highlight hotkey or clicks highlight button
- THEN highlight marker is created with current timestamp
- AND optional label is recorded if provided

## MODIFIED Requirements

None - this is a new specification.

## REMOVED Requirements

None.

---

## Success Criteria

- [ ] User can select between system audio and microphone
- [ ] Recording starts within 1 second of user action
- [ ] Recording stops and saves file correctly
- [ ] UI shows elapsed time during recording
- [ ] UI shows audio level meter during recording
- [ ] Highlights can be inserted during recording with timestamp