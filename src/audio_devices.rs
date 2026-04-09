//! Audio device management module for Scrivano.
//!
//! Provides functionality to enumerate available audio input and output devices
//! using PulseAudio command-line tools.

use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

static PACTL_FAILURE_LOGGED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub id: String,
    pub is_input: bool,
}

fn pactl_candidates() -> Vec<String> {
    let mut candidates = Vec::new();

    if let Ok(custom) = std::env::var("SCRIVANO_PACTL_PATH") {
        if !custom.trim().is_empty() {
            candidates.push(custom);
        }
    }

    candidates.push("pactl".to_string());

    if let Ok(snap_dir) = std::env::var("SNAP") {
        candidates.push(format!("{}/usr/bin/pactl", snap_dir));
    }

    candidates.push("/usr/bin/pactl".to_string());

    candidates
}

fn run_pactl(args: &[&str]) -> Option<String> {
    let mut errors = Vec::new();

    for candidate in pactl_candidates() {
        let output = Command::new(&candidate).args(args).output();
        match output {
            Ok(out) if out.status.success() => {
                return Some(String::from_utf8_lossy(&out.stdout).to_string());
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                errors.push(format!(
                    "candidate='{}' status={} stderr='{}'",
                    candidate, out.status, stderr
                ));
            }
            Err(e) => {
                errors.push(format!("candidate='{}' error='{}'", candidate, e));
            }
        }
    }

    if !errors.is_empty() && !PACTL_FAILURE_LOGGED.swap(true, Ordering::Relaxed) {
        eprintln!(
            "[audio_devices] pactl command failed for all candidates: {}",
            errors.join(" | ")
        );
    }

    None
}

fn parse_short_list_name(line: &str) -> Option<&str> {
    let mut tab_parts = line.split('\t');
    if let (Some(_idx), Some(name)) = (tab_parts.next(), tab_parts.next()) {
        let name = name.trim();
        if !name.is_empty() {
            return Some(name);
        }
    }

    let mut ws_parts = line.split_whitespace();
    let _idx = ws_parts.next()?;
    let name = ws_parts.next()?.trim();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn parse_input_devices(output: &str) -> Vec<AudioDevice> {
    output
        .lines()
        .filter_map(parse_short_list_name)
        .filter(|name| !name.contains(".monitor"))
        .map(|source_name| AudioDevice {
            name: source_name.to_string(),
            id: source_name.to_string(),
            is_input: true,
        })
        .collect()
}

fn parse_output_devices(output: &str) -> Vec<AudioDevice> {
    output
        .lines()
        .filter_map(parse_short_list_name)
        .map(|sink_name| AudioDevice {
            name: sink_name.to_string(),
            id: format!("{}.monitor", sink_name),
            is_input: false,
        })
        .collect()
}

fn parse_default_from_info(output: &str, key: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let (k, v) = line.split_once(':')?;
        if k.trim() == key {
            let value = v.trim();
            if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            }
        } else {
            None
        }
    })
}

fn default_source_name() -> Option<String> {
    let from_direct = run_pactl(&["get-default-source"])
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    if from_direct.is_some() {
        return from_direct;
    }

    run_pactl(&["info"]).and_then(|v| parse_default_from_info(&v, "Default Source"))
}

fn default_sink_name() -> Option<String> {
    let from_direct = run_pactl(&["get-default-sink"])
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    if from_direct.is_some() {
        return from_direct;
    }

    run_pactl(&["info"]).and_then(|v| parse_default_from_info(&v, "Default Sink"))
}

pub fn get_input_devices() -> Vec<AudioDevice> {
    let mut devices = run_pactl(&["list", "short", "sources"])
        .map(|v| parse_input_devices(&v))
        .unwrap_or_default();

    if devices.is_empty() {
        devices = run_pactl(&["list", "sources", "short"])
            .map(|v| parse_input_devices(&v))
            .unwrap_or_default();
    }

    if devices.is_empty() {
        if let Some(default_source) = default_source_name() {
            devices.push(AudioDevice {
                name: default_source.clone(),
                id: default_source,
                is_input: true,
            });
        }
    }

    if devices.is_empty() {
        devices.push(AudioDevice {
            name: "Default (PulseAudio)".to_string(),
            id: "default".to_string(),
            is_input: true,
        });
    }

    devices
}

pub fn get_output_devices() -> Vec<AudioDevice> {
    let mut devices = run_pactl(&["list", "short", "sinks"])
        .map(|v| parse_output_devices(&v))
        .unwrap_or_default();

    if devices.is_empty() {
        devices = run_pactl(&["list", "sinks", "short"])
            .map(|v| parse_output_devices(&v))
            .unwrap_or_default();
    }

    if devices.is_empty() {
        if let Some(default_sink) = default_sink_name() {
            devices.push(AudioDevice {
                name: default_sink.clone(),
                id: format!("{}.monitor", default_sink),
                is_input: false,
            });
        }
    }

    if devices.is_empty() {
        devices.push(AudioDevice {
            name: "Default (PulseAudio)".to_string(),
            id: "default.monitor".to_string(),
            is_input: false,
        });
    }

    devices
}

pub fn get_default_input_device() -> Option<AudioDevice> {
    default_source_name().map(|name| AudioDevice {
        name: name.clone(),
        id: name,
        is_input: true,
    })
}

pub fn get_default_output_device() -> Option<AudioDevice> {
    default_sink_name().map(|name| AudioDevice {
        name: name.clone(),
        id: format!("{}.monitor", name),
        is_input: false,
    })
}

/// Returns the directory where Whisper models are stored.
///
/// Search order:
/// 1. `$SNAP_USER_DATA/models` (snap runtime, user-writable).
/// 2. `$SNAP/models` (bundled models inside snap).
/// 3. `models/` relative to the current working directory (dev/local mode).
/// 4. `models/` next to the running executable (installed mode, e.g. /opt/Scrivano/models/).
/// Returns the first directory that exists and contains at least one `.bin` file.
fn find_models_dir() -> Option<std::path::PathBuf> {
    let candidates: Vec<std::path::PathBuf> = {
        let mut v = Vec::new();

        if let Ok(snap_user_data) = std::env::var("SNAP_USER_DATA") {
            v.push(std::path::PathBuf::from(snap_user_data).join("models"));
        }

        if let Ok(snap_dir) = std::env::var("SNAP") {
            v.push(std::path::PathBuf::from(snap_dir).join("models"));
        }

        v.push(std::path::PathBuf::from("models"));
        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                v.push(exe_dir.join("models"));
            }
        }
        v
    };

    for dir in candidates {
        if dir.is_dir() {
            let has_bin = std::fs::read_dir(&dir)
                .ok()
                .map(|mut e| {
                    e.any(|f| {
                        f.ok()
                            .and_then(|f| f.path().extension().map(|e| e == "bin"))
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);
            if has_bin {
                return Some(dir);
            }
        }
    }
    None
}

/// Scan the models directory and return all `.bin` files as
/// `(display_name, absolute_path)` pairs, sorted alphabetically.
pub fn scan_models() -> Vec<(String, String)> {
    let mut models = Vec::new();

    if let Some(models_dir) = find_models_dir() {
        if let Ok(entries) = std::fs::read_dir(&models_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("bin") {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()).map(str::to_owned)
                    {
                        // Use canonical absolute path so it works regardless of cwd
                        let abs = path.canonicalize().unwrap_or_else(|_| path.clone());
                        models.push((name, abs.to_string_lossy().into_owned()));
                    }
                }
            }
        }
    }

    models.sort_by(|a, b| a.0.cmp(&b.0));

    if models.is_empty() {
        // Fallback: use absolute path next to exe if possible
        let fallback = std::env::current_exe()
            .ok()
            .and_then(|e| e.parent().map(|d| d.join("models/ggml-tiny.bin")))
            .unwrap_or_else(|| std::path::PathBuf::from("models/ggml-tiny.bin"));
        models.push((
            "ggml-tiny.bin".to_string(),
            fallback.to_string_lossy().into_owned(),
        ));
    }

    models
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub input_device_id: Option<String>,
    pub output_device_id: Option<String>,
    pub recordings_folder: String,
    /// Path to the Whisper GGML model file to use for transcription.
    pub whisper_model: String,
    /// Use GPU for Whisper transcription
    pub whisper_use_gpu: bool,
    /// Whether to use Ollama to post-process the transcript.
    pub ollama_enabled: bool,
    /// Which Ollama model to use for post-processing.
    pub ollama_model: String,
    /// Ollama host (default: localhost)
    pub ollama_host: String,
    /// Ollama port (default: 11434)
    pub ollama_port: u16,
    /// Use Ollama for STT (alternative to Whisper)
    pub use_ollama_for_stt: bool,
    /// Summary model for Ollama (e.g., llama3.2, gemma4:e2b)
    pub summary_model: String,
    /// Streaming mode: auto, stream, non_stream
    pub summary_stream_mode: String,
    /// Thinking policy: hide_thinking, store_but_hide, show_for_debug
    pub summary_thinking_policy: String,
    /// Default language: es, en
    pub language_default: String,
    /// Hotkey for start/stop recording
    pub hotkey_start_stop: String,
    /// Hotkey for highlight
    pub hotkey_highlight: String,
    /// Custom prompt for Ollama corrections (e.g., correct technical terms)
    pub prompt_correction: String,
    /// Custom prompt for transcript improvement
    pub prompt_transcript: String,
    /// Custom prompt for executive summary
    pub custom_prompt_executive: String,
    /// Custom prompt for tasks summary
    pub custom_prompt_tasks: String,
    /// Custom prompt for decisions summary
    pub custom_prompt_decisions: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        let recordings_folder = std::env::var("HOME")
            .map(|h| format!("{}/Scrivano/recordings", h))
            .unwrap_or_else(|_| "recordings".to_string());

        // Use the first available model found via the search path
        let whisper_model = scan_models()
            .into_iter()
            .next()
            .map(|(_, path)| path)
            .unwrap_or_else(|| "models/ggml-tiny.bin".to_string());

        Self {
            input_device_id: None,
            output_device_id: None,
            recordings_folder,
            whisper_model,
            whisper_use_gpu: true,
            ollama_enabled: false,
            ollama_model: String::new(),
            ollama_host: "localhost".to_string(),
            ollama_port: 11434,
            use_ollama_for_stt: false,
            summary_model: "llama3.2".to_string(),
            summary_stream_mode: "auto".to_string(),
            summary_thinking_policy: "hide_thinking".to_string(),
            language_default: "es".to_string(),
            hotkey_start_stop: "Ctrl+Shift+R".to_string(),
            hotkey_highlight: "Ctrl+Shift+H".to_string(),
            prompt_correction: String::new(),
            prompt_transcript: String::new(),
            custom_prompt_executive: String::new(),
            custom_prompt_tasks: String::new(),
            custom_prompt_decisions: String::new(),
        }
    }
}

impl AppSettings {
    pub fn load() -> Self {
        let config_path = Self::config_path();
        let mut settings = if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                toml::from_str::<Self>(&content).unwrap_or_default()
            } else {
                Self::default()
            }
        } else {
            Self::default()
        };

        // If the saved model path no longer exists (e.g. after reinstall or
        // moving the binary), find a valid model from the current search paths.
        if !std::path::Path::new(&settings.whisper_model).exists() {
            let available = scan_models();
            if let Some((_, path)) = available.into_iter().next() {
                eprintln!(
                    "[settings] modelo '{}' no encontrado, usando '{}'",
                    settings.whisper_model, path
                );
                settings.whisper_model = path;
            }
        }

        settings
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }

    fn config_path() -> std::path::PathBuf {
        std::env::var("HOME")
            .map(|h| format!("{}/.config/Scrivano/settings.toml", h))
            .unwrap_or_else(|_| "settings.toml".to_string())
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_input_devices_filters_monitors() {
        let output = "0\talsa_input.usb-Mic-00.analog-stereo\tmodule-alsa-card.c\t...\n1\talsa_output.pci-0000_00_1f.3.analog-stereo.monitor\tmodule-alsa-card.c\t...\n";
        let devices = parse_input_devices(output);

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "alsa_input.usb-Mic-00.analog-stereo");
        assert_eq!(devices[0].id, "alsa_input.usb-Mic-00.analog-stereo");
        assert!(devices[0].is_input);
    }

    #[test]
    fn parse_output_devices_builds_monitor_ids() {
        let output = "0\talsa_output.pci-0000_00_1f.3.analog-stereo\tmodule-alsa-card.c\t...\n1\tbluez_output.XX_XX_XX_XX_XX_XX.a2dp-sink\tmodule-bluez5-device.c\t...\n";
        let devices = parse_output_devices(output);

        assert_eq!(devices.len(), 2);
        assert_eq!(
            devices[0].id,
            "alsa_output.pci-0000_00_1f.3.analog-stereo.monitor"
        );
        assert_eq!(
            devices[1].id,
            "bluez_output.XX_XX_XX_XX_XX_XX.a2dp-sink.monitor"
        );
        assert!(!devices[0].is_input);
    }

    #[test]
    fn parse_short_list_name_supports_whitespace_format() {
        let line = "0 alsa_input.usb-Mic-00.analog-stereo module-alsa-card.c s16le 2ch 48000Hz";
        assert_eq!(
            parse_short_list_name(line),
            Some("alsa_input.usb-Mic-00.analog-stereo")
        );
    }

    #[test]
    fn parse_default_from_info_extracts_values() {
        let info = "Server String: /run/user/1000/pulse/native\nDefault Sink: alsa_output.pci-0000_00_1f.3.analog-stereo\nDefault Source: alsa_input.usb-Mic-00.analog-stereo\n";

        assert_eq!(
            parse_default_from_info(info, "Default Sink").as_deref(),
            Some("alsa_output.pci-0000_00_1f.3.analog-stereo")
        );
        assert_eq!(
            parse_default_from_info(info, "Default Source").as_deref(),
            Some("alsa_input.usb-Mic-00.analog-stereo")
        );
    }
}
