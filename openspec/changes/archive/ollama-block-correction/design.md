# Design: Ollama Block Correction

## Architecture Decision Records

### ADR-001: Block splitting is time-based, not character-based

**Decision**: Group transcript segments into blocks by approximate spoken duration (e.g., 5 minutes), not by character count alone.

**Rationale**: 
- Segments come with `start_sec`/`end_sec` from Whisper timestamps.
- A 5-minute dense conversation block may be 2000 chars; a 5-minute sparse block may be 300 chars.
- Time-based splitting preserves natural conversation boundaries.
- Character count is a secondary cap to respect model context limits.

**Alternatives considered**:
- Pure character count: simpler but breaks conversation flow mid-topic.
- Fixed segment count: unpredictable block sizes.

### ADR-002: Sequential processing, not parallel

**Decision**: Process blocks one at a time, sequentially.

**Rationale**:
- Ollama instances on consumer hardware handle one request at a time efficiently.
- Parallel requests to the same Ollama instance queue internally anyway.
- Sequential processing keeps VRAM pressure low (one context at a time).
- Progress reporting is simpler and more accurate.

**Alternatives considered**:
- Parallel with 2 workers: adds complexity for marginal gain on consumer hardware.
- Async/tokio: overengineered for this use case; `std::thread::spawn` suffices.

### ADR-003: Block boundary overlap is handled at text level, not time level

**Decision**: The system prompt for each block includes the last 1-2 segments from the previous block as context (not included in final output). This provides continuity without duplicating content.

**Rationale**:
- Whisper chunking already handles time-based overlap with deduplication.
- At the Ollama correction layer, the concern is linguistic continuity, not audio overlap.
- Context-only segments are stripped before merging.
- Model sees "here's what came before" and corrects naturally.

**Alternatives considered**:
- No context: blocks may start with abrupt sentence fragments.
- Full overlap: duplicates text in output; requires post-merge deduplication.

### ADR-004: Fallback is transparent, not config-driven

**Decision**: The block pipeline auto-detects transcript length and falls back to single-shot for short transcripts. Users do not configure this behavior.

**Rationale**:
- Eliminates a settings toggle that users don't understand.
- 2000 chars ≈ ~3 minutes of spoken text. Single-shot is always faster for short content.
- Power users who want single-shot for long transcripts can use the existing `improve_transcript()` directly (no regression).

---

## Sequence Diagram: Block Correction Pipeline

```
┌──────┐     ┌──────────┐     ┌───────────┐     ┌────────┐     ┌──────────────┐
│  UI  │     │ollama_   │     │ ollama.rs │     │ Ollama │     │transcript_   │
│Thread│     │block.rs  │     │ (existing)│     │ Server │     │segments (DB) │
└──┬───┘     └────┬─────┘     └─────┬─────┘     └───┬────┘     └──────┬───────┘
   │              │                 │               │                │
   │ start_block  │                 │               │                │
   │ _correction()│                 │               │                │
   ├─────────────>│                 │               │                │
   │              │                 │               │                │
   │              │ load segments   │               │                │
   │              ├─────────────────┼───────────────┼───────────────>│
   │              │<────────────────┼───────────────┼────────────────│
   │              │                 │               │                │
   │              │ split_into_     │               │                │
   │              │ blocks()        │               │                │
   │              │ (N blocks)      │               │                │
   │              │                 │               │                │
   │              │                 │               │                │
   │ progress(0,N,│                │               │                │
   │   0)         │                │               │                │
   │<─────────────│                 │               │                │
   │              │                 │               │                │
   │              │ ── FOR each block ──            │                │
   │              │                 │               │                │
   │              │ build prompt    │               │                │
   │              │ (system + text) │               │                │
   │              │                 │               │                │
   │              │ generate_non_   │               │                │
   │              │ streaming()     │               │                │
   │              ├────────────────>│               │                │
   │              │                 │ POST /api/    │                │
   │              │                 │ generate      │                │
   │              │                 ├──────────────>│                │
   │              │                 │<──────────────│                │
   │              │<────────────────│               │                │
   │              │                 │               │                │
   │              │ store BlockResult               │                │
   │              │  (corrected or │               │                │
   │              │   original on  │               │                │
   │              │   failure)     │               │                │
   │              │                 │               │                │
   │ progress(i,N,│                │               │                │
   │   100)       │                │               │                │
   │<─────────────│                 │               │                │
   │              │                 │               │                │
   │              │ ── END FOR ──   │               │                │
   │              │                 │               │                │
   │              │ merge_corrected │               │                │
   │              │ _blocks()       │               │                │
   │              │                 │               │                │
   │              │ update segments │               │                │
   │              │ in DB           │               │                │
   │              ├─────────────────┼───────────────┼───────────────>│
   │              │<────────────────┼───────────────┼────────────────│
   │              │                 │               │                │
   │ result:      │                 │               │                │
   │ final_text   │                 │               │                │
   │<─────────────│                 │               │                │
   │              │                 │               │                │
```

---

## Module Structure

```
src/
├── ollama_block.rs    ← NEW: block correction pipeline
├── ollama.rs          ← UNCHANGED: single-shot improve_transcript, generate_text
├── transcription.rs   ← UNCHANGED: TranscriptSegment type referenced
├── database.rs        ← UNCHANGED: insert_segment used to persist corrected text
└── ui.rs              ← MODIFIED: replace improve_transcript call with block pipeline
```

---

## Data Flow

```
Raw Transcript (String)
    │
    ├── split_into_blocks(transcript, segments, block_minutes)
    │       │
    │       ▼
    │   Vec<String>  (N text blocks)
    │       │
    │       ├── For each block:
    │       │       build_system_prompt_block(previous_context)
    │       │       ollama::generate_non_streaming(prompt, model)
    │       │       → BlockResult { original, corrected, error }
    │       │
    │       ▼
    │   Vec<BlockResult>
    │       │
    │       ▼
    └── merge_corrected_blocks(original_blocks, results)
            │
            ▼
        Final corrected transcript (String)
```

---

## System Prompt for Block Correction

```
Eres un corrector de transcripciones de audio. Corrige SOLO el siguiente fragmento.
Reglas:
1. Corrige errores ortográficos y gramaticales.
2. Agrega puntuación donde falte.
3. NO agregues información que no está en el texto.
4. NO modifiques ni agregues timestamps.
5. NO escribas prefijos, comentarios ni explicaciones.
6. Devuelve SOLO el texto corregido.

[Contexto previo (solo referencia, NO incluir en salida):]
{previous_context}

[Fragmento a corregir:]
{block_text}

Texto corregido:
```

---

## Integration Point in ui.rs

**Current code** (simplified, in `stop_and_transcribe`):

```rust
let improved = ollama::improve_transcript(&model, &raw_text, progress_cb, None)?;
```

**New code**:

```rust
use crate::ollama_block;

let improved = ollama_block::improve_transcript_blocks(
    &model,
    &raw_text,
    &segments,      // TranscriptSegment list for time-based splitting
    5,              // 5-minute blocks
    move |block_idx, total_blocks, block_pct| {
        // Update UI progress showing: "Block 3/12 — 78%"
        ollama_progress.store(block_pct, Ordering::SeqCst);
    },
    None,           // no custom prompt
)?;
```

---

## Performance Budget

| Metric | Target |
|--------|--------|
| Per-block Ollama request | < 5 seconds (with qwen2.5:3b) |
| 60-min meeting (12 blocks × ~5s) | ~60 seconds total |
| Memory overhead | < 50 MB additional (transient block buffers) |
| UI frame rate during correction | ≥ 30 fps (non-blocking thread) |
| Block splitting latency | < 10 ms (in-memory string ops) |
| Block merging latency | < 5 ms |
