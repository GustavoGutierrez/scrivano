use scrivano::transcription::{init_whisper, transcribe, TranscriptionLanguage};
use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};
use std::time::Instant;

const BOUNDED_RELEASE_GATE_DURATIONS_SEC: [u32; 3] = [15, 30, 60];
const EXTENDED_BOUNDED_DURATIONS_SEC: [u32; 2] = [180, 300];
const ENABLE_EXTENDED_BOUNDED_ENV: &str = "SCRIVANO_BENCH_EXTENDED";
const BOUNDED_REPORT_PATH: &str = "target/perf-baseline/real-bounded-benchmark.json";
const LONG_BENCHMARK_STATUS: &str = "post-release/performance-lab only";

// ── Integration Tests ───────────────────────────────────────────────────────────

#[test]
#[ignore = "requiere modelo Whisper en models/ggml-small.bin"]
fn integration_transcribe_silence() {
    let ctx = init_whisper("models/ggml-small.bin");
    let silence = vec![0.0_f32; 16_000 * 3];
    let result = transcribe(&ctx, &silence, |_| {});
    assert!(result.is_ok());
}

#[test]
#[ignore = "requiere modelo Whisper en models/ggml-small.bin"]
fn integration_transcribe_short_audio() {
    let ctx = init_whisper("models/ggml-small.bin");
    // ~2 segundos de audio con señal sederhana (sin silencios)
    let audio: Vec<f32> = (0..32_000)
        .map(|i| ((i as f32 * 0.05).sin() * 0.3).abs())
        .collect();
    let final_progress = Arc::new(AtomicI32::new(-1));
    let progress_for_cb = Arc::clone(&final_progress);
    let result = transcribe(&ctx, &audio, move |pct| {
        progress_for_cb.store(pct, Ordering::SeqCst);
    });
    assert!(result.is_ok());
    assert!(
        final_progress.load(Ordering::SeqCst) == 100,
        "transcription should report completion progress"
    );
}

#[test]
#[ignore = "requiere modelo Whisper local y tarda varios minutos"]
fn integration_real_bounded_benchmark_writes_report() {
    let model_path = "models/ggml-medium-q5_0.bin";
    assert!(
        std::path::Path::new(model_path).exists(),
        "missing model file: {model_path}"
    );

    let ctx = init_whisper(model_path);
    let durations_sec = bounded_release_gate_durations();
    let mut rows = Vec::new();

    for seconds in durations_sec {
        let samples_len = 16_000 * seconds as usize;
        let audio: Vec<f32> = (0..samples_len)
            .map(|i| {
                let t = i as f32 / 16_000.0;
                (2.0 * std::f32::consts::PI * 220.0 * t).sin() * 0.2
            })
            .collect();

        let started = Instant::now();
        let result = transcribe(&ctx, &audio, |_| {});
        let elapsed_ms = started.elapsed().as_millis() as u64;

        assert!(
            result.is_ok(),
            "transcription should succeed for {} seconds of generated audio",
            seconds
        );

        rows.push(serde_json::json!({
            "audio_seconds": seconds,
            "elapsed_ms": elapsed_ms,
            "rtf": (elapsed_ms as f64 / 1000.0) / seconds as f64
        }));

        write_bounded_benchmark_report(
            model_path,
            &rows,
            "partial report updated after each duration to avoid losing evidence on abort",
        );
    }

    let output_path = write_bounded_benchmark_report(
        model_path,
        &rows,
        "Accepted release gate: bounded empirical benchmark + documented extrapolation; full 10/30/60 real run is deferred post-release.",
    );

    assert!(output_path.exists());
}

#[test]
#[ignore = "post-release/performance-lab only: full 10/30/60 minute real benchmark"]
fn integration_real_10_30_60_post_stop_benchmark_writes_report() {
    let model_path = "models/ggml-medium-q5_0.bin";
    assert!(
        std::path::Path::new(model_path).exists(),
        "missing model file: {model_path}"
    );

    let ctx = init_whisper(model_path);
    let durations_min = [10_u32, 30_u32, 60_u32];
    let chunk_backlog_sec = 30_u32;
    let mut rows = Vec::new();

    for minutes in durations_min {
        let full_seconds = minutes * 60;
        let full_audio = generated_tone(full_seconds);

        let baseline_started = Instant::now();
        let baseline_result = transcribe(&ctx, &full_audio, |_| {});
        let baseline_ms = baseline_started.elapsed().as_millis() as u64;
        assert!(
            baseline_result.is_ok(),
            "baseline transcription should succeed for {minutes} minutes"
        );

        drop(full_audio);

        let chunk_audio = generated_tone(chunk_backlog_sec);
        let chunked_started = Instant::now();
        let chunked_result = transcribe(&ctx, &chunk_audio, |_| {});
        let chunked_ms = chunked_started.elapsed().as_millis() as u64;
        assert!(
            chunked_result.is_ok(),
            "chunked post-stop transcription should succeed for {chunk_backlog_sec} seconds"
        );

        let reduction_pct = ((baseline_ms as f64 - chunked_ms as f64) / baseline_ms as f64) * 100.0;

        rows.push(serde_json::json!({
            "audio_minutes": minutes,
            "baseline_post_stop_ms": baseline_ms,
            "chunked_post_stop_ms": chunked_ms,
            "chunk_backlog_seconds": chunk_backlog_sec,
            "baseline_rtf": (baseline_ms as f64 / 1000.0) / full_seconds as f64,
            "chunked_backlog_rtf": (chunked_ms as f64 / 1000.0) / chunk_backlog_sec as f64,
            "reduction_pct": reduction_pct,
            "meets_40_percent_target": reduction_pct >= 40.0,
        }));
    }

    let output_dir = std::path::Path::new("target/perf-baseline");
    std::fs::create_dir_all(output_dir).expect("output dir must be creatable");
    let output_path = output_dir.join("real-10-30-60-post-stop-benchmark.json");

    let payload = serde_json::json!({
        "kind": "real-whisper-10-30-60-post-stop-benchmark",
        "model": model_path,
        "sample_rate_hz": 16000,
        "synthetic_signal": "220Hz sine wave",
        "chunk_backlog_seconds": chunk_backlog_sec,
        "notes": "Compares current full-session post-stop transcription against chunked post-stop backlog transcription using the same local Whisper model and synthetic input.",
        "rows": rows,
    });

    std::fs::write(
        &output_path,
        serde_json::to_string_pretty(&payload).expect("json should serialize"),
    )
    .expect("benchmark report must be written");

    assert!(output_path.exists());
}

fn generated_tone(seconds: u32) -> Vec<f32> {
    let samples_len = 16_000 * seconds as usize;
    (0..samples_len)
        .map(|i| {
            let t = i as f32 / 16_000.0;
            (2.0 * std::f32::consts::PI * 220.0 * t).sin() * 0.2
        })
        .collect()
}

fn bounded_release_gate_durations() -> Vec<u32> {
    let mut durations = BOUNDED_RELEASE_GATE_DURATIONS_SEC.to_vec();
    if std::env::var(ENABLE_EXTENDED_BOUNDED_ENV)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        durations.extend_from_slice(&EXTENDED_BOUNDED_DURATIONS_SEC);
    }
    durations
}

fn write_bounded_benchmark_report(
    model_path: &str,
    rows: &[serde_json::Value],
    notes: &str,
) -> std::path::PathBuf {
    let output_path = std::path::Path::new(BOUNDED_REPORT_PATH).to_path_buf();
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).expect("output dir must be creatable");
    }

    let payload = serde_json::json!({
        "kind": "real-whisper-bounded-benchmark",
        "model": model_path,
        "sample_rate_hz": 16000,
        "release_gate": "bounded-empirical-plus-extrapolation",
        "long_benchmark_status": LONG_BENCHMARK_STATUS,
        "rows": rows,
        "notes": notes,
    });

    std::fs::write(
        &output_path,
        serde_json::to_string_pretty(&payload).expect("json should serialize"),
    )
    .expect("benchmark report must be written");

    output_path
}

#[cfg(test)]
mod benchmark_gate_tests {
    #[test]
    fn bounded_release_gate_profile_defaults_to_15_30_60() {
        let durations = super::bounded_release_gate_durations();
        assert_eq!(durations, vec![15_u32, 30_u32, 60_u32]);
    }

    #[test]
    fn long_benchmark_is_marked_post_release_only() {
        assert!(super::LONG_BENCHMARK_STATUS.contains("post-release"));
    }
}

// ── Unit Tests ───────────────────────────────────────────────────────────────────

mod language_tests {
    use super::TranscriptionLanguage;

    #[test]
    fn test_spanish_language_code() {
        let lang = TranscriptionLanguage::Spanish;
        assert_eq!(lang.code(), "es");
    }

    #[test]
    fn test_english_language_code() {
        let lang = TranscriptionLanguage::English;
        assert_eq!(lang.code(), "en");
    }

    #[test]
    fn test_spanish_from_code() {
        assert_eq!(
            TranscriptionLanguage::from_code("es"),
            Some(TranscriptionLanguage::Spanish)
        );
        assert_eq!(
            TranscriptionLanguage::from_code("spanish"),
            Some(TranscriptionLanguage::Spanish)
        );
    }

    #[test]
    fn test_english_from_code() {
        assert_eq!(
            TranscriptionLanguage::from_code("en"),
            Some(TranscriptionLanguage::English)
        );
        assert_eq!(
            TranscriptionLanguage::from_code("english"),
            Some(TranscriptionLanguage::English)
        );
    }

    #[test]
    fn test_unknown_language_returns_none() {
        assert_eq!(TranscriptionLanguage::from_code("fr"), None);
        assert_eq!(TranscriptionLanguage::from_code("invalid"), None);
    }
}
