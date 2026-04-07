# Delta for Transcription

## Purpose

Define transcription behavior using embedded Whisper and optional Ollama STT fallback.

## ADDED Requirements

### Requirement: Primary Transcription via Whisper

The system MUST transcribe audio using embedded Whisper engine after recording stops.

The transcription SHALL produce text in Spanish or English based on user setting.

#### Scenario: Transcribe recording with Whisper

- GIVEN user has completed a recording
- WHEN user triggers transcription
- THEN Whisper processes audio file
- AND text segments are stored with start/end timestamps

### Requirement: Alternative STT via Ollama

The system SHOULD support Ollama as alternative transcription source when configured.

The system SHALL fall back gracefully if Ollama STT fails.

#### Scenario: Ollama STT unavailable

- GIVEN Ollama STT is enabled but Ollama is not running
- WHEN user requests transcription
- THEN system uses Whisper as primary engine
- AND notifies user of fallback

### Requirement: Segment Storage

The system SHALL store transcription segments with start/end times.

Each segment MUST contain timestamp and text.

#### Scenario: Segment storage

- GIVEN Whisper returns segments
- WHEN processing completes
- THEN each segment stored with start_sec, end_sec, text

## MODIFIED Requirements

None - this is a new specification.

## REMOVED Requirements

None.

---

## Success Criteria

- [ ] Recording transcribes with Whisper in ES/EN
- [ ] Ollama STT fallback works when enabled
- [ ] Segments stored with timestamps
- [ ] Clear error messages when Whisper fails