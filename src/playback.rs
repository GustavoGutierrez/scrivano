#[cfg(feature = "audio-playback")]
mod rodio_backend {
    use anyhow::{Context, Result};
    use rodio::{Decoder, OutputStream, Sink};
    use std::fs::File;
    use std::io::BufReader;
    use std::time::Instant;

    pub struct AudioPlayer {
        _stream: OutputStream,
        sink: Sink,
        pub current_file: Option<String>,
        start_time: Instant,
        duration_secs: f64,
    }

    impl AudioPlayer {
        pub fn new() -> Option<Self> {
            let (stream, stream_handle) = OutputStream::try_default().ok()?;

            let sink = Sink::try_new(&stream_handle).ok()?;

            Some(Self {
                _stream: stream,
                sink,
                current_file: None,
                start_time: Instant::now(),
                duration_secs: 0.0,
            })
        }

        pub fn play(&mut self, path: &str) -> Result<()> {
            let file = File::open(path).context("Failed to open audio file")?;
            let reader = BufReader::new(file);

            let source = Decoder::new(reader).context("Failed to decode audio file")?;

            self.sink.append(source);
            self.current_file = Some(path.to_string());
            self.start_time = Instant::now();
            self.duration_secs = 0.0;

            Ok(())
        }

        pub fn pause(&self) {
            self.sink.pause();
        }

        pub fn resume(&self) {
            self.sink.play();
        }

        pub fn stop(&self) {
            self.sink.stop();
        }

        pub fn is_playing(&self) -> bool {
            !self.sink.is_paused() && !self.sink.empty()
        }

        pub fn is_paused(&self) -> bool {
            self.sink.is_paused()
        }

        pub fn set_volume(&self, volume: f32) {
            self.sink.set_volume(volume.clamp(0.0, 1.0));
        }

        pub fn get_volume(&self) -> f32 {
            self.sink.volume()
        }

        pub fn is_stopped(&self) -> bool {
            self.sink.empty()
        }

        pub fn get_elapsed_secs(&self) -> f64 {
            self.start_time.elapsed().as_secs_f64()
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

    pub fn pause(&self) {}
    pub fn resume(&self) {}
    pub fn stop(&self) {}
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
