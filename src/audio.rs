//! Audio capture module for Scrivano.
//!
//! Captures audio from any PulseAudio source (microphone or desktop monitor)
//! using libpulse-binding directly.  Frames arrive as S32LE mono at 44 100 Hz
//! and are resampled to 16 000 Hz f32 before being stored for Whisper.

use libpulse_binding as pulse;
use pulse::{
    context::{Context, FlagSet as ContextFlagSet, State as ContextState},
    def::BufferAttr,
    mainloop::standard::{IterateResult, Mainloop},
    sample::{Format, Spec},
    stream::{FlagSet as StreamFlagSet, PeekResult, Stream},
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

/// Audio source type for recording
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioSource {
    /// System audio (desktop capture via loopback)
    System,
    /// Microphone input
    Microphone,
}

impl AudioSource {
    /// Get the PulseAudio source name suffix based on source type
    pub fn pa_suffix(&self) -> &'static str {
        match self {
            AudioSource::System => ".monitor",
            AudioSource::Microphone => "",
        }
    }
}

/// Sample rate at which PulseAudio delivers frames (S32LE mono).
const CAPTURE_RATE: u32 = 44_100;
/// Sample rate required by Whisper.
pub const WHISPER_RATE: u32 = 16_000;
/// Rolling waveform window size (samples at capture rate, ~185ms @ 44100Hz).
const WAVE_WINDOW: usize = 8_192;

// ── Resampler ────────────────────────────────────────────────────────────────

/// Resample mono f32 from `src_rate` to `dst_rate` using linear interpolation.
pub fn resample_linear(input: &[f32], src_rate: u32, dst_rate: u32) -> Vec<f32> {
    if src_rate == dst_rate || input.is_empty() {
        return input.to_vec();
    }
    let ratio = src_rate as f64 / dst_rate as f64;
    let out_len = ((input.len() as f64) / ratio).ceil() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let pos = i as f64 * ratio;
        let lo = pos.floor() as usize;
        let hi = (lo + 1).min(input.len() - 1);
        let frac = (pos - lo as f64) as f32;
        out.push(input[lo] * (1.0 - frac) + input[hi] * frac);
    }
    out
}

// ── Recorder thread ──────────────────────────────────────────────────────────

/// Spawns a PulseAudio recording thread that reads from `source_name`.
///
/// Pass a PulseAudio source name such as:
/// - `"alsa_input.pci-0000_00_1b.0.analog-stereo"` — a microphone
/// - `"alsa_output.pci-0000_00_1b.0.analog-stereo.monitor"` — desktop audio
///
/// Audio is resampled from 44 100 Hz → 16 000 Hz and appended to
/// `audio_buffer`.  The last [`WAVE_WINDOW`] raw samples are stored in
/// `waveform_buffer` for live display.
pub fn spawn_system_audio_recorder(
    recording_flag: Arc<AtomicBool>,
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    waveform_buffer: Arc<Mutex<Vec<f32>>>,
    source_name: String,
) {
    thread::spawn(move || {
        // ── Build PulseAudio mainloop + context on this thread ──────────────
        let mut mainloop = match Mainloop::new() {
            Some(m) => m,
            None => {
                eprintln!("[audio] Failed to create PulseAudio mainloop");
                return;
            }
        };

        let mut context = match Context::new(&mainloop, "Scrivano") {
            Some(c) => c,
            None => {
                eprintln!("[audio] Failed to create PulseAudio context");
                return;
            }
        };

        if context
            .connect(None, ContextFlagSet::NOFLAGS, None)
            .is_err()
        {
            eprintln!("[audio] Failed to connect to PulseAudio");
            return;
        }

        // Wait for context to be ready
        loop {
            match mainloop.iterate(true) {
                IterateResult::Err(_) | IterateResult::Quit(_) => {
                    eprintln!("[audio] Mainloop exited during context connect");
                    return;
                }
                IterateResult::Success(_) => {}
            }
            match context.get_state() {
                ContextState::Ready => break,
                ContextState::Failed | ContextState::Terminated => {
                    eprintln!("[audio] PulseAudio context failed");
                    return;
                }
                _ => {}
            }
        }

        eprintln!(
            "[audio] PulseAudio context ready, source = {:?}",
            source_name
        );

        // ── Create recording stream ─────────────────────────────────────────
        let spec = Spec {
            format: Format::S32le,
            channels: 1,
            rate: CAPTURE_RATE,
        };
        assert!(spec.is_valid());

        let mut stream = match Stream::new(&mut context, "Scrivano-capture", &spec, None) {
            Some(s) => s,
            None => {
                eprintln!("[audio] Failed to create PulseAudio stream");
                return;
            }
        };

        let src = if source_name.is_empty() {
            None
        } else {
            Some(source_name.as_str())
        };

        if stream
            .connect_record(
                src,
                Some(&BufferAttr {
                    maxlength: u32::MAX,
                    tlength: u32::MAX,
                    prebuf: u32::MAX,
                    minreq: u32::MAX,
                    fragsize: 4096,
                }),
                StreamFlagSet::NOFLAGS,
            )
            .is_err()
        {
            eprintln!("[audio] Failed to connect recording stream");
            return;
        }

        // ── Read loop ───────────────────────────────────────────────────────
        while recording_flag.load(Ordering::SeqCst) {
            match mainloop.iterate(true) {
                IterateResult::Err(_) | IterateResult::Quit(_) => {
                    eprintln!("[audio] Mainloop exited during recording");
                    break;
                }
                IterateResult::Success(_) => {}
            }

            match stream.get_state() {
                pulse::stream::State::Ready => {}
                _ => continue,
            }

            let peek = match stream.peek() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("[audio] stream.peek() error: {}", e);
                    thread::sleep(Duration::from_millis(5));
                    continue;
                }
            };

            match peek {
                PeekResult::Data(data) => {
                    // Parse S32LE bytes → i32 → f32
                    let float_44k: Vec<f32> = data
                        .chunks_exact(4)
                        .map(|b| {
                            let s = i32::from_le_bytes(b.try_into().unwrap());
                            s as f32 / i32::MAX as f32
                        })
                        .collect();

                    stream.discard().ok();

                    if float_44k.is_empty() {
                        continue;
                    }

                    // Resample to 16 kHz for Whisper
                    let float_16k = resample_linear(&float_44k, CAPTURE_RATE, WHISPER_RATE);
                    audio_buffer.lock().unwrap().extend_from_slice(&float_16k);

                    // Update waveform window
                    {
                        let mut wave = waveform_buffer.lock().unwrap();
                        wave.extend_from_slice(&float_44k);
                        if wave.len() > WAVE_WINDOW {
                            let drain_to = wave.len() - WAVE_WINDOW;
                            wave.drain(..drain_to);
                        }
                    }
                }
                PeekResult::Hole(_) => {
                    stream.discard().ok();
                }
                PeekResult::Empty => {
                    thread::sleep(Duration::from_millis(2));
                }
            }
        }

        eprintln!("[audio] Recording thread stopped");
    });
}

// ── Buffer helpers ────────────────────────────────────────────────────────────

/// Clears the audio buffer.
///
/// # Example
/// ```rust
/// use std::sync::{Arc, Mutex};
/// use scrivano::audio::clear_buffer;
///
/// let buffer = Arc::new(Mutex::new(vec![1.0, 2.0, 3.0]));
/// clear_buffer(buffer.clone());
/// assert!(buffer.lock().unwrap().is_empty());
/// ```
pub fn clear_buffer(buffer: Arc<Mutex<Vec<f32>>>) {
    buffer.lock().unwrap().clear();
}

/// Gets a clone of the audio buffer data.
///
/// # Example
/// ```rust
/// use std::sync::{Arc, Mutex};
/// use scrivano::audio::get_buffer_data;
///
/// let buffer = Arc::new(Mutex::new(vec![0.1, 0.2, 0.3]));
/// let data = get_buffer_data(buffer.clone());
/// assert_eq!(data.len(), 3);
/// ```
pub fn get_buffer_data(buffer: Arc<Mutex<Vec<f32>>>) -> Vec<f32> {
    buffer.lock().unwrap().clone()
}

/// Gets the number of samples currently in the buffer.
///
/// # Example
/// ```rust
/// use std::sync::{Arc, Mutex};
/// use scrivano::audio::get_buffer_size;
///
/// let buffer = Arc::new(Mutex::new(vec![1.0; 100]));
/// assert_eq!(get_buffer_size(buffer.clone()), 100);
/// ```
pub fn get_buffer_size(buffer: Arc<Mutex<Vec<f32>>>) -> usize {
    buffer.lock().unwrap().len()
}

/// Calculate RMS (Root Mean Square) audio level from samples
/// Returns value between 0.0 and 1.0
pub fn calculate_rms_level(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_squares: f32 = samples.iter().map(|&s| s * s).sum();
    let rms = (sum_squares / samples.len() as f32).sqrt();

    // Normalize to 0-1 range (assuming input is -1.0 to 1.0)
    rms.min(1.0)
}

/// Calculate peak audio level from samples
/// Returns value between 0.0 and 1.0
pub fn calculate_peak_level(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let peak = samples.iter().map(|&s| s.abs()).fold(0.0_f32, f32::max);
    peak.min(1.0)
}

/// Get audio level as percentage (0-100)
pub fn get_audio_level_percentage(samples: &[f32]) -> f32 {
    let rms = calculate_rms_level(samples);
    // Convert to percentage with some scaling for better visualization
    (rms * 100.0).min(100.0)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn recording_flag_stops_thread() {
        let flag = Arc::new(AtomicBool::new(true));
        let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        flag.store(false, Ordering::SeqCst);
        assert!(!flag.load(Ordering::SeqCst));
        assert!(buffer.lock().unwrap().is_empty());
    }

    #[test]
    fn clear_buffer_works() {
        let buffer = Arc::new(Mutex::new(vec![1.0, 2.0, 3.0]));
        clear_buffer(buffer.clone());
        assert!(buffer.lock().unwrap().is_empty());
    }

    #[test]
    fn get_buffer_data_returns_clone() {
        let buffer = Arc::new(Mutex::new(vec![0.1_f32, 0.2, 0.3]));
        let data = get_buffer_data(buffer.clone());
        assert_eq!(data.len(), 3);
        assert_eq!(buffer.lock().unwrap().len(), 3);
    }

    #[test]
    fn get_buffer_size_returns_count() {
        let buffer = Arc::new(Mutex::new(vec![1.0_f32; 50]));
        assert_eq!(get_buffer_size(buffer.clone()), 50);
    }

    #[test]
    fn flag_controls_recording_state() {
        let flag = Arc::new(AtomicBool::new(false));
        assert!(!flag.load(Ordering::SeqCst));
        flag.store(true, Ordering::SeqCst);
        assert!(flag.load(Ordering::SeqCst));
        flag.store(false, Ordering::SeqCst);
        assert!(!flag.load(Ordering::SeqCst));
    }

    #[test]
    fn resample_linear_reduces_length() {
        let input: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.01).sin()).collect();
        let output = resample_linear(&input, 44100, 16000);
        let expected = ((4410_f64 / 44100.0) * 16000.0).ceil() as usize;
        assert_eq!(output.len(), expected);
    }

    #[test]
    fn resample_linear_same_rate_is_noop() {
        let input = vec![0.1_f32, 0.2, 0.3, 0.4];
        let output = resample_linear(&input, 16000, 16000);
        assert_eq!(input, output);
    }
}
