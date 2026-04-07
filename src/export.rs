//! Export functionality for recordings, transcripts, and summaries.
//!
//! Supports TXT, Markdown, JSON, SRT, and WebVTT formats.

use crate::database::{Highlight, RecordingEntry, Summary, TranscriptSegment};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Export data structure for JSON serialization
#[derive(Serialize, Deserialize)]
pub struct ExportData {
    pub recording: RecordingExport,
    pub segments: Vec<SegmentExport>,
    pub highlights: Vec<HighlightExport>,
    pub summaries: Vec<SummaryExport>,
}

#[derive(Serialize, Deserialize)]
pub struct RecordingExport {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
    pub created_at: String,
    pub duration_secs: f64,
    pub ollama_used: bool,
    pub ollama_model: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SegmentExport {
    pub start_sec: f64,
    pub end_sec: f64,
    pub text: String,
}

#[derive(Serialize, Deserialize)]
pub struct HighlightExport {
    pub timestamp_sec: f64,
    pub label: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SummaryExport {
    pub template: String,
    pub content: String,
    pub model_name: Option<String>,
}

/// Export transcript to plain text file
pub fn export_to_txt(segments: &[TranscriptSegment], output_path: &Path) -> Result<()> {
    let mut file = File::create(output_path).context("Failed to create TXT file")?;

    for segment in segments {
        writeln!(file, "{}", segment.text)?;
    }

    Ok(())
}

/// Export transcript to Markdown file with timestamp headers
pub fn export_to_markdown(segments: &[TranscriptSegment], output_path: &Path) -> Result<()> {
    let mut file = File::create(output_path).context("Failed to create Markdown file")?;

    writeln!(file, "# Transcript\n")?;

    for segment in segments {
        let start = format_timestamp(segment.start_sec);
        writeln!(file, "## [{}]", start)?;
        writeln!(file, "{}\n", segment.text)?;
    }

    Ok(())
}

/// Export all data to JSON file
pub fn export_to_json(
    recording: &RecordingEntry,
    segments: &[TranscriptSegment],
    highlights: &[Highlight],
    summaries: &[Summary],
    output_path: &Path,
) -> Result<()> {
    let export_data = ExportData {
        recording: RecordingExport {
            id: recording.id,
            filename: recording.filename.clone(),
            filepath: recording.filepath.clone(),
            created_at: recording.created_at.clone(),
            duration_secs: recording.duration_secs,
            ollama_used: recording.ollama_used,
            ollama_model: recording.ollama_model.clone(),
        },
        segments: segments
            .iter()
            .map(|s| SegmentExport {
                start_sec: s.start_sec,
                end_sec: s.end_sec,
                text: s.text.clone(),
            })
            .collect(),
        highlights: highlights
            .iter()
            .map(|h| HighlightExport {
                timestamp_sec: h.timestamp_sec,
                label: h.label.clone(),
            })
            .collect(),
        summaries: summaries
            .iter()
            .map(|s| SummaryExport {
                template: s.template.clone(),
                content: s.content.clone(),
                model_name: s.model_name.clone(),
            })
            .collect(),
    };

    let json = serde_json::to_string_pretty(&export_data).context("Failed to serialize JSON")?;

    let mut file = File::create(output_path).context("Failed to create JSON file")?;
    file.write_all(json.as_bytes())?;

    Ok(())
}

/// Export transcript to SRT subtitle format
pub fn export_to_srt(segments: &[TranscriptSegment], output_path: &Path) -> Result<()> {
    let mut file = File::create(output_path).context("Failed to create SRT file")?;

    for (i, segment) in segments.iter().enumerate() {
        let index = i + 1;
        let start = format_srt_timestamp(segment.start_sec);
        let end = format_srt_timestamp(segment.end_sec);

        writeln!(file, "{}", index)?;
        writeln!(file, "{} --> {}", start, end)?;
        writeln!(file, "{}\n", segment.text)?;
    }

    Ok(())
}

/// Export transcript to WebVTT subtitle format
pub fn export_to_webvtt(segments: &[TranscriptSegment], output_path: &Path) -> Result<()> {
    let mut file = File::create(output_path).context("Failed to create WebVTT file")?;

    writeln!(file, "WEBVTT\n")?;

    for segment in segments {
        let start = format_vtt_timestamp(segment.start_sec);
        let end = format_vtt_timestamp(segment.end_sec);

        writeln!(file, "{} --> {}", start, end)?;
        writeln!(file, "{}\n", segment.text)?;
    }

    Ok(())
}

/// Format timestamp as HH:MM:SS
pub fn format_timestamp(seconds: f64) -> String {
    let total = seconds as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

/// Format timestamp for SRT (HH:MM:SS,mmm)
pub fn format_srt_timestamp(seconds: f64) -> String {
    let total_ms = (seconds * 1000.0) as u64;
    let hours = total_ms / 3600000;
    let minutes = (total_ms % 3600000) / 60000;
    let seconds = (total_ms % 60000) / 1000;
    let millis = total_ms % 1000;
    format!("{:02}:{:02}:{:02},{:03}", hours, minutes, seconds, millis)
}

/// Format timestamp for WebVTT (HH:MM:SS.mmm)
pub fn format_vtt_timestamp(seconds: f64) -> String {
    let total_ms = (seconds * 1000.0) as u64;
    let hours = total_ms / 3600000;
    let minutes = (total_ms % 3600000) / 60000;
    let seconds = (total_ms % 60000) / 1000;
    let millis = total_ms % 1000;
    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
}
