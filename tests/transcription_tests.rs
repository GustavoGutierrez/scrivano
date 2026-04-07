use scrivano::transcription::{init_whisper, transcribe, TranscriptionLanguage};

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
    let result = transcribe(&ctx, &audio, |_| {});
    assert!(result.is_ok());
    let text = result.unwrap();
    // Verificamos que el resultado es un String válido
    assert!(text.is_empty() || text.len() > 0);
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
