use hound::{SampleFormat, WavSpec, WavWriter};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct ClosedChunk {
    pub chunk_index: u32,
    pub path: PathBuf,
    pub start_sample: u64,
    pub end_sample: u64,
}

pub struct AudioChunker {
    sample_rate: u32,
    chunk_samples: usize,
    overlap_samples: usize,
    output_dir: PathBuf,
    active: Vec<f32>,
    next_chunk_index: u32,
    session_cursor: u64,
}

impl AudioChunker {
    pub fn new(
        output_dir: &Path,
        sample_rate: u32,
        chunk_seconds: u32,
        overlap_seconds: u32,
    ) -> Self {
        Self {
            sample_rate,
            chunk_samples: (sample_rate as usize) * (chunk_seconds as usize),
            overlap_samples: (sample_rate as usize) * (overlap_seconds as usize),
            output_dir: output_dir.to_path_buf(),
            active: Vec::new(),
            next_chunk_index: 0,
            session_cursor: 0,
        }
    }

    pub fn push_samples(&mut self, samples: &[f32]) -> std::io::Result<Vec<ClosedChunk>> {
        self.active.extend_from_slice(samples);
        let mut closed = Vec::new();

        while self.active.len() >= self.chunk_samples {
            let chunk = self.active[..self.chunk_samples].to_vec();
            let closed_chunk = self.persist_chunk(&chunk)?;
            closed.push(closed_chunk);

            let consumed = self.chunk_samples.saturating_sub(self.overlap_samples);
            self.active.drain(..consumed.min(self.active.len()));
        }

        Ok(closed)
    }

    pub fn active_len(&self) -> usize {
        self.active.len()
    }

    fn persist_chunk(&mut self, samples: &[f32]) -> std::io::Result<ClosedChunk> {
        std::fs::create_dir_all(&self.output_dir)?;
        let filename = format!("chunk_{:04}.wav", self.next_chunk_index);
        let path = self.output_dir.join(filename);

        let spec = WavSpec {
            channels: 1,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::create(&path, spec)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;

        for sample in samples {
            let s = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer
                .write_sample(s)
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;
        }

        writer
            .finalize()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;

        let start = self.session_cursor;
        let end = start + samples.len() as u64;
        self.session_cursor += (self.chunk_samples.saturating_sub(self.overlap_samples)) as u64;

        let closed = ClosedChunk {
            chunk_index: self.next_chunk_index,
            path,
            start_sample: start,
            end_sample: end,
        };

        self.next_chunk_index += 1;
        Ok(closed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotates_every_25_seconds() {
        let dir = tempfile::tempdir().expect("tempdir must be created");
        let mut chunker = AudioChunker::new(dir.path(), 16_000, 25, 5);
        let samples = vec![0.1_f32; 16_000 * 25];

        let closed = chunker
            .push_samples(&samples)
            .expect("rotation should work");
        assert_eq!(closed.len(), 1);
        assert_eq!(closed[0].chunk_index, 0);
        assert!(closed[0].path.exists());
    }

    #[test]
    fn retains_5_second_overlap_between_chunks() {
        let dir = tempfile::tempdir().expect("tempdir must be created");
        let mut chunker = AudioChunker::new(dir.path(), 16_000, 25, 5);
        let samples = vec![0.2_f32; 16_000 * 50];

        let closed = chunker
            .push_samples(&samples)
            .expect("rotation should work");
        assert_eq!(closed.len(), 2);
        assert_eq!(closed[0].start_sample, 0);
        assert_eq!(closed[1].start_sample, 16_000 * 20);
    }

    #[test]
    fn active_buffer_stays_bounded() {
        let dir = tempfile::tempdir().expect("tempdir must be created");
        let mut chunker = AudioChunker::new(dir.path(), 16_000, 25, 5);

        let samples = vec![0.3_f32; 16_000 * 120];
        let _ = chunker
            .push_samples(&samples)
            .expect("rotation should work");

        assert!(chunker.active_len() <= 16_000 * 30);
    }
}
