use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// Supported languages for transcription
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TranscriptionLanguage {
    Spanish,
    English,
}

impl TranscriptionLanguage {
    pub fn code(&self) -> &'static str {
        match self {
            TranscriptionLanguage::Spanish => "es",
            TranscriptionLanguage::English => "en",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_lowercase().as_str() {
            "es" | "spanish" => Some(TranscriptionLanguage::Spanish),
            "en" | "english" => Some(TranscriptionLanguage::English),
            _ => None,
        }
    }
}

/// Transcript segment with timestamps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub start_sec: f64,
    pub end_sec: f64,
    pub text: String,
}

/// Initialize Whisper context with given model path
pub fn init_whisper(model_path: &str) -> WhisperContext {
    WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
        .expect("Failed to load Whisper model")
}

/// Transcribe `audio` (16 kHz mono f32) using the given Whisper context.
/// Returns both the full text and individual segments with timestamps.
///
/// `progress_cb` is called with values 0–100 as transcription progresses.
pub fn transcribe_with_segments<F>(
    ctx: &WhisperContext,
    audio: &[f32],
    language: TranscriptionLanguage,
    progress_cb: F,
) -> Result<(String, Vec<TranscriptSegment>)>
where
    F: Fn(i32) + Send + 'static,
{
    let cb = Arc::new(Mutex::new(progress_cb));

    // Estimate processing time: ~0.5× real-time for ggml-tiny on modern hardware.
    let estimated_secs = ((audio.len() as f64 / 16_000.0) * 0.5).max(0.5);

    // Notify start
    cb.lock().unwrap()(0);

    // Background thread: increment 1→95 over estimated_secs
    let cb_thread = Arc::clone(&cb);
    let done_flag = Arc::new(Mutex::new(false));
    let done_thread = Arc::clone(&done_flag);

    let step_duration = std::time::Duration::from_millis(((estimated_secs / 95.0) * 1000.0) as u64);

    let handle = std::thread::spawn(move || {
        for pct in 1_i32..=95 {
            std::thread::sleep(step_duration);
            if *done_thread.lock().unwrap() {
                break;
            }
            if let Ok(f) = cb_thread.lock() {
                f(pct);
            }
        }
    });

    let mut state = ctx
        .create_state()
        .context("Failed to create Whisper state")?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

    // Set language
    params.set_language(Some(language.code()));

    // Performance: more threads, no historical context, single segment
    params.set_n_threads(8);
    params.set_no_context(true);
    params.set_single_segment(true);
    params.set_audio_ctx(0);

    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(true); // Enable timestamps for segment extraction
    params.set_translate(false);

    state
        .full(params, audio)
        .context("Failed to run Whisper transcription")?;

    // Signal progress thread to stop, then report 100%
    *done_flag.lock().unwrap() = true;
    let _ = handle.join();
    cb.lock().unwrap()(100);

    // Collect segments with timestamps
    let mut segments = Vec::new();
    let mut full_text = String::new();
    let num_segments = state.full_n_segments();

    for i in 0..num_segments {
        if let Some(segment) = state.get_segment(i) {
            let seg_text = segment.to_string();
            let seg_text = seg_text.trim();
            if seg_text.is_empty() {
                continue;
            }

            // Get timestamps (in centiseconds, convert to seconds)
            let start = segment.start_timestamp() as f64 / 100.0;
            let end = segment.end_timestamp() as f64 / 100.0;

            segments.push(TranscriptSegment {
                start_sec: start,
                end_sec: end,
                text: seg_text.to_string(),
            });

            if !full_text.is_empty() {
                full_text.push(' ');
            }
            full_text.push_str(seg_text);
        }
    }

    Ok((full_text, segments))
}

/// Transcribe `audio` (16 kHz mono f32) using the given Whisper context.
/// Legacy function for backward compatibility - returns only text.
///
/// `progress_cb` is called with values 0–100 as transcription progresses.
pub fn transcribe<F>(ctx: &WhisperContext, audio: &[f32], progress_cb: F) -> Result<String>
where
    F: Fn(i32) + Send + 'static,
{
    let (text, _) =
        transcribe_with_segments(ctx, audio, TranscriptionLanguage::Spanish, progress_cb)?;
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_codes_are_correct() {
        assert_eq!(TranscriptionLanguage::Spanish.code(), "es");
        assert_eq!(TranscriptionLanguage::English.code(), "en");
    }

    #[test]
    fn language_from_code_works() {
        assert_eq!(
            TranscriptionLanguage::from_code("es"),
            Some(TranscriptionLanguage::Spanish)
        );
        assert_eq!(
            TranscriptionLanguage::from_code("en"),
            Some(TranscriptionLanguage::English)
        );
    }

    #[test]
    #[ignore = "requires GGML model on disk"]
    fn transcribe_silence_returns_string() {
        let ctx = init_whisper("models/ggml-tiny.bin");
        let silence = vec![0.0_f32; 16_000 * 2];
        let result = transcribe(&ctx, &silence, |_| {});
        assert!(result.is_ok(), "transcribe must return Ok for silence");
    }

    #[test]
    #[ignore = "requires GGML model on disk"]
    fn transcribe_empty_audio_does_not_panic() {
        let ctx = init_whisper("models/ggml-tiny.bin");
        let result = transcribe(&ctx, &[], |_| {});
        let _ = result;
    }
}
