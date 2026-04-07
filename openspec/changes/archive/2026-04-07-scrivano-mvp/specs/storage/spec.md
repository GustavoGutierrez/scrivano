# Delta for Storage

## Purpose

Define SQLite persistence for recordings, transcripts, summaries, highlights, and settings.

## ADDED Requirements

### Requirement: Recording Persistence

The system MUST store recording metadata in SQLite.

Recording entity MUST contain: id, created_at, updated_at, title, duration_seconds, audio_path, sample_rate, channels, language, has_transcript, has_summaries.

#### Scenario: Save recording metadata

- GIVEN user completes a recording
- WHEN recording is saved to disk
- THEN metadata is stored in recordings table

### Requirement: Transcript Segment Persistence

The system MUST store transcript segments linked to recording.

Segment entity MUST contain: id, recording_id, start_sec, end_sec, text.

#### Scenario: Save transcript segments

- GIVEN transcription completes
- WHEN segments are generated
- THEN each segment stored in transcript_segments table

### Requirement: Highlight Persistence

The system MUST store highlights linked to recording.

Highlight entity MUST contain: id, recording_id, timestamp_sec, label.

#### Scenario: Save highlight

- GIVEN user creates highlight during recording
- WHEN highlight is created
- THEN highlight stored with timestamp in highlights table

### Requirement: Summary Persistence

The system MUST store summaries linked to recording.

Summary entity MUST contain: id, recording_id, template, content, model_name, is_thinking_model.

#### Scenario: Save summary

- GIVEN summary generation completes
- WHEN summary is available
- THEN summary stored in summaries table

### Requirement: User Settings Persistence

The system MUST persist user settings across sessions.

Settings MUST include: language_default, hotkey_start_stop, hotkey_highlight, audio_input_device, whisper_model_path, ollama_host, ollama_port.

#### Scenario: Save settings

- GIVEN user changes settings
- WHEN user saves settings
- THEN settings stored in user_settings table

## MODIFIED Requirements

None - this is a new specification.

## REMOVED Requirements

None.

---

## Success Criteria

- [ ] Recording metadata persists correctly
- [ ] Transcript segments link to recording
- [ ] Highlights link to recording with timestamp
- [ ] Summaries persist with template and content
- [ ] Settings persist across app restarts