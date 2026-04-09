#[cfg(feature = "audio-playback")]
mod rodio_backend {
    use anyhow::{Context, Result};
    use rodio::{Decoder, OutputStream, Sink};
    use std::fs::File;
    use std::io::BufReader;
    use std::process::{Child, Command};
    use std::time::Instant;

    fn find_pulse_player() -> Option<String> {
        ["paplay", "pacat"].iter().find_map(|bin| {
            let output = Command::new(bin).arg("--version").output().ok()?;
            if output.status.success() {
                Some((*bin).to_string())
            } else {
                None
            }
        })
    }

    enum PlaybackBackend {
        Rodio { _stream: OutputStream, sink: Sink },
        PulseCli { program: String },
    }

    pub struct AudioPlayer {
        backend: PlaybackBackend,
        paplay_child: Option<Child>,
        pub current_file: Option<String>,
        start_time: Instant,
        paused_elapsed_secs: f64,
        duration_secs: f64,
        last_pause_time: Option<Instant>,
    }

    impl AudioPlayer {
        pub fn new() -> Option<Self> {
            let (stream, stream_handle) = match OutputStream::try_default() {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("[playback] Failed to open default output stream: {e}");
                    if let Some(program) = find_pulse_player() {
                        eprintln!("[playback] Falling back to {program} backend");
                        return Some(Self {
                            backend: PlaybackBackend::PulseCli { program },
                            paplay_child: None,
                            current_file: None,
                            start_time: Instant::now(),
                            paused_elapsed_secs: 0.0,
                            duration_secs: 0.0,
                            last_pause_time: None,
                        });
                    }
                    return None;
                }
            };

            let sink = match Sink::try_new(&stream_handle) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("[playback] Failed to create sink: {e}");
                    if let Some(program) = find_pulse_player() {
                        eprintln!("[playback] Falling back to {program} backend");
                        return Some(Self {
                            backend: PlaybackBackend::PulseCli { program },
                            paplay_child: None,
                            current_file: None,
                            start_time: Instant::now(),
                            paused_elapsed_secs: 0.0,
                            duration_secs: 0.0,
                            last_pause_time: None,
                        });
                    }
                    return None;
                }
            };

            Some(Self {
                backend: PlaybackBackend::Rodio {
                    _stream: stream,
                    sink,
                },
                paplay_child: None,
                current_file: None,
                start_time: Instant::now(),
                paused_elapsed_secs: 0.0,
                duration_secs: 0.0,
                last_pause_time: None,
            })
        }

        pub fn play(&mut self, path: &str) -> Result<()> {
            self.stop();

            match &mut self.backend {
                PlaybackBackend::Rodio { sink, .. } => {
                    let file = File::open(path).context("Failed to open audio file")?;
                    let reader = BufReader::new(file);
                    let source = Decoder::new(reader).context("Failed to decode audio file")?;
                    sink.append(source);
                }
                PlaybackBackend::PulseCli { program } => {
                    let program_name = program.clone();
                    let child = Command::new(program_name.as_str())
                        .arg(path)
                        .spawn()
                        .with_context(|| format!("Failed to spawn {program_name}"))?;
                    self.paplay_child = Some(child);
                }
            }

            self.current_file = Some(path.to_string());
            self.start_time = Instant::now();
            self.paused_elapsed_secs = 0.0;
            self.last_pause_time = None;
            self.duration_secs = 0.0;

            eprintln!("[playback] Playing file: {path}");

            Ok(())
        }

        pub fn pause(&mut self) {
            if let PlaybackBackend::Rodio { sink, .. } = &mut self.backend {
                if !sink.is_paused() {
                    sink.pause();
                    self.last_pause_time = Some(Instant::now());
                }
            }
        }

        pub fn resume(&mut self) {
            if let PlaybackBackend::Rodio { sink, .. } = &mut self.backend {
                if sink.is_paused() {
                    sink.play();
                    if let Some(pause_time) = self.last_pause_time {
                        self.paused_elapsed_secs += pause_time.elapsed().as_secs_f64();
                        self.last_pause_time = None;
                    }
                }
            }
        }

        pub fn stop(&mut self) {
            match &mut self.backend {
                PlaybackBackend::Rodio { sink, .. } => sink.stop(),
                PlaybackBackend::PulseCli { .. } => {
                    if let Some(child) = &mut self.paplay_child {
                        let _ = child.kill();
                        let _ = child.wait();
                    }
                    self.paplay_child = None;
                }
            }
            self.paused_elapsed_secs = 0.0;
            self.last_pause_time = None;
        }

        pub fn is_playing(&self) -> bool {
            match &self.backend {
                PlaybackBackend::Rodio { sink, .. } => !sink.is_paused() && !sink.empty(),
                PlaybackBackend::PulseCli { .. } => self.paplay_child.is_some(),
            }
        }

        pub fn is_paused(&self) -> bool {
            match &self.backend {
                PlaybackBackend::Rodio { sink, .. } => sink.is_paused(),
                PlaybackBackend::PulseCli { .. } => false,
            }
        }

        pub fn set_volume(&self, volume: f32) {
            if let PlaybackBackend::Rodio { sink, .. } = &self.backend {
                sink.set_volume(volume.clamp(0.0, 1.0));
            }
        }

        pub fn get_volume(&self) -> f32 {
            match &self.backend {
                PlaybackBackend::Rodio { sink, .. } => sink.volume(),
                PlaybackBackend::PulseCli { .. } => 0.8,
            }
        }

        pub fn is_stopped(&self) -> bool {
            match &self.backend {
                PlaybackBackend::Rodio { sink, .. } => sink.empty(),
                PlaybackBackend::PulseCli { .. } => self.paplay_child.is_none(),
            }
        }

        pub fn get_elapsed_secs(&self) -> f64 {
            if self.is_stopped() {
                return 0.0;
            }

            if self.is_paused() {
                if let Some(pause_time) = self.last_pause_time {
                    return self.paused_elapsed_secs + pause_time.elapsed().as_secs_f64();
                }
                return self.paused_elapsed_secs;
            }

            self.paused_elapsed_secs + self.start_time.elapsed().as_secs_f64()
        }

        pub fn get_duration_secs(&self) -> f64 {
            self.duration_secs
        }
    }

    impl Default for AudioPlayer {
        fn default() -> Self {
            Self::new().expect("Failed to create audio player")
        }
    }
}

#[cfg(feature = "audio-playback")]
pub use rodio_backend::AudioPlayer;

#[cfg(not(feature = "audio-playback"))]
pub struct AudioPlayer;

#[cfg(not(feature = "audio-playback"))]
impl AudioPlayer {
    pub fn new() -> Option<Self> {
        None
    }

    pub fn play(&self, _path: &str) -> Result<(), String> {
        Err("Audio playback not available (compile with audio-playback feature)".to_string())
    }

    pub fn pause(&mut self) {}
    pub fn resume(&mut self) {}
    pub fn stop(&mut self) {}
    pub fn is_playing(&self) -> bool {
        false
    }
    pub fn is_paused(&self) -> bool {
        false
    }
    pub fn set_volume(&self, _volume: f32) {}
    pub fn get_volume(&self) -> f32 {
        0.8
    }
    pub fn is_stopped(&self) -> bool {
        true
    }
    pub fn get_elapsed_secs(&self) -> f64 {
        0.0
    }
    pub fn get_duration_secs(&self) -> f64 {
        0.0
    }
}
