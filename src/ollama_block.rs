//! Ollama block correction pipeline.
//!
//! Splits long transcripts into time-based blocks (~5 minutes each) and corrects
//! each block independently through Ollama, then merges the results.
//!
//! Short transcripts (≤ 2000 chars) are delegated to single-shot `improve_transcript()`.

use crate::ollama;
use crate::transcription::TranscriptSegment;
use anyhow::Result;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Result of correcting a single block.
#[derive(Debug, Clone)]
pub struct BlockResult {
    pub block_index: usize,
    pub original_text: String,
    pub corrected_text: Option<String>,
    pub error: Option<String>,
}

/// Progress callback: (block_index, total_blocks, block_progress_pct 0..100)
pub type BlockProgressFn = dyn Fn(usize, usize, i32) + Send + Sync + 'static;

// ── Public API ────────────────────────────────────────────────────────────────

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
    segments: &[TranscriptSegment],
    block_minutes: u32,
    progress_cb: F,
    custom_prompt: Option<&str>,
) -> Result<String>
where
    F: Fn(usize, usize, i32) + Send + Sync + 'static,
{
    // Empty transcript: return immediately
    if transcript.trim().is_empty() {
        progress_cb(0, 1, 100);
        return Ok(transcript.to_string());
    }

    // Check Ollama availability
    let client = ollama::OllamaClient::default_client();
    if !client.is_available() {
        eprintln!("[ollama_block] Ollama no disponible, devolviendo transcript original");
        progress_cb(0, 1, 100);
        return Ok(transcript.to_string());
    }

    // Short transcript: delegate to single-shot
    if transcript.len() <= 2000 {
        eprintln!(
            "[ollama_block] Transcript corto ({} chars), usando single-shot",
            transcript.len()
        );
        return ollama::improve_transcript(model, transcript, |_| {}, custom_prompt);
    }

    // Split into blocks
    let blocks = split_into_blocks(transcript, segments, block_minutes);
    let total_blocks = blocks.len();
    eprintln!(
        "[ollama_block] Transcript dividido en {} bloques (~{} min c/u)",
        total_blocks, block_minutes
    );

    if total_blocks == 0 {
        progress_cb(0, 1, 100);
        return Ok(transcript.to_string());
    }

    // Process each block
    let mut results: Vec<BlockResult> = Vec::with_capacity(total_blocks);
    let mut previous_context: Option<String> = None;

    for (idx, block_text) in blocks.iter().enumerate() {
        eprintln!(
            "[ollama_block] Procesando bloque {}/{} ({} chars)",
            idx + 1,
            total_blocks,
            block_text.len()
        );

        // Report progress: block start
        progress_cb(idx, total_blocks, 0);

        // Build prompt with context from previous block
        let prompt = build_block_prompt(block_text, previous_context.as_deref());

        // Try corection with one retry
        let (corrected, error) = try_correct_block(&client, model, &prompt);

        let result = BlockResult {
            block_index: idx,
            original_text: block_text.clone(),
            corrected_text: corrected,
            error,
        };

        // Update previous context: last 200 chars of this block's original text
        previous_context = Some(
            block_text
                .chars()
                .rev()
                .take(200)
                .collect::<String>()
                .chars()
                .rev()
                .collect(),
        );

        results.push(result);

        // Report progress: block done
        progress_cb(idx, total_blocks, 100);
    }

    // Merge results
    let final_text = merge_corrected_blocks(&blocks, &results);
    let all_failed = results.iter().all(|r| r.corrected_text.is_none());

    eprintln!(
        "[ollama_block] Corrección completada: {} blocks, {} corregidos, {} fallidos",
        total_blocks,
        results
            .iter()
            .filter(|r| r.corrected_text.is_some())
            .count(),
        results
            .iter()
            .filter(|r| r.corrected_text.is_none())
            .count(),
    );

    if all_failed {
        eprintln!("[ollama_block] Todos los bloques fallaron, devolviendo original");
        return Ok(transcript.to_string());
    }

    Ok(final_text)
}

/// Attempt to correct a single block through Ollama.
/// Retries once on failure.
fn try_correct_block(
    client: &ollama::OllamaClient,
    model: &str,
    prompt: &str,
) -> (Option<String>, Option<String>) {
    // First attempt
    match client.generate_non_streaming(prompt, model) {
        Ok(text) => {
            let cleaned = text.trim().to_string();
            if cleaned.is_empty() {
                return (None, Some("Respuesta vacía".to_string()));
            }
            return (Some(cleaned), None);
        }
        Err(e) => {
            eprintln!(
                "[ollama_block] Primer intento falló: {}. Reintentando...",
                e
            );
        }
    }

    // Retry after 1 second
    std::thread::sleep(std::time::Duration::from_secs(1));
    eprintln!("[ollama_block] Reintentando bloque...");

    match client.generate_non_streaming(prompt, model) {
        Ok(text) => {
            let cleaned = text.trim().to_string();
            if cleaned.is_empty() {
                return (None, Some("Respuesta vacía en reintento".to_string()));
            }
            (Some(cleaned), None)
        }
        Err(e) => (None, Some(format!("Falló tras reintento: {}", e))),
    }
}

// ── Block splitting ───────────────────────────────────────────────────────────

/// Split transcript text into blocks of approximately `block_minutes` minutes.
/// Uses segments with timestamps for accurate time-based splitting.
pub fn split_into_blocks(
    transcript: &str,
    segments: &[TranscriptSegment],
    block_minutes: u32,
) -> Vec<String> {
    if segments.is_empty() {
        return Vec::new();
    }

    let block_duration = (block_minutes * 60) as f64;
    let mut blocks: Vec<String> = Vec::new();
    let mut current_block_segments: Vec<String> = Vec::new();
    let mut block_start_sec = segments[0].start_sec;

    for segment in segments {
        let segment_end = segment.end_sec;

        // Start a new block if this segment would push current block beyond duration
        if !current_block_segments.is_empty() && (segment_end - block_start_sec) > block_duration {
            let block_text = current_block_segments.join(" ");
            blocks.push(cap_block_text(&block_text));
            current_block_segments.clear();
            block_start_sec = segment.start_sec;
        }

        current_block_segments.push(segment.text.clone());
    }

    // Flush remaining segments as the last block
    if !current_block_segments.is_empty() {
        let block_text = current_block_segments.join(" ");
        blocks.push(cap_block_text(&block_text));
    }

    // Fallback: if no blocks were created but transcript has text, make one block
    if blocks.is_empty() && !transcript.is_empty() {
        blocks.push(cap_block_text(transcript));
    }

    blocks
}

/// Cap block text at 4096 characters to respect model context limits.
fn cap_block_text(text: &str) -> String {
    if text.len() <= 4096 {
        text.to_string()
    } else {
        // Truncate at last word boundary before 4096
        let mut end = 4096;
        while end > 0 && text.as_bytes().get(end).is_some_and(|&b| b != b' ') {
            end -= 1;
        }
        if end == 0 {
            end = 4096; // No word boundary found, cut at 4096
        }
        text[..end].trim_end().to_string()
    }
}

// ── Block merging ─────────────────────────────────────────────────────────────

/// Merge corrected blocks back into a single transcript, preserving ordering
/// and using corrected text where available, original text where not.
pub fn merge_corrected_blocks(original_blocks: &[String], results: &[BlockResult]) -> String {
    let mut parts: Vec<&str> = Vec::new();

    for result in results {
        let text = match &result.corrected_text {
            Some(corrected) if !corrected.trim().is_empty() => corrected.as_str(),
            _ => {
                // Use original text for this block
                original_blocks
                    .get(result.block_index)
                    .map(|s| s.as_str())
                    .unwrap_or("")
            }
        };
        if !text.trim().is_empty() {
            parts.push(text);
        }
    }

    parts.join(" ")
}

// ── Prompt builder ────────────────────────────────────────────────────────────

/// Build the system prompt for correcting a single block.
/// `previous_context` is the last ~200 chars of the previous block (for continuity).
pub fn build_block_prompt(block_text: &str, previous_context: Option<&str>) -> String {
    let mut prompt = String::from(
        "Eres un corrector de transcripciones de audio. Corrige SOLO el siguiente fragmento.\n\
         Reglas:\n\
         1. Corrige errores ortográficos y gramaticales.\n\
         2. Agrega puntuación donde falte.\n\
         3. NO agregues información que no está en el texto.\n\
         4. NO modifiques ni agregues timestamps.\n\
         5. NO escribas prefijos, comentarios ni explicaciones.\n\
         6. Devuelve SOLO el texto corregido.",
    );

    if let Some(ctx) = previous_context {
        if !ctx.trim().is_empty() {
            prompt.push_str("\n\n[Contexto previo (solo referencia, NO incluir en salida):]\n");
            prompt.push_str(ctx);
        }
    }

    prompt.push_str("\n\n[Fragmento a corregir:]\n");
    prompt.push_str(block_text);
    prompt.push_str("\n\nTexto corregido:");

    prompt
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: create a TranscriptSegment with given timestamps and text.
    fn seg(start: f64, end: f64, text: &str) -> TranscriptSegment {
        TranscriptSegment {
            start_sec: start,
            end_sec: end,
            text: text.to_string(),
        }
    }

    // ── 1.2: split_into_blocks tests ─────────────────────────────────────────

    #[test]
    fn split_into_blocks_with_30_min_transcript_produces_6_blocks() {
        // 30 minutes = 1800 seconds.
        // Generate segments spanning 30 minutes.
        let mut segments = Vec::new();
        for i in 0..180 {
            // Each segment = 10 seconds
            let start = (i * 10) as f64;
            let end = start + 10.0;
            segments.push(seg(start, end, &format!("segment {}", i)));
        }
        let transcript = segments
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        let blocks = split_into_blocks(&transcript, &segments, 5);
        assert_eq!(blocks.len(), 6, "30 min / 5 min blocks = 6 blocks");
    }

    #[test]
    fn split_into_blocks_with_single_segment_produces_1_block() {
        let segments = vec![seg(0.0, 60.0, "short transcript")];
        let transcript = "short transcript".to_string();

        let blocks = split_into_blocks(&transcript, &segments, 5);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0], "short transcript");
    }

    #[test]
    fn split_into_blocks_with_empty_segments_produces_empty_vec() {
        let blocks = split_into_blocks("some text", &[], 5);
        assert!(blocks.is_empty());
    }

    #[test]
    fn split_into_blocks_respects_char_cap() {
        // Generate segments that would produce a >4096 char block.
        // One very long segment should be capped.
        let long_text = "word ".repeat(1000); // ~5000 chars
        let segments = vec![seg(0.0, 300.0, &long_text)]; // 5 minutes
        let transcript = long_text.clone();

        let blocks = split_into_blocks(&transcript, &segments, 5);
        assert!(!blocks.is_empty());
        // The block should be capped at 4096 chars
        for block in &blocks {
            assert!(
                block.len() <= 4096,
                "block length {} exceeds 4096 cap",
                block.len()
            );
        }
    }

    // ── 1.3: merge_corrected_blocks tests ────────────────────────────────────

    #[test]
    fn merge_all_succeeded_returns_corrected_texts_joined() {
        let original = vec!["hola".to_string(), "mundo".to_string()];
        let results = vec![
            BlockResult {
                block_index: 0,
                original_text: "hola".to_string(),
                corrected_text: Some("Hola".to_string()),
                error: None,
            },
            BlockResult {
                block_index: 1,
                original_text: "mundo".to_string(),
                corrected_text: Some("mundo.".to_string()),
                error: None,
            },
        ];

        let merged = merge_corrected_blocks(&original, &results);
        assert_eq!(merged, "Hola mundo.");
    }

    #[test]
    fn merge_one_failed_uses_original_for_that_block() {
        let original = vec!["texto uno".to_string(), "texto dos".to_string()];
        let results = vec![
            BlockResult {
                block_index: 0,
                original_text: "texto uno".to_string(),
                corrected_text: Some("Texto uno.".to_string()),
                error: None,
            },
            BlockResult {
                block_index: 1,
                original_text: "texto dos".to_string(),
                corrected_text: None,
                error: Some("timeout".to_string()),
            },
        ];

        let merged = merge_corrected_blocks(&original, &results);
        assert!(merged.contains("Texto uno."));
        assert!(merged.contains("texto dos"));
    }

    #[test]
    fn merge_all_failed_returns_original_concatenated() {
        let original = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let results = vec![
            BlockResult {
                block_index: 0,
                original_text: "a".to_string(),
                corrected_text: None,
                error: Some("err".to_string()),
            },
            BlockResult {
                block_index: 1,
                original_text: "b".to_string(),
                corrected_text: None,
                error: Some("err".to_string()),
            },
            BlockResult {
                block_index: 2,
                original_text: "c".to_string(),
                corrected_text: None,
                error: Some("err".to_string()),
            },
        ];

        let merged = merge_corrected_blocks(&original, &results);
        assert_eq!(merged, "a b c");
    }

    #[test]
    fn merge_empty_results_returns_empty_string() {
        let merged = merge_corrected_blocks(&[], &[]);
        assert_eq!(merged, "");
    }

    // ── 1.4: build_block_prompt tests ───────────────────────────────────────

    #[test]
    fn prompt_builder_first_block_has_no_context() {
        let prompt = build_block_prompt("texto de prueba", None);
        assert!(prompt.contains("texto de prueba"));
        assert!(!prompt.contains("Contexto previo"));
    }

    #[test]
    fn prompt_builder_subsequent_block_has_context() {
        let prompt = build_block_prompt("nuevo bloque", Some("contexto anterior"));
        assert!(prompt.contains("nuevo bloque"));
        assert!(prompt.contains("contexto anterior"));
        assert!(prompt.contains("Contexto previo"));
    }

    #[test]
    fn prompt_builder_instructs_no_timestamp_modification() {
        let prompt = build_block_prompt("texto", None);
        let lower = prompt.to_lowercase();
        // Must contain instruction about timestamps
        assert!(
            lower.contains("timestamp") || lower.contains("marca"),
            "Prompt must forbid timestamp modification. Got: {}",
            prompt
        );
    }
}
