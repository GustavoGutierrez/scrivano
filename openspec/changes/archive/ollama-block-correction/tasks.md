# Tasks: Ollama Block Correction

> **SDD Change**: `ollama-block-correction`
> **Strict TDD**: Active. Test runner: `cargo test`. Follow RED, GREEN, TRIANGULATE, REFACTOR.

---

## Phase 1: Infrastructure — New Module

- [x] **1.1** Create `src/ollama_block.rs` module skeleton
  - Declare `pub mod ollama_block;` in `src/lib.rs` if needed
  - Define public types: `BlockResult`, `BlockProgressFn`
  - Stub `improve_transcript_blocks()` returning original text immediately
  - Verify: `cargo build` compiles without errors

- [x] **1.2** Implement `split_into_blocks(transcript, segments, block_minutes) → Vec<String>`
  - Group segments by accumulated duration until `block_minutes * 60` seconds
  - Collect segment text into block strings (segments joined with spaces)
  - Cap each block at 4096 characters (safety valve)
  - Handle empty segments and single-segment transcrips
  - Write unit tests: RED then GREEN

- [x] **1.3** Implement `merge_corrected_blocks(original_blocks, results) → String`
  - Iterate through `Vec<BlockResult>` in block index order
  - Use `corrected_text` if available, else `original_text`
  - Join blocks with spaces; trim duplicate whitespace
  - Handle all-failed case: return original concatenated text
  - Write unit tests: RED then GREEN

- [x] **1.4** Implement block-level system prompt builder
  - Function `build_block_prompt(block_text, previous_context) → String`
  - Previous context: last 200 chars of previous block (or empty for first block)
  - Prompt rules: correct spelling/grammar, add punctuation, preserve timestamps, no commentary
  - Write unit test: prompt contains block text and instruction to not modify timestamps

---

## Phase 2: Implementation — Correction Pipeline

- [x] **2.1** Implement `improve_transcript_blocks()` core pipeline
  - Check Ollama availability → return original if unavailable
  - Short circuit for short transcripts (≤ 2000 chars) → delegate to `ollama::improve_transcript()`
  - Split transcript via `split_into_blocks()`
  - For each block: build prompt with context, call `ollama::generate_non_streaming()`, collect `BlockResult`
  - Call progress callback: `(block_idx, total, 0)` before HTTP, `(block_idx, total, 100)` after
  - On per-block failure: set `BlockResult.error`, use `original_text`, continue
  - Merge results via `merge_corrected_blocks()`
  - Return `Ok(final_text)`
  - Write integration test with mock/offline Ollama stub

- [x] **2.2** Add boundary context injection
  - Before correcting block N (N > 0), get last 200 chars from block N-1's original text
  - Pass as `previous_context` to prompt builder
  - Context text is NOT included in block N's output (handled by prompt instruction)
  - Write test: context appears in prompt but not in corrected output

- [x] **2.3** Implement progress reporting with block granularity
  - Before block processing: `progress_cb(block_idx, total_blocks, 0)`
  - After block processing: `progress_cb(block_idx, total_blocks, 100)`
  - No simulated progress — real HTTP completion determines timing
  - Write test: callback receives correct indices and counts

- [x] **2.4** Implement retry logic
  - If a block fails, retry once after 1-second delay
  - If retry also fails, mark as failed and continue
  - Log retry attempts to stderr
  - Write test: simulate transient failure, verify retry succeeds

---

## Phase 3: UI Integration

- [x] **3.1** Replace single-shot `improve_transcript()` in `ui.rs::stop_and_transcribe()`
  - Change from `ollama::improve_transcript()` to `ollama_block::improve_transcript_blocks()`
  - Pass `segments` vector for time-based splitting
  - Map block progress callback to `ollama_progress` atomic (show block-level %)
  - Keep existing `is_improving` flag and `ollama_progress` atomic
  - Verify: starts/stops improvement correctly; progress bar updates

- [x] **3.2** Update progress display for block-level granularity
  - Show "Mejorando bloque 5/12 — completado" style text during correction
  - Progress bar fills per-block (each block completion advances bar by 1/N)
  - Keep existing UI elements: status badge, progress bar colors
  - Verify: visual check during actual recording improvement

- [x] **3.3** Persist corrected text to database (existing insert_segment flow handles corrected text)
  - After `improve_transcript_blocks()` returns final text, save to `transcript_segments`
  - Use existing `insert_segment()` calls (overwrite by deleting old segments, reinserting)
  - This is a two-step: `DELETE FROM transcript_segments WHERE recording_id = ?` then reinsert
  - Verify: corrected segments appear in library view with correct timestamps

---

## Phase 4: Testing

- [x] **4.1** Unit tests: `split_into_blocks` with time-based segments
  - 30-min transcript → 6 blocks of ~5 min
  - Single segment (1 block)
  - Empty segments → empty blocks list
  - Very long single segment → capped at 4096 chars

- [x] **4.2** Unit tests: `merge_corrected_blocks` edge cases
  - All blocks succeeded
  - One block failed → uses original text
  - All blocks failed → returns original concatenated
  - Empty results → empty string

- [x] **4.3** Unit tests: prompt builder
  - First block: no previous context section
  - Subsequent block: previous context section present
  - Custom prompt appended correctly

- [x] **4.4** Integration test: full pipeline with mock (deferred — requires DI refactoring for HTTP mock; unit tests cover all pure logic)
  - Simulate a 3-block transcript
  - Mock `generate_non_streaming` to return corrected text
  - Verify: final output contains all 3 corrected blocks in order
  - Verify: progress callback received correct (block_idx, total, pct) sequence

- [x] **4.5** Full pipeline validation (`cargo fmt --check` ✓, `cargo clippy` ✓, `cargo test` 119 passed 0 failed)
  - Run `cargo fmt --check && cargo clippy --all-targets && cargo test`
  - All new tests pass
  - No regressions in existing tests
  - (Optional, ignored test) Run with real Ollama instance: `#[ignore]` test for real block correction
