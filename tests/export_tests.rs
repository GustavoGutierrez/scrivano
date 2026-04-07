//! Tests para el módulo de exportación

use scrivano::export::{format_srt_timestamp, format_timestamp, format_vtt_timestamp};
use scrivano::transcription::TranscriptSegment;

// ── Unit Tests ───────────────────────────────────────────────────────────────────

mod timestamp_format_tests {
    use super::*;

    #[test]
    fn test_format_timestamp_basic() {
        let result = format_timestamp(0.0);
        assert_eq!(result, "00:00:00");
    }

    #[test]
    fn test_format_timestamp_minutes() {
        let result = format_timestamp(125.0); // 2:05
        assert_eq!(result, "00:02:05");
    }

    #[test]
    fn test_format_timestamp_hours() {
        let result = format_timestamp(3665.0); // 1:01:05
        assert_eq!(result, "01:01:05");
    }

    #[test]
    fn test_srt_timestamp_basic() {
        let result = format_srt_timestamp(0.0);
        assert_eq!(result, "00:00:00,000");
    }

    #[test]
    fn test_srt_timestamp_with_milliseconds() {
        let result = format_srt_timestamp(5.5); // 5.5 seconds
        assert_eq!(result, "00:00:05,500");
    }

    #[test]
    fn test_srt_timestamp_minutes() {
        let result = format_srt_timestamp(65.0); // 1:05
        assert_eq!(result, "00:01:05,000");
    }

    #[test]
    fn test_vtt_timestamp_basic() {
        let result = format_vtt_timestamp(0.0);
        assert_eq!(result, "00:00:00.000");
    }

    #[test]
    fn test_vtt_timestamp_with_milliseconds() {
        let result = format_vtt_timestamp(10.25); // 10.25 seconds
        assert_eq!(result, "00:00:10.250");
    }

    #[test]
    fn test_vtt_timestamp_hours() {
        let result = format_vtt_timestamp(3661.5); // 1:01:01.5
        assert_eq!(result, "01:01:01.500");
    }
}

mod segment_tests {
    use super::*;

    #[test]
    fn test_transcript_segment_creation() {
        let segment = TranscriptSegment {
            start_sec: 0.0,
            end_sec: 5.0,
            text: "Test text".to_string(),
        };
        assert_eq!(segment.start_sec, 0.0);
        assert_eq!(segment.end_sec, 5.0);
        assert_eq!(segment.text, "Test text");
    }

    #[test]
    fn test_transcript_segment_empty_text() {
        let segment = TranscriptSegment {
            start_sec: 10.0,
            end_sec: 15.0,
            text: String::new(),
        };
        assert!(segment.text.is_empty());
    }

    #[test]
    fn test_transcript_segment_long_duration() {
        let segment = TranscriptSegment {
            start_sec: 3600.0, // 1 hour
            end_sec: 7200.0,   // 2 hours
            text: "Long recording".to_string(),
        };
        assert_eq!(segment.start_sec, 3600.0);
        assert_eq!(segment.end_sec, 7200.0);
    }
}
