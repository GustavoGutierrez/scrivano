# Proposal: Ollama Block Correction

## Summary

Replace the current single-shot `improve_transcript()` call with a block-by-block correction pipeline. Instead of sending the entire transcript (potentially 60+ min) to Ollama in one request, the corrected flow:

1. Splits the full transcript into blocks of 3–8 minutes of spoken content.
2. Sends each block independently to Ollama for correction (spelling, punctuation, grammar).
3. Merges corrected blocks back in chronological order, preserving timestamps.
4. Runs a final lightweight pass for title, tags, and summary generation.

## Motivation

The current `improve_transcript()` sends the entire meeting transcript in a single Ollama request. For long meetings (60+ min):

- **Latency**: single request can take 60–180 seconds blocking UI.
- **Memory pressure**: large prompt context fills VRAM quickly.
- **Model limits**: smaller local models handle short contexts better.
- **Failure risk**: one failure loses all improvement work.

Block correction keeps context small per request, allows incremental progress reporting, and enables retry of individual blocks.

## Scope

### In Scope

- **src/ollama_block.rs**: new module with `improve_transcript_blocks()` public API.
- Block splitting strategy: group transcript segments into ~5-minute text blocks.
- Per-block Ollama correction using a configurable model (default: first available from settings).
- Ordered merge with deduplication at block boundaries.
- Progress callback per block (0..100 per block, with block index context).
- Retry failed blocks individually.
- Graceful fallback: if Ollama is unavailable, return raw transcript unchanged.
- Title/tags generation reuses existing `generate_text()` — no change needed.
- Database: store corrected blocks in `transcript_segments` table (updated text per segment).

### Out of Scope

- Changing the chunking/transcription pipeline (`audio_chunker`, `chunk_transcription`).
- Changing summarization templates or flow.
- Auto-selecting a different Ollama model per block.
- Parallel block processing (single-threaded sequential with progress).
- Storing intermediate block results to disk separately from final transcript.
- UI redesign — only adding a new button/state for block-level progress.

## Affected Modules

| Module | Impact |
|--------|--------|
| `src/ollama_block.rs` | **NEW** — block correction engine |
| `src/ollama.rs` | No changes (used by new module) |
| `src/ui.rs` | Replace `improve_transcript()` call with block-based alternative; add block progress UI |
| `src/database.rs` | No schema changes; existing segment storage handles per-segment corrected text |

## Risks

1. **Block boundary coherence**: neighboring blocks may produce inconsistent corrected text at boundaries. Mitigated by overlap-aware splitting and boundary deduplication.
2. **Model context size**: small models have limited context windows. Mitigated by keeping blocks ≤ ~1500 chars.
3. **Latency sum**: sequential processing of N blocks adds up. Mitigated by: 5-minute blocks → ~12 blocks for 60 min meeting → ~2-3s per block with qwen2.5:3b → ~30s total, which is faster than a single 60-min request.
4. **No regression**: existing `improve_transcript()` path MUST remain functional for recordings without chunked pipeline support.

## Rollback Plan

- The new `improve_transcript_blocks()` is additive. If it fails or produces worse results, the existing single-shot `improve_transcript()` can be restored with a one-line change in `ui.rs`.
- No database migrations — segment text updates use existing schema.

## Decisions Needed

1. **Block size**: 5 minutes vs 3 minutes vs 8 minutes? Recommendation: 5 minutes (balanced).
2. **Model selection**: use summary model from settings vs dedicated correction model? Recommendation: reuse `ollama_model` from settings (user's chosen model).
3. **Override threshold**: should the user be able to force single-shot mode? Recommendation: auto-detect by transcript length (> 2000 chars uses blocks, ≤ 2000 chars uses single-shot).
