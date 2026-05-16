use crate::transcription::TranscriptSegment;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkStatus {
    Pending,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ChunkTranscript {
    pub chunk_index: u32,
    pub session_start_sec: f64,
    pub status: ChunkStatus,
    pub segments: Vec<TranscriptSegment>,
}

pub fn merge_ordered_chunks(
    mut chunks: Vec<ChunkTranscript>,
    overlap_sec: f64,
) -> Vec<TranscriptSegment> {
    chunks.sort_by_key(|c| c.chunk_index);
    let mut out: Vec<TranscriptSegment> = Vec::new();

    for chunk in chunks {
        if chunk.status != ChunkStatus::Succeeded {
            continue;
        }

        for seg in chunk.segments {
            let seg_norm = normalize_text(&seg.text);
            let duplicate = out
                .iter()
                .rev()
                .take_while(|prev| prev.end_sec >= seg.start_sec - overlap_sec)
                .any(|prev| normalize_text(&prev.text) == seg_norm);

            if duplicate {
                continue;
            }

            out.push(seg);
        }
    }

    out
}

pub fn failed_chunk_indices(chunks: &[ChunkTranscript]) -> Vec<u32> {
    chunks
        .iter()
        .filter(|c| c.status == ChunkStatus::Failed)
        .map(|c| c.chunk_index)
        .collect()
}

fn normalize_text(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(start: f64, end: f64, text: &str) -> TranscriptSegment {
        TranscriptSegment {
            start_sec: start,
            end_sec: end,
            text: text.to_string(),
        }
    }

    #[test]
    fn overlap_dedup_removes_repeated_boundary_text() {
        let merged = merge_ordered_chunks(
            vec![
                ChunkTranscript {
                    chunk_index: 1,
                    session_start_sec: 20.0,
                    status: ChunkStatus::Succeeded,
                    segments: vec![seg(20.0, 22.0, "Hola mundo"), seg(22.0, 24.0, "seguimos")],
                },
                ChunkTranscript {
                    chunk_index: 2,
                    session_start_sec: 40.0,
                    status: ChunkStatus::Succeeded,
                    segments: vec![
                        seg(24.2, 25.0, "hola mundo"),
                        seg(25.0, 28.0, "nuevo bloque"),
                    ],
                },
            ],
            5.0,
        );

        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].text, "Hola mundo");
        assert_eq!(merged[2].text, "nuevo bloque");
    }

    #[test]
    fn absolute_timestamps_are_preserved_in_output_order() {
        let merged = merge_ordered_chunks(
            vec![
                ChunkTranscript {
                    chunk_index: 2,
                    session_start_sec: 40.0,
                    status: ChunkStatus::Succeeded,
                    segments: vec![seg(40.0, 41.0, "second")],
                },
                ChunkTranscript {
                    chunk_index: 0,
                    session_start_sec: 0.0,
                    status: ChunkStatus::Succeeded,
                    segments: vec![seg(0.0, 1.0, "first")],
                },
            ],
            5.0,
        );

        assert_eq!(merged[0].text, "first");
        assert_eq!(merged[0].start_sec, 0.0);
        assert_eq!(merged[1].text, "second");
        assert_eq!(merged[1].start_sec, 40.0);
    }

    #[test]
    fn retry_targets_only_failed_chunks() {
        let chunks = vec![
            ChunkTranscript {
                chunk_index: 0,
                session_start_sec: 0.0,
                status: ChunkStatus::Succeeded,
                segments: vec![seg(0.0, 1.0, "ok")],
            },
            ChunkTranscript {
                chunk_index: 1,
                session_start_sec: 20.0,
                status: ChunkStatus::Failed,
                segments: vec![],
            },
            ChunkTranscript {
                chunk_index: 2,
                session_start_sec: 40.0,
                status: ChunkStatus::Failed,
                segments: vec![],
            },
        ];

        let failed = failed_chunk_indices(&chunks);
        assert_eq!(failed, vec![1, 2]);
    }
}
