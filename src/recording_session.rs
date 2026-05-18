use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs::{self, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestEntry {
    pub chunk_index: u32,
    pub filename: String,
    pub start_sample: u64,
    pub end_sample: u64,
}

#[derive(Debug, Clone)]
pub struct RecordingSession {
    pub session_id: String,
    pub session_dir: PathBuf,
    manifest_path: PathBuf,
}

impl RecordingSession {
    pub fn stable_session_id(seed: &str) -> String {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    pub fn open(base_dir: &Path, seed: &str) -> std::io::Result<Self> {
        let session_id = Self::stable_session_id(seed);
        let session_dir = base_dir.join(&session_id);
        fs::create_dir_all(&session_dir)?;
        let manifest_path = session_dir.join("manifest.jsonl");

        if !manifest_path.exists() {
            fs::File::create(&manifest_path)?;
        }

        Ok(Self {
            session_id,
            session_dir,
            manifest_path,
        })
    }

    pub fn append(&self, entry: &ManifestEntry) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.manifest_path)?;
        let line = serde_json::to_string(entry)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }

    pub fn finalize(&self) -> std::io::Result<()> {
        fs::metadata(&self.manifest_path)?;
        Ok(())
    }

    pub fn cancel(&self) -> std::io::Result<()> {
        match fs::remove_dir_all(&self.session_dir) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub fn recover(base_dir: &Path, session_id: &str) -> std::io::Result<Vec<ManifestEntry>> {
        let manifest_path = base_dir.join(session_id).join("manifest.jsonl");
        let file = fs::File::open(manifest_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: ManifestEntry = serde_json::from_str(&line)?;
            entries.push(entry);
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_session_folder_hashing_is_deterministic() {
        let a = RecordingSession::stable_session_id("seed-abc");
        let b = RecordingSession::stable_session_id("seed-abc");
        let c = RecordingSession::stable_session_id("seed-other");

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn manifest_is_append_only_jsonl() {
        let dir = tempfile::tempdir().expect("tempdir must be created");
        let session = RecordingSession::open(dir.path(), "append-seed").expect("session must open");

        session
            .append(&ManifestEntry {
                chunk_index: 0,
                filename: "chunk_0000.wav".to_string(),
                start_sample: 0,
                end_sample: 100,
            })
            .expect("first append must work");

        session
            .append(&ManifestEntry {
                chunk_index: 1,
                filename: "chunk_0001.wav".to_string(),
                start_sample: 80,
                end_sample: 180,
            })
            .expect("second append must work");

        let recovered =
            RecordingSession::recover(dir.path(), &session.session_id).expect("must recover");
        assert_eq!(recovered.len(), 2);
        assert_eq!(recovered[0].chunk_index, 0);
        assert_eq!(recovered[1].chunk_index, 1);
    }

    #[test]
    fn recovery_scan_reads_existing_manifest() {
        let dir = tempfile::tempdir().expect("tempdir must be created");
        let session =
            RecordingSession::open(dir.path(), "recover-seed").expect("session must open");
        session
            .append(&ManifestEntry {
                chunk_index: 7,
                filename: "chunk_0007.wav".to_string(),
                start_sample: 1_000,
                end_sample: 1_900,
            })
            .expect("append must work");
        session.finalize().expect("finalize must work");

        let recovered =
            RecordingSession::recover(dir.path(), &session.session_id).expect("must recover");
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].chunk_index, 7);
    }

    #[test]
    fn cancel_removes_session_directory() {
        let dir = tempfile::tempdir().expect("tempdir must be created");
        let session = RecordingSession::open(dir.path(), "cancel-seed").expect("session must open");

        let session_path = session.session_dir.clone();
        assert!(session_path.exists());

        session
            .cancel()
            .expect("cancel must remove session directory");
        assert!(!session_path.exists());
    }
}
