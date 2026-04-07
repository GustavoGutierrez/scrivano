use anyhow::{Context, Result};
use std::sync::{Arc, Mutex};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub fn init_whisper(model_path: &str) -> WhisperContext {
    WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
        .expect("Failed to load Whisper model")
}

/// Transcribe `audio` (16 kHz mono f32) using the given Whisper context.
///
/// `progress_cb` is called with values 0–100 as transcription progresses.
/// Progress is simulated via a background thread since whisper-rs 0.15
/// does not expose a safe, owned progress callback API.
pub fn transcribe<F>(ctx: &WhisperContext, audio: &[f32], progress_cb: F) -> Result<String>
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

    let step_duration =
        std::time::Duration::from_millis(((estimated_secs / 95.0) * 1000.0) as u64);

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

    // Español — evita que Whisper pierda tiempo detectando idioma
    params.set_language(Some("es"));

    // Rendimiento: más hilos, sin contexto histórico, segmento único
    params.set_n_threads(8);
    params.set_no_context(true);
    params.set_single_segment(true);
    params.set_audio_ctx(0);

    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_translate(false);

    state
        .full(params, audio)
        .context("Failed to run Whisper transcription")?;

    // Signal progress thread to stop, then report 100%
    *done_flag.lock().unwrap() = true;
    let _ = handle.join();
    cb.lock().unwrap()(100);

    let mut text = String::new();
    let num_segments = state.full_n_segments();
    for i in 0..num_segments {
        if let Some(segment) = state.get_segment(i) {
            let seg_text = segment.to_string();
            let seg_text = seg_text.trim();
            if seg_text.is_empty() {
                continue;
            }
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(seg_text);
        }
    }

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

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