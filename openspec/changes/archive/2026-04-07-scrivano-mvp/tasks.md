# Tasks: Scrivano MVP Implementation

## Phase 1: Database & Storage Foundation

- [x] 1.1 Extend `src/database.rs` - Add `transcript_segments` table with recording_id, start_sec, end_sec, text
- [x] 1.2 Add `highlights` table to database - id, recording_id, timestamp_sec, label
- [x] 1.3 Add `summaries` table - id, recording_id, template, content, model_name, is_thinking_model
- [x] 1.4 Add `user_settings` table - language_default, hotkey_start_stop, hotkey_highlight, audio_input_device, whisper_model_path, ollama_host, ollama_port
- [x] 1.5 Add database methods: insert_segment(), get_segments_by_recording(), insert_highlight(), get_highlights_by_recording(), insert_summary(), get_summaries_by_recording(), save_settings(), load_settings()

## Phase 2: Audio Capture Implementation

- [x] 2.1 Modify `src/audio.rs` - Add source selection enum (System, Microphone)
- [x] 2.2 Implement device enumeration in `src/audio_devices.rs` - list_input_devices()
- [x] 2.3 Add VU meter / audio level display - sample analysis for visualization
- [ ] 2.4 Add highlight timestamp recording during recording
- [ ] 2.5 Add global hotkey support for start/stop and highlight

## Phase 3: Transcription Engine

- [x] 3.1 Modify `src/transcription.rs` - Add segment timestamp extraction from Whisper output
- [x] 3.2 Add language selection parameter (ES/EN)
- [ ] 3.3 Create segment storage flow - save to database after transcription
- [x] 3.4 Add progress callback for UI display

## Phase 4: Ollama Integration

- [x] 4.1 Modify `src/ollama.rs` - Add Ollama STT fallback capability
- [x] 4.2 Add thinking model detection - check model name prefixes (deepseek-r1, qwen3)
- [x] 4.3 Implement content extraction from thinking blocks (</think> tags)
- [x] 4.4 Add streaming mode support with auto-detection
- [x] 4.5 Create summarization module `src/summarization.rs` - generate summary with template selection

## Phase 5: Export Functionality

- [x] 5.1 Create `src/export.rs` - Implement TXT export (concatenate segments)
- [x] 5.2 Implement Markdown export with timestamp headers
- [x] 5.3 Implement JSON export with full metadata, segments, highlights, summaries
- [x] 5.4 Implement SRT export (index, timestamps HH:MM:SS,mmm --> HH:MM:SS,mmm, text)
- [x] 5.5 Implement WebVTT export

## Phase 6: UI Implementation

- [ ] 6.1 Modify `src/ui.rs` - Add main recording view with record/stop button, elapsed time, VU meter
- [ ] 6.2 Add history view - list recordings with date/duration/title, filter by date
- [ ] 6.3 Add detail/review view - audio player, transcript with timestamps, highlights list, summaries tabs
- [ ] 6.4 Add settings view - audio device dropdowns, language selector, Whisper path, Ollama host:port, hotkey config
- [ ] 6.5 Add error messaging - Whisper errors, Ollama connection errors, disk space warnings

## Phase 7: Integration & Wiring

- [ ] 7.1 Wire recording flow - start capture → stop → save file → save metadata → trigger transcription
- [ ] 7.2 Wire summary flow - transcription complete → enable summary button → generate on demand
- [ ] 7.3 Wire export flow - select recording → choose format → export to file
- [ ] 7.4 Wire settings flow - save settings → reload on app start → apply to audio/Whisper/Ollama

## Phase 8: Testing

- [ ] 8.1 Test: Recording starts and saves audio file
- [ ] 8.2 Test: Transcription produces segmented output with timestamps
- [ ] 8.3 Test: Summary generates for each template (executive, tasks, decisions)
- [ ] 8.4 Test: Export produces valid TXT/MD/JSON/SRT/VTT files
- [ ] 8.5 Test: Settings persist across app restarts
- [ ] 8.6 Test: UI displays recording state, history, detail views correctly