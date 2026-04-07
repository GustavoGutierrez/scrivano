# Design: Scrivano MVP Implementation

## Technical Approach

Build MVP using layered architecture with clear interface separation:
- `audio/` - capture layer (system + mic)
- `transcription/` - Whisper engine + Ollama fallback
- `summarization/` - Ollama client with thinking parser
- `storage/` - SQLite repositories
- `export/` - format writers
- `ui/` - egui screens

## Architecture Decisions

### Decision: Embedded Whisper as Primary STT

**Choice**: Use whisper-rs (Rust bindings) for transcription  
**Alternatives**: Ollama STT, cloud APIs  
**Rationale**: MVP requires offline-first operation. Whisper embedded gives full control over pipeline, progress reporting, and error recovery. Ollama remains as fallback for advanced scenarios.

### Decision: SQLite Schema Extension

**Choice**: Extend existing `recordings` table with new columns; add `transcript_segments`, `highlights`, `summaries`, `user_settings` tables  
**Alternatives**: Separate database per domain, noSQL  
**Rationale**: Simplifies MVP by reusing existing rusqlite integration. Schema versioning through migrations.

### Decision: Thinking Model Parser via Model Name Detection

**Choice**: Detect thinking models by known prefixes (deepseek-r1, qwen3, etc.) + extract content from `<think>` blocks  
**Alternatives**: Always show thinking, always hide  
**Rationale**: User requirement is to hide thinking in UI. Detection by name is reliable for v1; can add payload inspection later if needed.

### Decision: Streaming Mode Auto-Detection

**Choice**: Default to `auto` mode; use streaming when model supports it and latency heuristic suggests benefit  
**Alternatives**: Force streaming, force non-streaming  
**Rationale**: Improves UX on good connections while maintaining reliability on marginal connections.

## Data Flow

```
┌─────────────┐     ┌───────────────┐     ┌──────────────┐
│ Audio Capture│ ──→ │ Save to Disk  │ ──→ │ SQLite Meta  │
└─────────────┘     └───────────────┘     └──────────────┘
       │                                           │
       ▼                                           ▼
┌─────────────┐                            ┌──────────────┐
│ Whisper     │                            │ UI Display   │
│ Transcription                                    │ History      │
└─────────────┘                            └──────────────┘
       │                                           │
       ▼                                           ▼
┌─────────────┐                            ┌──────────────┐
│ Ollama      │                            │ Export       │
│ Summary    │                            │ TXT/MD/SRT   │
└─────────────┘                            └──────────────┘
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src/audio.rs` | Modify | Add source selection, VU meter, highlight support |
| `src/audio_devices.rs` | Modify | Extend with device enumeration |
| `src/database.rs` | Modify | Add tables for segments/highlights/summaries/settings |
| `src/transcription.rs` | Modify | Support segment timestamps, language selection |
| `src/ollama.rs` | Modify | Add STT fallback, streaming support, thinking parser |
| `src/export.rs` | Create | New module for TXT/MD/JSON/SRT/VTT export |
| `src/summarization.rs` | Create | New module for summary generation logic |
| `src/ui.rs` | Modify | Add history, detail, settings views |
| `src/lib.rs` | Modify | Export new modules |

## Interfaces / Contracts

```rust
// Transcription Engine trait
pub trait TranscriptionEngine {
    fn transcribe(&self, audio_path: &Path) -> Result<Vec<TranscriptSegment>>;
}

// Summary Engine trait  
pub trait SummaryEngine {
    fn generate(&self, transcript: &str, template: SummaryTemplate) -> Result<Summary>;
    fn extract_thinking(&self, response: &str) -> (&str, Option<&str>);
}

// Storage Repository trait
pub trait StorageRepo {
    fn save_recording(&self, recording: Recording) -> Result<i64>;
    fn get_recording(&self, id: i64) -> Result<Recording>;
    fn list_recordings(&self, filter: RecordingFilter) -> Result<Vec<Recording>>;
}
```

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | Whisper segments, Ollama parsing, export formatters | Mock audio input, verify output format |
| Integration | SQLite CRUD, file system operations | Use temp DB and temp dirs |
| E2E | Full recording → transcript → export flow | Manual testing for MVP |

## Migration / Rollback

No migration required for MVP v1 - fresh database schema. Schema stored in code via `init()` functions.

Rollback: Git revert of modified source files. SQLite database uses `IF NOT EXISTS` for forward compatibility.

## Open Questions

- [ ] Linux audio backend: PulseAudio vs PipeWire priority? Will validate during implementation
- [ ] macOS system audio capture: need research on BlackHole/Loopback integration
- [ ] Default summary model for 4-8GB VRAM: test qwen2.5:3b vs llama3.2:3b during implementation