# Spec: Ollama Block Correction

## Overview

The Ollama block correction feature MUST replace single-shot transcript improvement with a block-by-block pipeline for long transcripts. It SHALL preserve transcript structure (segment order, timestamps), MUST NOT block the UI, and SHALL degrade gracefully when Ollama is unavailable.

---

## Scenario: Short transcript uses single-shot fallback

**Given** a transcript with ≤ 2000 characters
**When** `improve_transcript_blocks()` is called
**Then** the function SHALL delegate to the existing single-shot `improve_transcript()` path
**And** it SHALL NOT split the transcript into blocks

## Scenario: Long transcript is split into blocks

**Given** a transcript with > 2000 characters containing segments spanning 60 minutes
**When** `improve_transcript_blocks()` is called with `block_minutes = 5`
**Then** the function SHALL group segments into text blocks of approximately 5 minutes of spoken content each
**And** each block SHALL NOT exceed 4096 characters
**And** each block SHALL contain complete segments (no partial segment splitting)

## Scenario: Blocks are corrected sequentially with progress

**Given** a transcript split into N blocks
**When** `improve_transcript_blocks()` processes each block through Ollama
**Then** the progress callback SHALL be called with `(block_index: usize, total_blocks: usize, block_pct: i32)`
**And** `block_pct` SHALL range from 0 to 100 for each block
**And** `block_index` SHALL increment from 0 to N-1

## Scenario: Corrected blocks are merged preserving timestamps

**Given** 3 corrected blocks, each containing segments with absolute `start_sec` and `end_sec`
**When** the merge function combines them into a final transcript
**Then** all segments SHALL appear in chronological order
**And** no segment SHALL have its `start_sec` or `end_sec` modified
**And** the full concatenated text SHALL be returned as a single string with segments separated by spaces

## Scenario: Failed block leaves original text intact

**Given** a block whose Ollama correction request fails (network error, model error, timeout)
**When** the block pipeline encounters the failure
**Then** the original uncorrected text for that block SHALL be used in the final transcript
**And** the block SHALL be logged as failed with its index
**And** processing SHALL continue with the next block (the pipeline MUST NOT abort)

## Scenario: All blocks failed returns original transcript

**Given** a transcript split into 4 blocks
**When** all 4 block corrections fail
**Then** `improve_transcript_blocks()` SHALL return the original transcript unchanged
**And** the return SHALL be `Ok(original_text)` — it MUST NOT return an error

## Scenario: Ollama completely unavailable returns original

**Given** Ollama is not reachable (`is_available()` returns false)
**When** `improve_transcript_blocks()` is called
**Then** the function SHALL return the original transcript immediately
**And** no HTTP requests SHALL be attempted
**And** the return SHALL be `Ok(original_text)`

## Scenario: Empty transcript returns empty

**Given** an empty or whitespace-only transcript
**When** `improve_transcript_blocks()` is called
**Then** the function SHALL return the empty transcript unchanged
**And** no Ollama requests SHALL be attempted

## Scenario: Block boundaries handle consecutive segments correctly

**Given** segments spanning from `00:00` to `15:00` split into 3 blocks of ~5 min each
**When** the block builder groups segments
**Then** the last segment of block 0 and the first segment of block 1 SHALL be from adjacent time ranges (e.g., block 0 ends at ~04:58, block 1 starts at ~05:02)
**And** no segment SHALL appear in more than one block

## Scenario: Block correction does not modify timestamps in LLM output

**Given** a block of transcript text being sent to Ollama
**When** the system prompt is constructed for block correction
**Then** the prompt SHALL explicitly instruct the model NOT to add, remove, or modify any timestamps
**And** the prompt SHALL instruct the model to return ONLY the corrected text, without prefixes or comments

## Scenario: Corrected text can be saved back to database segments

**Given** a recording with segments in `transcript_segments` table
**When** the corrected transcript replaces the original
**Then** the corrected text per segment SHALL be updatable via `insert_segment()` (overwrite by segment index)
**And** segment timestamps SHALL remain unchanged

---

## API Contract

### `src/ollama_block.rs`

```rust
/// Progress callback: (block_index, total_blocks, block_progress_pct 0..100)
pub type BlockProgressFn = dyn Fn(usize, usize, i32) + Send + Sync + 'static;

/// Improve a transcript by splitting into blocks and correcting each block
/// through Ollama, then merging the results.
///
/// Automatically falls back to single-shot `improve_transcript()` for short
/// transcripts (≤ 2000 chars).
///
/// Returns the full corrected text, or the original text if correction fails.
pub fn improve_transcript_blocks<F>(
    model: &str,
    transcript: &str,
    block_minutes: u32,      // approximate minutes per block (default: 5)
    progress_cb: F,
    custom_prompt: Option<&str>,
) -> Result<String>
where
    F: Fn(usize, usize, i32) + Send + Sync + 'static;

/// Split transcript text into blocks of approximately `block_minutes` minutes.
/// Also accepts segments with timestamps for more accurate splitting.
pub fn split_into_blocks(
    transcript: &str,
    segments: &[TranscriptSegment],
    block_minutes: u32,
) -> Vec<String>;

/// Merge corrected blocks back into a single transcript, preserving ordering
/// and removing duplicated boundary text.
pub fn merge_corrected_blocks(
    original_blocks: &[String],
    corrected_blocks: &[BlockResult],
) -> String;

#[derive(Debug, Clone)]
pub struct BlockResult {
    pub block_index: usize,
    pub original_text: String,
    pub corrected_text: Option<String>,
    pub error: Option<String>,
}
```

---

## Non-Functional Requirements

| Requirement | Specification |
|-------------|---------------|
| **UI blocking** | MUST NOT block egui frame updates during correction. Corrections run on background thread. |
| **Memory** | Ollama request context per block ≤ 2048 tokens (`num_predict`), keeping VRAM/RAM bounded. |
| **Timeout** | Per-block HTTP timeout: 60 seconds. Total pipeline timeout: 10 minutes. |
| **Progress granularity** | Progress updated per block, not simulated per-second. Real HTTP completion triggers progress update. |
| **Logging** | Each block MUST log: index, start time, duration, char count in/out, success/failure. |

---

## Error Handling Contract

| Error condition | Behavior |
|-----------------|----------|
| Ollama unreachable | Return original transcript, `Ok(...)` |
| Single block fails | Use original text for that block, continue pipeline |
| All blocks fail | Return original transcript, `Ok(...)` |
| Block response empty | Use original text for that block |
| Block response is error marker | Use original text, log warning |
| Transcript too long (> 100k chars) | Split into blocks, but log warning about potential quality degradation |
| Custom prompt injection | Sanitize: trim only, no special escaping needed (JSON handles it) |
