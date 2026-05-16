use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerfSnapshot {
    pub session_minutes: u32,
    pub post_stop_wait_ms: u64,
    pub peak_active_chunk_bytes: usize,
    pub peak_overlap_bytes: usize,
    pub peak_waveform_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeterministicAssumptions {
    pub chunk_seconds: u32,
    pub overlap_seconds: u32,
    pub post_stop_workers: u32,
    pub baseline_rtf: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SurrogateReduction {
    pub session_minutes: u32,
    pub baseline_post_stop_wait_ms: u64,
    pub chunked_post_stop_wait_ms: u64,
    pub reduction_pct: f64,
}

pub fn baseline_report_path(base_dir: &Path, minutes: u32) -> PathBuf {
    base_dir.join(format!("{}min-baseline.json", minutes))
}

pub fn write_baseline_reports(
    base_dir: &Path,
    snapshots: &[PerfSnapshot],
) -> std::io::Result<Vec<PathBuf>> {
    fs::create_dir_all(base_dir)?;
    let mut paths = Vec::with_capacity(snapshots.len());

    for snapshot in snapshots {
        let path = baseline_report_path(base_dir, snapshot.session_minutes);
        let payload = serde_json::to_string_pretty(snapshot)?;
        fs::write(&path, payload)?;
        paths.push(path);
    }

    Ok(paths)
}

pub fn deterministic_surrogate(
    session_minutes: u32,
    a: &DeterministicAssumptions,
) -> SurrogateReduction {
    let session_sec = (session_minutes as f64) * 60.0;
    let baseline_ms = (session_sec * a.baseline_rtf * 1000.0).round() as u64;

    // Conservative pipeline surrogate: with chunked transcription running during capture,
    // only the final active chunk (+ overlap) remains at stop.
    let backlog_sec = (a.chunk_seconds + a.overlap_seconds) as f64;
    let workers = a.post_stop_workers.max(1) as f64;
    let chunked_ms = (backlog_sec * a.baseline_rtf * 1000.0 / workers).round() as u64;

    let reduction = if baseline_ms == 0 {
        0.0
    } else {
        ((baseline_ms as f64 - chunked_ms as f64) / baseline_ms as f64) * 100.0
    };

    SurrogateReduction {
        session_minutes,
        baseline_post_stop_wait_ms: baseline_ms,
        chunked_post_stop_wait_ms: chunked_ms,
        reduction_pct: reduction,
    }
}

pub fn surrogate_report_path(base_dir: &Path) -> PathBuf {
    base_dir.join("surrogate-reduction.json")
}

pub fn write_surrogate_report(
    base_dir: &Path,
    assumptions: &DeterministicAssumptions,
    rows: &[SurrogateReduction],
) -> std::io::Result<PathBuf> {
    fs::create_dir_all(base_dir)?;
    let path = surrogate_report_path(base_dir);
    let payload = serde_json::json!({
        "kind": "deterministic-surrogate",
        "assumptions": assumptions,
        "rows": rows,
    });
    fs::write(&path, serde_json::to_string_pretty(&payload)?)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_paths_are_under_target_perf_baseline() {
        let base = Path::new("target/perf-baseline");
        let path_10 = baseline_report_path(base, 10);
        let path_30 = baseline_report_path(base, 30);
        let path_60 = baseline_report_path(base, 60);

        assert!(path_10.starts_with(base));
        assert!(path_30.starts_with(base));
        assert!(path_60.starts_with(base));
        assert!(path_10.ends_with("10min-baseline.json"));
        assert!(path_30.ends_with("30min-baseline.json"));
        assert!(path_60.ends_with("60min-baseline.json"));
    }

    #[test]
    fn snapshot_contains_peak_ram_surrogate_fields() {
        let snapshot = PerfSnapshot {
            session_minutes: 30,
            post_stop_wait_ms: 12_345,
            peak_active_chunk_bytes: 128_000,
            peak_overlap_bytes: 24_000,
            peak_waveform_bytes: 8_192,
        };

        assert_eq!(snapshot.session_minutes, 30);
        assert_eq!(snapshot.peak_active_chunk_bytes, 128_000);
        assert_eq!(snapshot.peak_overlap_bytes, 24_000);
        assert_eq!(snapshot.peak_waveform_bytes, 8_192);
    }

    #[test]
    fn write_baselines_for_10_30_60_minutes() {
        let dir = tempfile::tempdir().expect("tempdir must be created");
        let base = dir.path().join("target/perf-baseline");
        let snapshots = vec![
            PerfSnapshot {
                session_minutes: 10,
                post_stop_wait_ms: 1000,
                peak_active_chunk_bytes: 100,
                peak_overlap_bytes: 50,
                peak_waveform_bytes: 25,
            },
            PerfSnapshot {
                session_minutes: 30,
                post_stop_wait_ms: 3000,
                peak_active_chunk_bytes: 200,
                peak_overlap_bytes: 75,
                peak_waveform_bytes: 30,
            },
            PerfSnapshot {
                session_minutes: 60,
                post_stop_wait_ms: 6000,
                peak_active_chunk_bytes: 300,
                peak_overlap_bytes: 100,
                peak_waveform_bytes: 35,
            },
        ];

        let written = write_baseline_reports(&base, &snapshots).expect("reports must be written");
        assert_eq!(written.len(), 3);
        assert!(written.iter().all(|p| p.exists()));
    }

    #[test]
    fn deterministic_surrogate_meets_40_percent_for_60_minutes() {
        let assumptions = DeterministicAssumptions {
            chunk_seconds: 25,
            overlap_seconds: 5,
            post_stop_workers: 1,
            baseline_rtf: 1.0,
        };

        let result_60 = deterministic_surrogate(60, &assumptions);

        assert_eq!(result_60.session_minutes, 60);
        assert!(result_60.reduction_pct >= 40.0);
        assert!(result_60.chunked_post_stop_wait_ms < result_60.baseline_post_stop_wait_ms);
    }

    #[test]
    fn writes_surrogate_report_for_10_30_60() {
        let dir = tempfile::tempdir().expect("tempdir must be created");
        let base = dir.path().join("target/perf-baseline");
        let assumptions = DeterministicAssumptions {
            chunk_seconds: 25,
            overlap_seconds: 5,
            post_stop_workers: 1,
            baseline_rtf: 1.0,
        };
        let rows = vec![
            deterministic_surrogate(10, &assumptions),
            deterministic_surrogate(30, &assumptions),
            deterministic_surrogate(60, &assumptions),
        ];

        let report = write_surrogate_report(&base, &assumptions, &rows)
            .expect("surrogate report should be written");

        assert!(report.exists());
        assert!(report.ends_with("surrogate-reduction.json"));
    }
}
