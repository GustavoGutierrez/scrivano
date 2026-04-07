use meet_whisperer::transcription::{init_whisper, transcribe};

#[test]
#[ignore = "requiere models/ggml-small.bin"]
fn integration_transcribe_silence() {
    let ctx = init_whisper("models/ggml-small.bin");
    let silence = vec![0.0_f32; 16_000 * 3];
    let result = transcribe(&ctx, &silence);
    assert!(result.is_ok());
}
