//! Audio device management module for MeetWhisperer.
//!
//! Provides functionality to enumerate available audio input and output devices
//! using PulseAudio command-line tools.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub id: String,
    pub is_input: bool,
}

pub fn get_input_devices() -> Vec<AudioDevice> {
    let mut devices = Vec::new();

    if let Ok(output) = std::process::Command::new("pactl")
        .args(["list", "short", "sources"])
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                // parts[0] = numeric index, parts[1] = PulseAudio source name
                let source_name = parts[1].to_string();

                if !source_name.contains(".monitor") {
                    devices.push(AudioDevice {
                        name: source_name.clone(),
                        id: source_name, // use the PA name, not the numeric index
                        is_input: true,
                    });
                }
            }
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
    let mut devices = Vec::new();

    if let Ok(output) = std::process::Command::new("pactl")
        .args(["list", "short", "sinks"])
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                // parts[1] = PulseAudio sink name; monitor = sink_name + ".monitor"
                let sink_name = parts[1].to_string();
                let monitor_name = format!("{}.monitor", sink_name);

                devices.push(AudioDevice {
                    name: sink_name.clone(),
                    id: monitor_name, // capture from the monitor source
                    is_input: false,
                });
            }
        }
    }

    if devices.is_empty() {
        devices.push(AudioDevice {
            name: "Default (PulseAudio)".to_string(),
            id: "default".to_string(),
            is_input: false,
        });
    }

    devices
}

pub fn get_default_input_device() -> Option<AudioDevice> {
    if let Ok(output) = std::process::Command::new("pactl")
        .args(["get-default-source"])
        .output()
    {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !name.is_empty() {
            return Some(AudioDevice {
                name: name.clone(),
                id: name,
                is_input: true,
            });
        }
    }
    None
}

pub fn get_default_output_device() -> Option<AudioDevice> {
    if let Ok(output) = std::process::Command::new("pactl")
        .args(["get-default-sink"])
        .output()
    {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !name.is_empty() {
            return Some(AudioDevice {
                name: name.clone(),
                id: name,
                is_input: false,
            });
        }
    }
    None
}

/// Returns the directory where Whisper models are stored.
///
/// Search order:
/// 1. `models/` relative to the current working directory (dev/local mode).
/// 2. `models/` next to the running executable (installed mode, e.g. /opt/meet-whisperer/models/).
/// Returns the first directory that exists and contains at least one `.bin` file.
fn find_models_dir() -> Option<std::path::PathBuf> {
    let candidates: Vec<std::path::PathBuf> = {
        let mut v = vec![std::path::PathBuf::from("models")];
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
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()).map(str::to_owned) {
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
        models.push(("ggml-tiny.bin".to_string(), fallback.to_string_lossy().into_owned()));
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
    /// Whether to use Ollama to post-process the transcript.
    pub ollama_enabled: bool,
    /// Which Ollama model to use for post-processing.
    pub ollama_model: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        let recordings_folder = std::env::var("HOME")
            .map(|h| format!("{}/MeetWhisperer/recordings", h))
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
            ollama_enabled: false,
            ollama_model: String::new(),
        }
    }
}

impl AppSettings {
    pub fn load() -> Self {
        let config_path = Self::config_path();
        let mut settings = if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(s) = toml::from_str::<Self>(&content) {
                    s
                } else {
                    Self::default()
                }
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
            .map(|h| format!("{}/.config/meet-whisperer/settings.toml", h))
            .unwrap_or_else(|_| "settings.toml".to_string())
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_devices() {
        let input_devices = get_input_devices();
        let output_devices = get_output_devices();

        println!("Input devices: {:?}", input_devices.len());
        println!("Output devices: {:?}", output_devices.len());

        for device in &input_devices {
            println!("  Input: {} ({})", device.name, device.id);
        }

        for device in &output_devices {
            println!("  Output: {} ({})", device.name, device.id);
        }
    }
}
