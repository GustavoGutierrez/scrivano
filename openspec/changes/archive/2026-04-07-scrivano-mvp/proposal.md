# Proposal: Scrivano MVP Implementation

## Intent

Build Scrivano v1, a Rust desktop application for recording system audio and transcribing meetings locally using Whisper, with optional summarization via Ollama. Solves the problem of capturing meeting content for later review without cloud dependencies.

## Scope

### In Scope
- Audio capture (system loopback + optional mic) with start/stop control
- Local transcription via embedded Whisper (whisper-rs)
- Optional STT via Ollama as fallback
- Summary generation via Ollama (executive, tasks, decisions templates)
- Thinking model support with hidden reasoning in UI
- SQLite persistence for recordings, transcripts, summaries, highlights
- Export to TXT/Markdown/JSON/SRT/WebVTT
- egui/eframe UI with recording state display
- Basic highlights with timestamps
- Settings for audio devices, Whisper path, Ollama host, language

### Out of Scope
- Calendar integration
- DOCX export
- Cloud sync
- Encryption at rest
- Mobile apps
- Platform-specific features beyond MVP

## Approach

Implement layered architecture:
- `audio/` - capture layer (PulseAudio/PipeWire abstraction)
- `transcription/` - Whisper engine interface + Ollama STT fallback
- `summarization/` - Ollama client with thinking parser
- `storage/` - SQLite repositories + filesystem
- `export/` - format writers
- `ui/` - egui screens

Use interfaces for testability: `TranscriptionEngine`, `SummaryEngine`, `StorageRepo`.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/audio.rs` | Modified | Core audio capture logic |
| `src/transcription.rs` | Modified | Whisper integration |
| `src/ollama.rs` | Modified | Ollama client for STT/summaries |
| `src/database.rs` | Modified | SQLite schema and queries |
| `src/ui.rs` | Modified | egui UI screens |
| `src/export.rs` | New | Export formatters |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Audio capture backend instability on Linux | Medium | Abstract capture layer, validate early |
| Whisper performance on limited hardware | Medium | Allow CPU/GPU toggle, batch processing |
| Thinking model payload variability | Medium | Parser by model name + structure detection |
| Long sessions cause memory issues | Medium | Incremental writes, checkpoints |

## Rollback Plan

- Keep original `src/audio.rs`, `src/transcription.rs`, `src/database.rs` in git before modification
- Feature flags for Ollama integration (disabled by default until validated)
- SQLite migrations with backward compatibility

## Dependencies

- whisper-rs / whisper.cpp bindings
- Ollama API (localhost:11434 default)
- libpulse-dev or PipeWire headers for Linux
- rusqlite with bundled SQLite

## Success Criteria

- [ ] Recording starts in <1s from hotkey/click
- [ ] 2-hour session completes without crash
- [ ] Transcription produces ES/EN output
- [ ] Summary generates in executive/tasks/decisions formats
- [ ] Export produces valid TXT/MD/SRT files
- [ ] Settings persist across restarts
- [ ] UI shows recording state, VU meter, elapsed time
- [ ] App works offline when Whisper model loaded