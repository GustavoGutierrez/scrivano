//! UI module for MeetWhisperer desktop application.

use std::{
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use eframe::egui::{self, Color32, FontId, Painter, Pos2, Rect, RichText, Sense, Stroke};
// Icon library: Phosphor Icons (https://phosphoricons.com/)
// This library provides high-quality SVG icons for the UI.
// Browse all available icons at: https://phosphoricons.com/
use egui_phosphor::regular as icons;
use hound::{WavSpec, WavWriter};
use whisper_rs::WhisperContext;

use crate::audio::spawn_system_audio_recorder;
use crate::audio_devices::{get_input_devices, get_output_devices, scan_models, AppSettings};
use crate::database::{Database, RecordingEntry, Summary};
use crate::ollama;
use crate::transcription::{transcribe_with_segments, TranscriptionLanguage};
use std::collections::HashMap;

// ── Palette ──────────────────────────────────────────────────────────────────
const BG_DARK: Color32 = Color32::from_rgb(14, 17, 23);
const BG_PANEL: Color32 = Color32::from_rgb(22, 27, 36);
const BG_CARD: Color32 = Color32::from_rgb(28, 34, 46);
const ACCENT_RED: Color32 = Color32::from_rgb(220, 50, 50);
const ACCENT_RED_HOVER: Color32 = Color32::from_rgb(255, 70, 70);
const ACCENT_GREEN: Color32 = Color32::from_rgb(34, 197, 94);
const ACCENT_GREEN_HOVER: Color32 = Color32::from_rgb(74, 222, 128);
const ACCENT_BLUE: Color32 = Color32::from_rgb(56, 139, 253);
const ACCENT_PURPLE: Color32 = Color32::from_rgb(139, 92, 246);
const TEXT_PRIMARY: Color32 = Color32::from_rgb(230, 237, 243);
const TEXT_DIM: Color32 = Color32::from_rgb(120, 130, 145);
const TEXT_MUTED: Color32 = Color32::from_rgb(75, 85, 99);
const BORDER: Color32 = Color32::from_rgb(40, 50, 65);

// Spectrum gradient stops (left → right): cyan → green → yellow
const GRAD: [(f32, Color32); 4] = [
    (0.0, Color32::from_rgb(0, 210, 255)),
    (0.33, Color32::from_rgb(0, 230, 120)),
    (0.66, Color32::from_rgb(100, 255, 80)),
    (1.0, Color32::from_rgb(255, 220, 0)),
];

// Modern audio spectrum gradient: cyan → blue → purple → pink
const SPECTRUM_GRAD: [(f32, Color32); 5] = [
    (0.0, Color32::from_rgb(0, 255, 255)),   // Cyan
    (0.25, Color32::from_rgb(0, 150, 255)),  // Blue
    (0.5, Color32::from_rgb(100, 50, 255)),  // Purple
    (0.75, Color32::from_rgb(200, 50, 255)), // Violet
    (1.0, Color32::from_rgb(255, 50, 200)),  // Pink
];

/// Interpolate color from spectrum gradient (0.0 to 1.0)
fn interpolate_spectrum_color(t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    for i in 0..SPECTRUM_GRAD.len() - 1 {
        let (t0, c0) = SPECTRUM_GRAD[i];
        let (t1, c1) = SPECTRUM_GRAD[i + 1];
        if t >= t0 && t <= t1 {
            let local_t = (t - t0) / (t1 - t0);
            return Color32::from_rgb(
                (c0.r() as f32 * (1.0 - local_t) + c1.r() as f32 * local_t) as u8,
                (c0.g() as f32 * (1.0 - local_t) + c1.g() as f32 * local_t) as u8,
                (c0.b() as f32 * (1.0 - local_t) + c1.b() as f32 * local_t) as u8,
            );
        }
    }
    SPECTRUM_GRAD[SPECTRUM_GRAD.len() - 1].1
}

// ── Icons (using simple ASCII/Unicode that renders reliably) ──────────────────
// Phosphor Icons - Real icon font
// Library: egui-phosphor (https://crates.io/crates/egui-phosphor)
// Icon catalog: https://phosphoricons.com/
// Usage: icons::ICON_NAME (e.g., icons::PLAY, icons::TRASH, etc.)
const ICON_PLAY: &str = icons::PLAY;
const ICON_PAUSE: &str = icons::PAUSE;
const ICON_STOP: &str = icons::STOP;
const ICON_RECORD: &str = icons::CIRCLE;
const ICON_EXPAND: &str = icons::CARET_DOWN;
const ICON_COLLAPSE: &str = icons::CARET_RIGHT;
const ICON_DELETE: &str = icons::TRASH;
const ICON_SETTINGS: &str = icons::GEAR;
const ICON_AUDIO: &str = icons::FILE_AUDIO;
const ICON_FILE: &str = icons::FILE_TEXT;
const ICON_MAGIC: &str = icons::SPARKLE;
const ICON_CHECK: &str = icons::CHECK;
const ICON_WARNING: &str = icons::WARNING;
const ICON_VOLUME: &str = icons::SPEAKER_HIGH;
const ICON_TRANSCRIPT: &str = icons::FILE_TEXT;
const ICON_COPY: &str = icons::COPY;
const ICON_EXPORT: &str = icons::DOWNLOAD_SIMPLE;
const ICON_FOLDER: &str = icons::FOLDER;
const ICON_CLOCK: &str = icons::CLOCK;
const ICON_MIC: &str = icons::MICROPHONE;
const ICON_SOUND: &str = icons::SPEAKER_HIGH;

// ── App struct ────────────────────────────────────────────────────────────────

#[derive(PartialEq, Clone)]
enum Tab {
    Recording,
    Settings,
    About,
}

pub struct App {
    pub recording: Arc<AtomicBool>,
    pub audio_buffer: Arc<Mutex<Vec<f32>>>,
    pub waveform_buffer: Arc<Mutex<Vec<f32>>>,
    pub transcript: Arc<Mutex<String>>,
    transcript_edit: String,
    whisper_ctx: Arc<WhisperContext>,
    active_tab: Tab,
    input_devices: Vec<(String, String)>,
    output_devices: Vec<(String, String)>,
    selected_input_index: usize,
    selected_output_index: usize,
    settings: AppSettings,
    is_transcribing: Arc<AtomicBool>,
    available_models: Vec<(String, String)>,
    selected_model_index: usize,
    model_changed: bool,
    // ── Ollama ────────────────────────────────────────────────────────────────
    ollama_available: bool,
    ollama_models: Vec<String>,
    ollama_selected_index: usize,
    ollama_enabled: bool,
    is_improving: Arc<AtomicBool>,
    // ── Progress tracking ─────────────────────────────────────────────────────
    /// Transcription progress 0-100. -1 = idle.
    transcribe_progress: Arc<AtomicI32>,
    /// Ollama improvement progress 0-100. -1 = idle.
    ollama_progress: Arc<AtomicI32>,
    // ── Recording timing ──────────────────────────────────────────────────────
    recording_start: Option<std::time::Instant>,
    recording_start_timestamp: Option<String>,
    last_recording_duration: f64,
    // ── Highlights ────────────────────────────────────────────────────────────
    current_recording_id: Option<i64>,
    pending_highlights: Arc<Mutex<Vec<(f64, Option<String>)>>>,
    // ── Database & history ────────────────────────────────────────────────────
    db: Option<Database>,
    recordings: Vec<RecordingEntry>,
    recordings_dirty: Arc<AtomicBool>,
    show_recordings: bool,
    expanded_recording_id: Option<i64>,
    current_summary_recording_id: Option<i64>,
    show_delete_confirmation: bool,
    recording_to_delete: Option<i64>,
    // ── Audio playback ──────────────────────────────────────────────────────
    #[cfg(feature = "audio-playback")]
    audio_player: Option<crate::playback::AudioPlayer>,
    current_playing_id: Option<i64>,
    playback_waveform: Vec<f32>,
    // ── Notifications ─────────────────────────────────────────────────────
    config_save_notification: Option<(String, bool)>, // (message, is_error)
    playback_volume: f32,
    // ── Summaries cache ────────────────────────────────────────────────────
    summaries_cache: HashMap<i64, Vec<Summary>>,
    // ── Summary generation tracking ────────────────────────────────────────
    generating_summaries: Arc<Mutex<HashMap<i64, Vec<String>>>>, // recording_id -> list of generating templates
    summary_generation_complete: Arc<AtomicBool>,                // flag to trigger reload
    // ── Smooth spectrum animation ──────────────────────────────────────────
    spectrum_bars: Vec<f32>, // smoothed bar values for animation
    spectrum_peak: Vec<f32>, // peak hold values
}

impl App {
    pub fn new(ctx: WhisperContext, settings: AppSettings) -> Self {
        let input_devices: Vec<(String, String)> = get_input_devices()
            .into_iter()
            .map(|d| (d.name.clone(), d.id.clone()))
            .collect();
        let input_count = input_devices.len();
        let selected_input_index = settings
            .input_device_id
            .as_ref()
            .and_then(|id| input_devices.iter().position(|(_, did)| did == id))
            .unwrap_or(0)
            .min(input_count.saturating_sub(1));

        let output_devices: Vec<(String, String)> = get_output_devices()
            .into_iter()
            .map(|d| (d.name.clone(), d.id.clone()))
            .collect();
        let output_count = output_devices.len();
        let selected_output_index = settings
            .output_device_id
            .as_ref()
            .and_then(|id| output_devices.iter().position(|(_, did)| did == id))
            .unwrap_or(0)
            .min(output_count.saturating_sub(1));

        let available_models = scan_models();
        let current_model = settings.whisper_model.clone();
        let selected_model_index = available_models
            .iter()
            .position(|(_, path)| *path == current_model)
            .unwrap_or(0);

        let ollama_available = ollama::is_available();
        let ollama_models = if ollama_available {
            ollama::list_models()
        } else {
            Vec::new()
        };
        let saved_ollama_model = settings.ollama_model.clone();
        let ollama_selected_index = ollama_models
            .iter()
            .position(|m| *m == saved_ollama_model)
            .unwrap_or(0);
        let ollama_enabled = settings.ollama_enabled && ollama_available;

        std::fs::create_dir_all(&settings.recordings_folder).ok();

        // ── Database ──────────────────────────────────────────────────────────
        let db_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("meet-whisperer")
            .join("recordings.db");
        // Ensure the config directory exists before SQLite tries to create the file
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let db = Database::open(&db_path).ok();
        let recordings = db
            .as_ref()
            .and_then(|d| d.list_recordings().ok())
            .unwrap_or_default();

        let initial = "Esperando grabación...".to_string();
        Self {
            recording: Arc::new(AtomicBool::new(false)),
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            waveform_buffer: Arc::new(Mutex::new(Vec::new())),
            transcript: Arc::new(Mutex::new(initial.clone())),
            transcript_edit: initial,
            whisper_ctx: Arc::new(ctx),
            active_tab: Tab::Recording,
            input_devices,
            output_devices,
            selected_input_index,
            selected_output_index,
            settings,
            is_transcribing: Arc::new(AtomicBool::new(false)),
            available_models,
            selected_model_index,
            model_changed: false,
            ollama_available,
            ollama_models,
            ollama_selected_index,
            ollama_enabled,
            is_improving: Arc::new(AtomicBool::new(false)),
            transcribe_progress: Arc::new(AtomicI32::new(-1)),
            ollama_progress: Arc::new(AtomicI32::new(-1)),
            recording_start: None,
            recording_start_timestamp: None,
            last_recording_duration: 0.0,
            current_recording_id: None,
            pending_highlights: Arc::new(Mutex::new(Vec::new())),
            db,
            recordings,
            recordings_dirty: Arc::new(AtomicBool::new(false)),
            show_recordings: true,
            expanded_recording_id: None,
            current_summary_recording_id: None,
            show_delete_confirmation: false,
            recording_to_delete: None,
            #[cfg(feature = "audio-playback")]
            audio_player: crate::playback::AudioPlayer::new(),
            current_playing_id: None,
            playback_waveform: Vec::new(),
            playback_volume: 0.8,
            config_save_notification: None,
            summaries_cache: HashMap::new(),
            generating_summaries: Arc::new(Mutex::new(HashMap::new())),
            summary_generation_complete: Arc::new(AtomicBool::new(false)),
            spectrum_bars: vec![0.0; 48], // 48 bars for smooth spectrum
            spectrum_peak: vec![0.0; 48], // peak hold values
        }
    }

    fn reload_recordings(&mut self) {
        if let Some(db) = &self.db {
            self.recordings = db.list_recordings().unwrap_or_default();
        }
        self.recordings_dirty.store(false, Ordering::SeqCst);
    }

    fn add_highlight_during_recording(&mut self, label: Option<String>) {
        if self.recording.load(Ordering::SeqCst) {
            let timestamp = self
                .recording_start
                .map(|s| s.elapsed().as_secs_f64())
                .unwrap_or(0.0);
            let mut highlights = self.pending_highlights.lock().unwrap();
            highlights.push((timestamp, label));
            eprintln!("[highlight] Added at {:.2}s", timestamp);
        }
    }

    fn save_pending_highlights(&mut self, recording_id: i64) {
        if let Some(db) = &self.db {
            let mut highlights = self.pending_highlights.lock().unwrap();
            for (timestamp, label) in highlights.drain(..) {
                if let Err(e) = db.insert_highlight(recording_id, timestamp, label.as_deref()) {
                    eprintln!("[highlight] Error saving: {}", e);
                } else {
                    eprintln!(
                        "[highlight] Saved at {:.2}s with label: {:?}",
                        timestamp, label
                    );
                }
            }
        }
    }
}

// ── eframe::App ───────────────────────────────────────────────────────────────

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Initialize Phosphor icons font on first frame
        static mut PHOSPHOR_INITIALIZED: bool = false;
        unsafe {
            if !PHOSPHOR_INITIALIZED {
                PHOSPHOR_INITIALIZED = true;
                let mut fonts = egui::FontDefinitions::default();
                egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
                ctx.set_fonts(fonts);
            }
        }

        ctx.style_mut(|s| {
            s.visuals.panel_fill = BG_DARK;
            s.visuals.window_fill = BG_DARK;
            s.visuals.override_text_color = Some(TEXT_PRIMARY);
            s.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
            s.visuals.widgets.inactive.bg_fill = BG_PANEL;
            s.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
        });

        // Keep repainting at 60fps while recording / processing / playing audio for smooth animations
        #[cfg(feature = "audio-playback")]
        let is_playing_audio = self
            .audio_player
            .as_ref()
            .map(|p| p.is_playing())
            .unwrap_or(false);
        #[cfg(not(feature = "audio-playback"))]
        let is_playing_audio = false;

        let needs_smooth_animation = self.recording.load(Ordering::SeqCst) || is_playing_audio;
        if needs_smooth_animation {
            // 60fps for smooth spectrum animation
            ctx.request_repaint_after(Duration::from_millis(16));
        } else if self.is_transcribing.load(Ordering::SeqCst)
            || self.is_improving.load(Ordering::SeqCst)
        {
            // 30fps is enough for progress bars
            ctx.request_repaint_after(Duration::from_millis(33));
        }

        // Reload recordings list if dirty
        if self.recordings_dirty.load(Ordering::SeqCst) {
            self.reload_recordings();
        }

        // Reload summaries when generation completes
        if self.summary_generation_complete.load(Ordering::SeqCst) {
            self.summary_generation_complete
                .store(false, Ordering::SeqCst);
            // Clear summaries cache to force reload on next view
            self.summaries_cache.clear();
            eprintln!("[ui] Resúmenes recargados tras generación");
        }

        // Sync shared transcript → local editable copy when idle
        if !self.recording.load(Ordering::SeqCst) {
            let shared = self.transcript.lock().unwrap().clone();
            if shared != self.transcript_edit {
                self.transcript_edit = shared;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // ── Tab bar ──────────────────────────────────────────────────────
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                for (tab, label, icon) in [
                    (Tab::Recording, "Grabación", ICON_RECORD),
                    (Tab::Settings, "Configuración", ICON_SETTINGS),
                    (Tab::About, "Acerca de", "?"),
                ] {
                    let active = self.active_tab == tab;
                    let (bg, fg) = if active {
                        (ACCENT_BLUE, Color32::WHITE)
                    } else {
                        (Color32::TRANSPARENT, TEXT_DIM)
                    };
                    let btn = egui::Button::new(
                        RichText::new(format!("{} {}", icon, label))
                            .size(14.0)
                            .color(fg),
                    )
                    .fill(bg)
                    .stroke(if active {
                        Stroke::new(0.0, Color32::TRANSPARENT)
                    } else {
                        Stroke::new(1.0, BORDER)
                    })
                    .rounding(6.0);
                    if ui.add(btn).clicked() {
                        self.active_tab = tab;
                    }
                    ui.add_space(4.0);
                }
            });
            ui.add_space(6.0);
            ui.separator();

            match self.active_tab {
                Tab::Recording => self.show_recording_tab(ui),
                Tab::Settings => self.show_settings_tab(ui),
                Tab::About => self.show_about_tab(ui),
            }
        });

        // Delete confirmation dialog (outside CentralPanel)
        if self.show_delete_confirmation {
            let recording_id = self.recording_to_delete.unwrap_or(0);
            let recording_name = self
                .recordings
                .iter()
                .find(|r| r.id == recording_id)
                .map(|r| r.filename.clone())
                .unwrap_or_else(|| "esta grabación".to_string());

            egui::Window::new("Confirmar eliminación")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(format!(
                            "¿Estás seguro de que deseas eliminar '{}'?",
                            recording_name
                        ))
                        .size(14.0),
                    );
                    ui.label(
                        RichText::new("Esta acción eliminará:\n• La grabación de la base de datos\n• El archivo de audio (.wav)\n• La transcripción (.txt)\n• Todos los resúmenes asociados")
                            .size(12.0)
                            .color(TEXT_DIM),
                    );
                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button(RichText::new("Eliminar").color(ACCENT_RED))
                                .clicked()
                            {
                                self.delete_recording(recording_id);
                                self.show_delete_confirmation = false;
                                self.recording_to_delete = None;
                            }

                            ui.add_space(8.0);

                            if ui.button("Cancelar").clicked() {
                                self.show_delete_confirmation = false;
                                self.recording_to_delete = None;
                            }
                        });
                    });
                });
        }
    }
}

// ── Tabs ──────────────────────────────────────────────────────────────────────

impl App {
    fn show_recording_tab(&mut self, ui: &mut egui::Ui) {
        let is_recording = self.recording.load(Ordering::SeqCst);
        let is_transcribing = self.is_transcribing.load(Ordering::SeqCst);
        let is_improving = self.is_improving.load(Ordering::SeqCst);
        let is_busy = is_transcribing || is_improving;

        let transcribe_pct = self.transcribe_progress.load(Ordering::SeqCst);
        let ollama_pct = self.ollama_progress.load(Ordering::SeqCst);

        ui.add_space(20.0);

        // ── Big action button ────────────────────────────────────────────────
        ui.vertical_centered(|ui| {
            if is_busy {
                let (label, pct) = if is_transcribing {
                    let p = if transcribe_pct >= 0 {
                        transcribe_pct
                    } else {
                        0
                    };
                    (format!("...  Transcribiendo...  {}%", p), p)
                } else {
                    let p = if ollama_pct >= 0 { ollama_pct } else { 0 };
                    (
                        format!("{}  Mejorando con Ollama...  {}%", ICON_MAGIC, p),
                        p,
                    )
                };
                ui.add_enabled_ui(false, |ui| {
                    ui.add_sized(
                        egui::vec2(340.0, 64.0),
                        egui::Button::new(RichText::new(&label).size(18.0).color(TEXT_DIM))
                            .fill(BG_PANEL),
                    );
                });
                // Progress bar
                ui.add_space(12.0);
                let bar_width = 340.0_f32;
                let bar_height = 8.0_f32;
                let (bar_rect, _) =
                    ui.allocate_exact_size(egui::vec2(bar_width, bar_height), Sense::hover());
                ui.painter().rect_filled(bar_rect, 4.0, BG_PANEL);
                let fill_w = bar_rect.width() * (pct as f32 / 100.0).clamp(0.0, 1.0);
                if fill_w > 0.0 {
                    let fill_rect =
                        Rect::from_min_size(bar_rect.min, egui::vec2(fill_w, bar_height));
                    let bar_color = if is_transcribing {
                        ACCENT_BLUE
                    } else {
                        ACCENT_PURPLE
                    };
                    ui.painter().rect_filled(fill_rect, 4.0, bar_color);
                }
            } else if is_recording {
                let btn = egui::Button::new(
                    RichText::new(format!("{}  Detener y transcribir", ICON_STOP))
                        .size(20.0)
                        .color(Color32::WHITE),
                )
                .fill(ACCENT_RED)
                .stroke(Stroke::new(2.0, ACCENT_RED_HOVER))
                .rounding(10.0);

                if ui.add_sized(egui::vec2(320.0, 64.0), btn).clicked() {
                    self.last_recording_duration = self
                        .recording_start
                        .map(|s| s.elapsed().as_secs_f64())
                        .unwrap_or(0.0);
                    self.stop_and_transcribe();
                }
            } else {
                let btn = egui::Button::new(
                    RichText::new(format!("{}  Iniciar grabación", ICON_RECORD))
                        .size(20.0)
                        .color(Color32::WHITE),
                )
                .fill(ACCENT_GREEN)
                .stroke(Stroke::new(2.0, ACCENT_GREEN_HOVER))
                .rounding(10.0);

                if ui.add_sized(egui::vec2(320.0, 64.0), btn).clicked() {
                    self.recording_start = Some(std::time::Instant::now());
                    self.start_recording();
                }
            }
        });

        ui.add_space(16.0);

        // ── Waveform/Spectrum during recording ────────────────────────────────
        if is_recording {
            let frame = egui::Frame::none()
                .fill(BG_PANEL)
                .stroke(Stroke::new(1.0, BORDER))
                .rounding(8.0)
                .inner_margin(egui::Margin::symmetric(16.0, 12.0));

            frame.show(ui, |ui| {
                ui.label(
                    RichText::new(format!("{} Audio en tiempo real", ICON_AUDIO))
                        .size(14.0)
                        .color(TEXT_DIM),
                );
                ui.add_space(8.0);

                let samples = self.waveform_buffer.lock().unwrap();

                let (rect, _resp) =
                    ui.allocate_exact_size(egui::vec2(ui.available_width(), 100.0), Sense::hover());
                let painter = ui.painter();
                let center_y = rect.center().y;
                let max_bar_height = 45.0;
                let num_bars = 48;
                let bar_width = (rect.width() - 20.0) / num_bars as f32;
                let gap = bar_width * 0.25; // 25% gap between bars
                let actual_bar_width = bar_width - gap;

                // Draw subtle center line
                painter.line_segment(
                    [
                        Pos2::new(rect.min.x + 5.0, center_y),
                        Pos2::new(rect.right() - 5.0, center_y),
                    ],
                    Stroke::new(1.0, Color32::from_rgb(40, 50, 65)),
                );

                // Calculate target values for each bar
                let mut target_values = vec![0.0f32; num_bars];

                if samples.len() >= 32 {
                    // Calculate RMS for each frequency band
                    let samples_per_band = samples.len() / num_bars;

                    for bar_idx in 0..num_bars {
                        let start_sample = bar_idx * samples_per_band;
                        let end_sample = ((bar_idx + 1) * samples_per_band).min(samples.len());

                        // Calculate RMS for this band
                        let mut sum_squares: f32 = 0.0;
                        let mut count = 0;
                        for i in start_sample..end_sample {
                            if i < samples.len() {
                                sum_squares += samples[i] * samples[i];
                                count += 1;
                            }
                        }
                        let rms = if count > 0 {
                            (sum_squares / count as f32).sqrt()
                        } else {
                            0.0
                        };

                        // Faster response with higher gain and less compression
                        target_values[bar_idx] = (rms * 12.0).min(1.0).powf(0.6);
                    }
                } else {
                    // Idle animation - subtle wave
                    let t = ui.input(|i| i.time) as f32 * 3.0;
                    for bar_idx in 0..num_bars {
                        let x = bar_idx as f32 / num_bars as f32;
                        target_values[bar_idx] = ((t + x * 10.0).sin() * 0.15 + 0.15).max(0.05);
                    }
                }

                // Smooth animation using lerp for fluid motion
                let smoothing_factor = 0.35; // Higher = faster response (0.0-1.0)
                let peak_decay = 0.92; // How fast peaks fall (0.0-1.0)

                for bar_idx in 0..num_bars {
                    // Smoothly interpolate current value to target
                    let current = self.spectrum_bars[bar_idx];
                    let target = target_values[bar_idx];
                    let smoothed = current + (target - current) * smoothing_factor;
                    self.spectrum_bars[bar_idx] = smoothed;

                    // Update peak hold
                    if smoothed > self.spectrum_peak[bar_idx] {
                        self.spectrum_peak[bar_idx] = smoothed;
                    } else {
                        self.spectrum_peak[bar_idx] *= peak_decay;
                    }

                    let bar_height = smoothed * max_bar_height;
                    let peak_height = self.spectrum_peak[bar_idx] * max_bar_height;

                    // Calculate x position
                    let x = rect.min.x + 10.0 + bar_idx as f32 * bar_width;

                    // Get color from gradient based on amplitude (changes with volume)
                    let color_t = bar_idx as f32 / num_bars as f32;
                    let base_color = interpolate_spectrum_color(color_t);
                    // Brightness varies with amplitude
                    let brightness = 0.5 + smoothed * 0.5;
                    let color = Color32::from_rgb(
                        (base_color.r() as f32 * brightness) as u8,
                        (base_color.g() as f32 * brightness) as u8,
                        (base_color.b() as f32 * brightness) as u8,
                    );

                    // Draw the bar (both up and down from center)
                    if bar_height > 1.0 {
                        // Upper bar with gradient effect
                        let upper_rect = Rect::from_min_max(
                            Pos2::new(x, center_y - bar_height),
                            Pos2::new(x + actual_bar_width, center_y - 1.0),
                        );
                        painter.rect_filled(upper_rect, 2.0, color);

                        // Lower bar (mirror)
                        let lower_rect = Rect::from_min_max(
                            Pos2::new(x, center_y + 1.0),
                            Pos2::new(x + actual_bar_width, center_y + bar_height),
                        );
                        painter.rect_filled(lower_rect, 2.0, color);

                        // Draw peak indicator
                        if peak_height > bar_height + 3.0 {
                            let peak_y = center_y - peak_height;
                            let peak_rect = Rect::from_min_max(
                                Pos2::new(x, peak_y),
                                Pos2::new(x + actual_bar_width, peak_y + 2.0),
                            );
                            painter.rect_filled(peak_rect, 1.0, base_color);
                        }
                    } else {
                        // Just a small indicator for low amplitude
                        let dot_rect = Rect::from_center_size(
                            Pos2::new(x + actual_bar_width / 2.0, center_y),
                            egui::vec2(actual_bar_width, 2.0),
                        );
                        painter.rect_filled(dot_rect, 1.0, base_color);
                    }
                }
            });
        }

        ui.add_space(12.0);

        // ── Recording status ─────────────────────────────────────────────────
        let status_height = 56.0;
        let desired_size = egui::vec2(ui.available_width(), status_height);
        let (response, painter) = ui.allocate_painter(desired_size, Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 8.0, BG_PANEL);

        if is_recording {
            let t = ui.input(|i| i.time);
            let blink = ((t * 2.5).sin() * 0.5 + 0.5) as f32;
            let dot = Color32::from_rgba_premultiplied(230, (40.0 + 200.0 * blink) as u8, 40, 255);
            painter.circle_filled(Pos2::new(rect.left() + 20.0, rect.center().y), 8.0, dot);
            painter.text(
                Pos2::new(rect.left() + 36.0, rect.center().y - 8.0),
                egui::Align2::LEFT_TOP,
                "GRABANDO",
                FontId::monospace(14.0),
                dot,
            );

            let elapsed = self
                .recording_start
                .map(|s| s.elapsed().as_secs_f32())
                .unwrap_or(0.0);
            let mins = (elapsed / 60.0) as u32;
            let secs = (elapsed % 60.0) as u32;
            painter.text(
                Pos2::new(rect.right() - 16.0, rect.center().y - 8.0),
                egui::Align2::RIGHT_TOP,
                format!("{:02}:{:02}", mins, secs),
                FontId::monospace(18.0),
                TEXT_PRIMARY,
            );

            // Highlight button during recording
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                let highlight_btn = egui::Button::new(
                    RichText::new("🏷️  Marcar highlight")
                        .size(14.0)
                        .color(ACCENT_PURPLE),
                )
                .fill(Color32::from_rgb(45, 35, 65))
                .stroke(Stroke::new(1.5, ACCENT_PURPLE))
                .rounding(8.0);

                if ui
                    .add(highlight_btn)
                    .on_hover_text("Ctrl+Shift+H")
                    .clicked()
                {
                    self.add_highlight_during_recording(None);
                }
            });
        } else if !is_busy {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Presiona Iniciar grabación para comenzar",
                FontId::proportional(14.0),
                TEXT_MUTED,
            );
        }

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(10.0);

        // ── Transcript area + Recordings history ─────────────────────────────
        // Split available height: transcript 40%, recordings list 60%
        let avail = ui.available_height();
        let transcript_h = if self.show_recordings {
            avail * 0.35
        } else {
            avail - 4.0
        };
        let recordings_h = avail - transcript_h - 32.0;

        ui.label(RichText::new("Transcripción").size(13.0).color(TEXT_DIM));
        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .id_source("transcript_scroll")
            .max_height(transcript_h)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.transcript_edit)
                        .desired_width(f32::INFINITY)
                        .desired_rows(6)
                        .font(FontId::proportional(16.0))
                        .text_color(TEXT_PRIMARY)
                        .interactive(true),
                );
            });

        ui.add_space(8.0);

        // ── Recordings history header ─────────────────────────────────────────
        ui.horizontal(|ui| {
            let arrow = if self.show_recordings {
                ICON_EXPAND
            } else {
                ICON_COLLAPSE
            };
            let count = self.recordings.len();
            let header = format!("{} Grabaciones recientes  ({})", arrow, count);
            if ui
                .add(
                    egui::Button::new(RichText::new(&header).size(13.0).color(TEXT_DIM))
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::NONE),
                )
                .clicked()
            {
                self.show_recordings = !self.show_recordings;
            }
        });

        if self.show_recordings {
            ui.add_space(4.0);
            egui::ScrollArea::vertical()
                .id_source("recordings_scroll")
                .max_height(recordings_h)
                .show(ui, |ui| {
                    if self.recordings.is_empty() {
                        ui.add_space(8.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("No hay grabaciones aún")
                                    .size(13.0)
                                    .color(TEXT_MUTED),
                            );
                        });
                    } else {
                        let expanded_id = self.expanded_recording_id;
                        let recordings = self.recordings.clone();
                        for entry in recordings {
                            let is_expanded = expanded_id == Some(entry.id);
                            let recording_id = entry.id;
                            let expand_btn = egui::Button::new(
                                RichText::new(if is_expanded {
                                    ICON_EXPAND
                                } else {
                                    ICON_COLLAPSE
                                })
                                .size(14.0)
                                .color(TEXT_DIM),
                            )
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE);

                            ui.horizontal(|ui| {
                                if ui.add(expand_btn).clicked() {
                                    if self.expanded_recording_id == Some(recording_id) {
                                        self.expanded_recording_id = None;
                                    } else {
                                        self.expanded_recording_id = Some(recording_id);
                                    }
                                }

                                ui.add_space(4.0);

                                let title = entry.title.as_deref().unwrap_or(&entry.filename);
                                let truncated = if title.len() > 40 {
                                    format!("{}...", &title[..37])
                                } else {
                                    title.to_string()
                                };

                                ui.label(
                                    RichText::new(truncated)
                                        .size(13.0)
                                        .color(TEXT_PRIMARY)
                                        .strong(),
                                );

                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new(format!("⏱ {}", entry.duration_display()))
                                        .size(11.0)
                                        .color(TEXT_DIM),
                                );

                                // Status badges
                                ui.add_space(8.0);
                                if entry.has_transcript {
                                    ui.label(
                                        RichText::new(ICON_FILE).size(11.0).color(ACCENT_GREEN),
                                    )
                                    .on_hover_text("Tiene transcripción");
                                }
                                if entry.has_summaries {
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(ICON_MAGIC).size(11.0).color(ACCENT_PURPLE),
                                    )
                                    .on_hover_text("Tiene resúmenes");
                                }

                                #[cfg(feature = "audio-playback")]
                                {
                                    ui.add_space(8.0);
                                    let btn = egui::Button::new(
                                        RichText::new(ICON_PLAY).size(12.0).color(ACCENT_GREEN),
                                    )
                                    .fill(Color32::TRANSPARENT)
                                    .stroke(Stroke::new(1.0, ACCENT_GREEN))
                                    .rounding(4.0);

                                    if ui.add(btn).clicked() {
                                        let wav_path = entry.filepath.replace(".txt", ".wav");
                                        if std::path::Path::new(&wav_path).exists() {
                                            if let Some(ref mut player) = self.audio_player {
                                                let _ = player.play(&wav_path);
                                                self.current_playing_id = Some(entry.id);
                                            }
                                        }
                                    }
                                }

                                // Delete button
                                ui.add_space(4.0);
                                let delete_btn = egui::Button::new(
                                    RichText::new("×").size(14.0).color(ACCENT_RED),
                                )
                                .fill(Color32::TRANSPARENT)
                                .stroke(Stroke::new(1.0, ACCENT_RED))
                                .rounding(4.0)
                                .min_size(egui::vec2(24.0, 24.0));

                                if ui
                                    .add(delete_btn)
                                    .on_hover_text("Eliminar grabación")
                                    .clicked()
                                {
                                    self.recording_to_delete = Some(entry.id);
                                    self.show_delete_confirmation = true;
                                }
                            });

                            // Show expanded content if expanded
                            if is_expanded {
                                ui.add_space(8.0);
                                self.show_recording_row_expanded(ui, &entry);
                            }

                            ui.add_space(4.0);
                        }
                    }
                });
        }
    }

    fn show_recording_row_expanded(&mut self, ui: &mut egui::Ui, entry: &RecordingEntry) {
        // Load summaries if not cached
        if !self.summaries_cache.contains_key(&entry.id) {
            if let Some(db) = &self.db {
                if let Ok(summaries) = db.get_summaries_by_recording(entry.id) {
                    self.summaries_cache.insert(entry.id, summaries);
                }
            }
        }
        let summaries = self
            .summaries_cache
            .get(&entry.id)
            .cloned()
            .unwrap_or_default();

        let frame = egui::Frame::none()
            .fill(BG_CARD)
            .stroke(Stroke::new(1.0, BORDER))
            .rounding(6.0)
            .inner_margin(egui::Margin::symmetric(10.0, 8.0));

        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                // Action buttons row
                ui.horizontal(|ui| {
                    // Title
                    let display_title = entry
                        .title.as_deref()
                        .unwrap_or(&entry.filename);
                    let truncated = if display_title.len() > 45 {
                        format!("{}...", &display_title[..42])
                    } else {
                        display_title.to_string()
                    };

                    ui.label(
                        RichText::new(truncated)
                            .size(14.0)
                            .color(TEXT_PRIMARY)
                            .strong(),
                    );

                    ui.add_space(8.0);

                    // Metadata row
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&entry.created_at)
                                .size(11.0)
                                .color(TEXT_MUTED),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(format!("⏱ {}", entry.duration_display()))
                                .size(11.0)
                                .color(TEXT_DIM),
                        );

                        if let Some(tags) = &entry.tags {
                            ui.add_space(8.0);
                            for tag in tags.split(',').take(2) {
                                let t = tag.trim();
                                if !t.is_empty() {
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(format!("🏷️ {}", t))
                                            .size(10.0)
                                            .color(ACCENT_BLUE),
                                    );
                                }
                            }
                        }

                        if entry.ollama_used {
                            ui.add_space(6.0);
                            ui.label(RichText::new("✨").size(11.0).color(ACCENT_PURPLE));
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Play button
                        #[cfg(feature = "audio-playback")]
                        {
                            let is_playing = self
                                .audio_player
                                .as_ref()
                                .map(|p| p.is_playing())
                                .unwrap_or(false);
                            let play_icon =
                                if self.current_playing_id == Some(entry.id) && is_playing {
                                    "⏸"
                                } else {
                                    "▶"
                                };
                            let btn = egui::Button::new(
                                RichText::new(play_icon).size(12.0).color(ACCENT_GREEN),
                            )
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::new(1.0, ACCENT_GREEN))
                            .rounding(4.0);

                            if ui.add(btn).on_hover_text("Reproducir").clicked() {
                                self.toggle_playback(entry);
                            }
                            ui.add_space(8.0);
                        }

                        // Open button
                        let open_btn =
                            egui::Button::new(RichText::new("📄").size(12.0).color(ACCENT_BLUE))
                                .fill(Color32::TRANSPARENT)
                                .stroke(Stroke::new(1.0, ACCENT_BLUE))
                                .rounding(4.0);

                        if ui.add(open_btn).on_hover_text("Abrir archivo").clicked() {
                            let _ = std::process::Command::new("xdg-open")
                                .arg(&entry.filepath)
                                .spawn();
                        }
                    });
                });

                // Expanded view content
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                // Action buttons
                ui.horizontal(|ui| {
                    // Export transcript
                    ui.menu_button("📄 Texto", |ui| {
                        ui.set_min_width(120.0);
                        if ui.button("📄 TXT").clicked() {
                            self.export_recording_transcript(entry, "txt");
                            ui.close_menu();
                        }
                        if ui.button("📝 Markdown").clicked() {
                            self.export_recording_transcript(entry, "md");
                            ui.close_menu();
                        }
                        if ui.button("📋 JSON").clicked() {
                            self.export_recording_transcript(entry, "json");
                            ui.close_menu();
                        }
                        if ui.button("🎬 SRT").clicked() {
                            self.export_recording_transcript(entry, "srt");
                            ui.close_menu();
                        }
                        if ui.button("🌐 VTT").clicked() {
                            self.export_recording_transcript(entry, "vtt");
                            ui.close_menu();
                        }
                    });

                    ui.add_space(8.0);

                    // Export audio
                    ui.menu_button("🎵 Audio", |ui| {
                        ui.set_min_width(140.0);
                        if ui.button("🎵 WAV (原始)").clicked() {
                            self.export_recording_audio(entry, "wav");
                            ui.close_menu();
                        }
                        if ui.button("🎵 MP3").clicked() {
                            ui.close_menu();
                        }
                        if ui.button("🎵 FLAC").clicked() {
                            ui.close_menu();
                        }
                    });

                    ui.add_space(16.0);

                    // Summary buttons (if Ollama available)
                    if self.ollama_available {
                        ui.label(RichText::new("✨ Resumir:").size(12.0).color(TEXT_DIM));
                        ui.add_space(4.0);

                        // Check which summaries are being generated
                        let generating = self.generating_summaries.lock().unwrap();
                        let is_generating_exec = generating.get(&entry.id).map(|v| v.contains(&"executive".to_string())).unwrap_or(false);
                        let is_generating_comp = generating.get(&entry.id).map(|v| v.contains(&"complete".to_string())).unwrap_or(false);
                        let is_generating_tasks = generating.get(&entry.id).map(|v| v.contains(&"tasks".to_string())).unwrap_or(false);
                        let is_generating_jira = generating.get(&entry.id).map(|v| v.contains(&"jira".to_string())).unwrap_or(false);
                        let is_generating_decisions = generating.get(&entry.id).map(|v| v.contains(&"decisions".to_string())).unwrap_or(false);
                        drop(generating);

                        // Check if transcript exists and is not empty
                        let has_transcript_content = std::fs::read_to_string(&entry.filepath)
                            .map(|content| !content.trim().is_empty() && content != "Grabando...")
                            .unwrap_or(false);
                        
                        if !has_transcript_content {
                            ui.label(RichText::new("⚠️ Transcripción vacía - no se puede generar resumen")
                                .size(11.0)
                                .color(ACCENT_RED));
                        } else {

                        // Row 1: Executive, Complete
                        ui.horizontal(|ui| {
                            let exec_label = if is_generating_exec { "⏳ Ejecutivo..." } else { "📋 Ejecutivo" };
                            let exec_btn =
                                egui::Button::new(RichText::new(exec_label).size(11.0))
                                    .fill(if is_generating_exec { Color32::from_rgb(80, 90, 110) } else { ACCENT_BLUE })
                                    .rounding(4.0);
                            if ui.add(exec_btn).clicked() && !is_generating_exec {
                                self.generate_summary_for_recording(entry.id, "executive");
                            }

                            ui.add_space(4.0);

                            let comp_label = if is_generating_comp { "⏳ Completo..." } else { "📄 Completo" };
                            let complete_btn =
                                egui::Button::new(RichText::new(comp_label).size(11.0))
                                    .fill(if is_generating_comp { Color32::from_rgb(80, 90, 110) } else { ACCENT_BLUE })
                                    .rounding(4.0);
                            if ui.add(complete_btn).clicked() && !is_generating_comp {
                                self.generate_summary_for_recording(entry.id, "complete");
                            }
                        });

                        ui.add_space(6.0);

                        // Row 2: Tasks, Jira Tasks, Decisions
                        ui.horizontal(|ui| {
                            let tasks_label = if is_generating_tasks { "⏳ Tareas..." } else { "✅ Tareas" };
                            let tasks_btn =
                                egui::Button::new(RichText::new(tasks_label).size(11.0))
                                    .fill(if is_generating_tasks { Color32::from_rgb(80, 90, 110) } else { ACCENT_BLUE })
                                    .rounding(4.0);
                            if ui.add(tasks_btn).clicked() && !is_generating_tasks {
                                self.generate_summary_for_recording(entry.id, "tasks");
                            }

                            ui.add_space(4.0);

                            let jira_label = if is_generating_jira { "⏳ Jira..." } else { "📊 Jira" };
                            let jira_btn = egui::Button::new(RichText::new(jira_label).size(11.0))
                                .fill(if is_generating_jira { Color32::from_rgb(80, 90, 110) } else { ACCENT_BLUE })
                                .rounding(4.0);
                            if ui.add(jira_btn).clicked() && !is_generating_jira {
                                self.generate_summary_for_recording(entry.id, "jira");
                            }

                            ui.add_space(4.0);

                            let decisions_label = if is_generating_decisions { "⏳ Decisiones..." } else { "📝 Decisiones" };
                            let decisions_btn =
                                egui::Button::new(RichText::new(decisions_label).size(11.0))
                                    .fill(if is_generating_decisions { Color32::from_rgb(80, 90, 110) } else { ACCENT_BLUE })
                                    .rounding(4.0);
                            if ui.add(decisions_btn).clicked() && !is_generating_decisions {
                                self.generate_summary_for_recording(entry.id, "decisions");
                            }
                        });
                        } // close else has_transcript_content
                    } else {
                        ui.label(RichText::new("⚠️ Ollama no disponible - resúmenes deshabilitados")
                            .size(11.0)
                            .color(ACCENT_RED));
                    }
                }); // close ui.horizontal

                // Show summaries if available
                if !summaries.is_empty() {
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);
                    
                    ui.label(RichText::new("✨ Resúmenes generados").size(13.0).color(TEXT_PRIMARY).strong());
                    ui.add_space(8.0);
                    
                    for summary in &summaries {
                        let template_label = match summary.template.as_str() {
                            "executive" => "📋 Ejecutivo",
                            "complete" => "📄 Completo",
                            "tasks" => "✅ Tareas",
                            "jira" => "📊 Jira",
                            "decisions" => "📝 Decisiones",
                            _ => &summary.template,
                        };
                        
                        egui::CollapsingHeader::new(
                            RichText::new(format!("{} (via {})", template_label, 
                                summary.model_name.as_deref().unwrap_or("Ollama")))
                                .size(12.0)
                                .color(ACCENT_BLUE)
                        )
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.add_space(4.0);
                            
                            // Show thinking block if available and user wants to see it
                            if summary.is_thinking_model && summary.raw_thinking.is_some() {
                                if self.settings.summary_thinking_policy == "show_for_debug" {
                                    ui.label(RichText::new("🧠 Proceso de thinking:").size(11.0).color(TEXT_DIM));
                                    ui.add_sized(
                                        egui::vec2(ui.available_width(), 80.0),
                                        egui::TextEdit::multiline(
                                            &mut summary.raw_thinking.clone().unwrap_or_default()
                                        )
                                        .font(FontId::proportional(10.0))
                                        .text_color(Color32::from_rgb(100, 120, 140))
                                    );
                                    ui.add_space(8.0);
                                }
                            }
                            
                            // Show summary content
                            let mut content = summary.content.clone();
                            ui.add_sized(
                                egui::vec2(ui.available_width(), 120.0),
                                egui::TextEdit::multiline(&mut content)
                                    .font(FontId::proportional(12.0))
                                    .text_color(TEXT_PRIMARY)
                            );
                            ui.add_space(8.0);
                            
                            // Copy button
                            if ui.button("📋 Copiar al portapapeles").clicked() {
                                ui.output_mut(|o| o.copied_text = summary.content.clone());
                            }
                        });
                        ui.add_space(4.0);
                    }
                }

                ui.add_space(12.0);

                // Audio playback controls (full player)
                #[cfg(feature = "audio-playback")]
                {
                    ui.add_space(8.0);
                    let frame = egui::Frame::none()
                        .fill(Color32::from_rgb(35, 42, 56))
                        .stroke(Stroke::new(1.0, BORDER))
                        .rounding(6.0)
                        .inner_margin(egui::Margin::symmetric(12.0, 8.0));

                    frame.show(ui, |ui| {
                        ui.label(RichText::new("🎧 Reproductor de audio").size(13.0).color(TEXT_PRIMARY).strong());
                        ui.add_space(8.0);

                        // Check if WAV file exists
                        let wav_path = entry.filepath.replace(".txt", ".wav");
                        let wav_exists = std::path::Path::new(&wav_path).exists();

                        if !wav_exists {
                            ui.label(RichText::new("⚠️ No hay archivo de audio (.wav) para esta grabación")
                                .size(12.0)
                                .color(ACCENT_RED));
                            ui.add_space(4.0);
                            ui.label(RichText::new("La grabación de audio del sistema debe estar habilitada para reproducir")
                                .size(11.0)
                                .color(TEXT_MUTED));
                        } else {
                            let player = self.audio_player.as_ref();
                            let elapsed = player.map(|p| {
                                if p.is_playing() || p.is_paused() {
                                    p.get_elapsed_secs()
                                } else {
                                    0.0
                                }
                            }).unwrap_or(0.0);
                            let total = entry.duration_secs;
                            let is_playing = player.map(|p| p.is_playing()).unwrap_or(false);
                            let is_current_item = self.current_playing_id == Some(entry.id);

                            // All controls in a single horizontal row
                            ui.horizontal(|ui| {
                                // Play/Pause button - perfectly circular
                                let button_size = 36.0;
                                let play_icon = if is_current_item && is_playing { ICON_PAUSE } else { ICON_PLAY };
                                let play_btn = egui::Button::new(
                                    RichText::new(play_icon).size(14.0).color(ACCENT_GREEN),
                                )
                                .fill(Color32::TRANSPARENT)
                                .stroke(Stroke::new(2.0, ACCENT_GREEN))
                                .rounding(button_size / 2.0)  // Perfect circle
                                .min_size(egui::vec2(button_size, button_size));

                                if ui.add(play_btn).on_hover_text(if is_playing { "Pausar" } else { "Reproducir" }).clicked() {
                                    self.toggle_playback(entry);
                                }

                                ui.add_space(12.0);

                                // Time display
                                ui.label(
                                    RichText::new(format!(
                                        "{} / {}",
                                        format_time_simple(elapsed),
                                        format_time_simple(total)
                                    ))
                                    .size(13.0)
                                    .color(TEXT_PRIMARY),
                                );

                                ui.add_space(16.0);

                                // Stop button - square
                                let stop_btn = egui::Button::new(
                                    RichText::new(ICON_STOP).size(12.0).color(TEXT_DIM),
                                )
                                .fill(Color32::TRANSPARENT)
                                .stroke(Stroke::new(1.0, TEXT_DIM))
                                .rounding(4.0)
                                .min_size(egui::vec2(28.0, 28.0));

                                if ui.add(stop_btn).on_hover_text("Detener").clicked() {
                                    self.stop_playback();
                                }

                                ui.add_space(24.0);

                                // Volume control with icon and slider
                                ui.label(RichText::new(ICON_VOLUME).size(14.0).color(TEXT_DIM));
                                let mut vol = self.playback_volume;
                                ui.add_sized(egui::vec2(80.0, 20.0), egui::Slider::new(&mut vol, 0.0..=1.0).show_value(false));
                                self.playback_volume = vol;

                                if let Some(ref player) = self.audio_player {
                                    player.set_volume(self.playback_volume);
                                }
                            });
                        }
                    });
                }

                ui.add_space(4.0);
            });
        });
    }

    fn toggle_playback(&mut self, entry: &RecordingEntry) {
        #[cfg(feature = "audio-playback")]
        {
            // Find the .wav file for this recording
            let wav_path = entry.filepath.replace(".txt", ".wav");

            if let Some(ref mut player) = self.audio_player {
                if self.current_playing_id == Some(entry.id) {
                    if player.is_playing() {
                        player.pause();
                    } else if player.is_paused() {
                        player.resume();
                    } else {
                        // Start playing from beginning - use .wav file
                        if std::path::Path::new(&wav_path).exists() {
                            if let Ok(_) = player.play(&wav_path) {
                                self.current_playing_id = Some(entry.id);
                            }
                        }
                    }
                } else {
                    // Play new file - use .wav file
                    if std::path::Path::new(&wav_path).exists() {
                        if let Ok(_) = player.play(&wav_path) {
                            self.current_playing_id = Some(entry.id);
                        }
                    }
                }
            }
        }
    }

    fn stop_playback(&mut self) {
        #[cfg(feature = "audio-playback")]
        {
            if let Some(ref mut player) = self.audio_player {
                player.stop();
            }
            self.current_playing_id = None;
        }
    }

    fn delete_recording(&mut self, recording_id: i64) {
        eprintln!(
            "[delete] Iniciando eliminación de grabación {}",
            recording_id
        );

        // Find the recording to get file paths
        let recording = self
            .recordings
            .iter()
            .find(|r| r.id == recording_id)
            .cloned();

        if let Some(rec) = recording {
            // Delete associated files
            let base_path = rec.filepath.replace(".txt", "");

            // Files to delete
            let files_to_delete = vec![
                format!("{}.txt", base_path),
                format!("{}.wav", base_path),
                format!("{}.mp3", base_path),
                format!("{}.flac", base_path),
            ];

            for file_path in files_to_delete {
                if std::path::Path::new(&file_path).exists() {
                    match std::fs::remove_file(&file_path) {
                        Ok(_) => eprintln!("[delete] Archivo eliminado: {}", file_path),
                        Err(e) => eprintln!("[delete] Error eliminando {}: {}", file_path, e),
                    }
                }
            }

            // Delete from database
            if let Some(db) = &self.db {
                match db.delete_recording(recording_id) {
                    Ok(_) => {
                        eprintln!(
                            "[delete] Grabación {} eliminada de la base de datos",
                            recording_id
                        );
                    }
                    Err(e) => {
                        eprintln!("[delete] Error eliminando de DB: {}", e);
                    }
                }
            }

            // Clear from cache
            self.summaries_cache.remove(&recording_id);

            // Reload recordings list
            self.reload_recordings();

            // If currently playing this recording, stop playback
            if self.current_playing_id == Some(recording_id) {
                self.stop_playback();
            }

            // If this was expanded, collapse it
            if self.expanded_recording_id == Some(recording_id) {
                self.expanded_recording_id = None;
            }

            eprintln!(
                "[delete] Eliminación completada para grabación {}",
                recording_id
            );
        }
    }

    fn export_recording_transcript(&self, entry: &RecordingEntry, format: &str) {
        let folder = std::path::Path::new(&entry.filepath)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let transcript_text = std::fs::read_to_string(&entry.filepath).unwrap_or_default();

        // Load segments and highlights from database for SRT/VTT/JSON
        let db_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("meet-whisperer")
            .join("recordings.db");

        let (segments, highlights) = if let Ok(db) = Database::open(&db_path) {
            let segs = db.get_segments_by_recording(entry.id).unwrap_or_default();
            let highs = db.get_highlights_by_recording(entry.id).unwrap_or_default();
            (segs, highs)
        } else {
            (Vec::new(), Vec::new())
        };

        let (final_text, ext) = match format {
            "txt" => (transcript_text.clone(), "txt"),
            "md" => {
                let mut content = format!(
                    "# {}\n\n**Fecha:** {}\n**Duración:** {}\n**Tags:** {}\n\n---\n\n",
                    entry.title.as_ref().unwrap_or(&entry.filename),
                    entry.created_at,
                    entry.duration_display(),
                    entry.tags.as_ref().unwrap_or(&"Sin tags".to_string())
                );

                // Add highlights section if available
                if !highlights.is_empty() {
                    content.push_str("## Highlights\n\n");
                    for highlight in &highlights {
                        let time_str = format_time_simple(highlight.timestamp_sec);
                        let label_str = highlight
                            .label
                            .as_ref()
                            .map(|l| format!(" - {}", l))
                            .unwrap_or_default();
                        content.push_str(&format!("- **{}**{}\n", time_str, label_str));
                    }
                    content.push_str("\n---\n\n");
                }

                content.push_str(&transcript_text);
                (content, "md")
            }
            "json" => {
                let json = serde_json::json!({
                    "recording": {
                        "filename": entry.filename,
                        "created_at": entry.created_at,
                        "duration_secs": entry.duration_secs,
                        "title": entry.title,
                        "tags": entry.tags,
                    },
                    "highlights": highlights.iter().map(|h| {
                        serde_json::json!({
                            "timestamp_sec": h.timestamp_sec,
                            "label": h.label,
                        })
                    }).collect::<Vec<_>>(),
                    "segments": segments.iter().map(|s| {
                        serde_json::json!({
                            "start_sec": s.start_sec,
                            "end_sec": s.end_sec,
                            "text": s.text,
                        })
                    }).collect::<Vec<_>>(),
                });
                (
                    serde_json::to_string_pretty(&json).unwrap_or_default(),
                    "json",
                )
            }
            "srt" => {
                // Generate SRT with real timestamps from segments
                let mut srt_content = String::new();
                if !segments.is_empty() {
                    for (i, segment) in segments.iter().enumerate() {
                        let start = format_srt_timestamp(segment.start_sec);
                        let end = format_srt_timestamp(segment.end_sec);
                        srt_content.push_str(&format!(
                            "{}\n{} --> {}\n{}\n\n",
                            i + 1,
                            start,
                            end,
                            segment.text
                        ));
                    }
                } else {
                    // Fallback: create a single entry with full text
                    srt_content =
                        format!("1\n00:00:00,000 --> 00:00:00,000\n{}\n\n", transcript_text);
                }
                (srt_content, "srt")
            }
            "vtt" => {
                // Generate WebVTT with real timestamps from segments
                let mut vtt_content = "WEBVTT\n\n".to_string();
                if !segments.is_empty() {
                    for (i, segment) in segments.iter().enumerate() {
                        let start = format_vtt_timestamp(segment.start_sec);
                        let end = format_vtt_timestamp(segment.end_sec);
                        vtt_content.push_str(&format!(
                            "{}\n{} --> {}\n{}\n\n",
                            i + 1,
                            start,
                            end,
                            segment.text
                        ));
                    }
                } else {
                    // Fallback: create a single entry with full text
                    vtt_content = format!(
                        "WEBVTT\n\n1\n00:00:00.000 --> 00:00:00.000\n{}\n\n",
                        transcript_text
                    );
                }
                (vtt_content, "vtt")
            }
            _ => (transcript_text, "txt"),
        };

        let output_path = format!("{}/export_{}_{}.{}", folder, entry.id, timestamp, ext);
        if std::fs::write(&output_path, &final_text).is_ok() {
            let _ = std::process::Command::new("xdg-open")
                .arg(&output_path)
                .spawn();
        }
    }

    fn export_recording_audio(&self, entry: &RecordingEntry, format: &str) {
        let output_path = entry.filepath.replace(".txt", &format!(".{}", format));
        if std::path::Path::new(&output_path).exists() {
            let parent = std::path::Path::new(&output_path)
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_default();
            let _ = std::process::Command::new("xdg-open").arg(parent).spawn();
        }
    }

    fn generate_summary_for_recording(&mut self, recording_id: i64, template: &str) {
        if !self.ollama_available {
            eprintln!("[summary] Ollama no está disponible");
            return;
        }

        let entry = self
            .recordings
            .iter()
            .find(|e| e.id == recording_id)
            .cloned();
        if let Some(entry) = entry {
            let transcript = std::fs::read_to_string(&entry.filepath).unwrap_or_default();
            if transcript.is_empty() {
                eprintln!("[summary] Transcripción vacía");
                return;
            }

            // Use summary model from settings, fallback to ollama model
            let model = if !self.settings.summary_model.is_empty() {
                self.settings.summary_model.clone()
            } else {
                self.ollama_models
                    .get(self.ollama_selected_index)
                    .cloned()
                    .unwrap_or_else(|| "llama3.2".to_string())
            };

            let summary_template = match template {
                "executive" => crate::summarization::SummaryTemplate::Executive,
                "complete" => crate::summarization::SummaryTemplate::Complete,
                "tasks" => crate::summarization::SummaryTemplate::Tasks,
                "jira" => crate::summarization::SummaryTemplate::Jira,
                "decisions" => crate::summarization::SummaryTemplate::Decisions,
                _ => crate::summarization::SummaryTemplate::Executive,
            };

            // Mark this summary as generating
            {
                let mut gen = self.generating_summaries.lock().unwrap();
                gen.entry(recording_id)
                    .or_insert_with(Vec::new)
                    .push(template.to_string());
            }

            // Clear cache to force reload when generation completes
            self.summaries_cache.remove(&recording_id);

            // Get custom prompt from settings if available (clone for thread)
            let custom_prompt_str: Option<String> = match template {
                "executive" => {
                    let s = &self.settings.custom_prompt_executive;
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.clone())
                    }
                }
                "tasks" => {
                    let s = &self.settings.custom_prompt_tasks;
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.clone())
                    }
                }
                "decisions" => {
                    let s = &self.settings.custom_prompt_decisions;
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.clone())
                    }
                }
                _ => None,
            };

            self.current_summary_recording_id = Some(recording_id);
            let recording_id_copy = recording_id;
            let template_string = template.to_string();
            let model_clone = model.to_string();
            let db_path = dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("meet-whisperer")
                .join("recordings.db");
            let generating_clone = self.generating_summaries.clone();
            let complete_flag = self.summary_generation_complete.clone();

            std::thread::spawn(move || {
                let client = crate::ollama::OllamaClient::new("http://localhost", 11434);

                // Convert Option<String> to Option<String> (owned) for the thread
                let custom_prompt_ref: Option<String> = custom_prompt_str;

                let result = crate::summarization::generate_summary(
                    &client,
                    &transcript,
                    summary_template,
                    &model_clone,
                    custom_prompt_ref.as_deref(),
                );

                match result {
                    Ok(summary_result) => {
                        eprintln!(
                            "[summary] {} generated for recording {}: {} chars",
                            template_string,
                            recording_id_copy,
                            summary_result.content.len()
                        );

                        // Save summary to database
                        if let Ok(db) = Database::open(&db_path) {
                            match db.insert_summary(
                                recording_id_copy,
                                &template_string,
                                &summary_result.content,
                                Some(&summary_result.model_name),
                                summary_result.is_thinking_model,
                                summary_result.raw_thinking.as_deref(),
                            ) {
                                Ok(_) => {
                                    eprintln!("[summary] Guardado en base de datos exitosamente");
                                }
                                Err(e) => {
                                    eprintln!("[summary] Error guardando en DB: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[summary] Error generando resumen: {:?}", e);
                    }
                }

                // Remove from generating list and mark complete
                {
                    let mut gen = generating_clone.lock().unwrap();
                    if let Some(templates) = gen.get_mut(&recording_id_copy) {
                        templates.retain(|t| t != &template_string);
                        if templates.is_empty() {
                            gen.remove(&recording_id_copy);
                        }
                    }
                }
                complete_flag.store(true, Ordering::SeqCst);
            });
        }
    }

    fn start_recording(&mut self) {
        self.audio_buffer.lock().unwrap().clear();
        self.waveform_buffer.lock().unwrap().clear();
        self.waveform_buffer.lock().unwrap().clear();
        self.pending_highlights.lock().unwrap().clear();
        self.transcript_edit = "Grabando...".to_string();
        *self.transcript.lock().unwrap() = "Grabando...".to_string();
        self.recording.store(true, Ordering::SeqCst);
        self.recording_start_timestamp = Some(chrono_local_now());

        let source_name = self
            .input_devices
            .get(self.selected_input_index)
            .map(|(_, id)| id.clone())
            .unwrap_or_default();

        eprintln!("[ui] Grabando desde: {:?}", source_name);

        spawn_system_audio_recorder(
            self.recording.clone(),
            self.audio_buffer.clone(),
            self.waveform_buffer.clone(),
            source_name,
        );
    }

    fn stop_and_transcribe(&mut self) {
        self.recording.store(false, Ordering::SeqCst);

        let pending_highlights = self.pending_highlights.clone();
        let buffer = self.audio_buffer.clone();
        let transcript = self.transcript.clone();
        let ctx = self.whisper_ctx.clone();
        let is_transcribing = self.is_transcribing.clone();
        let is_improving = self.is_improving.clone();
        let folder = self.settings.recordings_folder.clone();
        let ollama_enabled = self.ollama_enabled;
        let ollama_model = self
            .ollama_models
            .get(self.ollama_selected_index)
            .cloned()
            .unwrap_or_default();
        let duration_secs = self.last_recording_duration;
        let recordings_dirty = self.recordings_dirty.clone();
        let transcribe_progress = self.transcribe_progress.clone();
        let ollama_progress = self.ollama_progress.clone();
        let language_default = self.settings.language_default.clone();

        // DB path for inserting the new entry
        let db_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("meet-whisperer")
            .join("recordings.db");

        is_transcribing.store(true, Ordering::SeqCst);
        transcribe_progress.store(0, Ordering::SeqCst);

        std::thread::spawn(move || {
            let audio_data = buffer.lock().unwrap().clone();
            eprintln!(
                "[transcribe] {} muestras ({:.1}s a 16 kHz)",
                audio_data.len(),
                audio_data.len() as f32 / 16_000.0
            );

            // ── Paso 1: Whisper ──────────────────────────────────────────────
            let tp = transcribe_progress.clone();
            let (raw_text, segments) = if audio_data.is_empty() {
                tp.store(100, Ordering::SeqCst);
                (
                    "(Buffer vacío — no se capturó audio)".to_string(),
                    Vec::new(),
                )
            } else {
                let tp2 = tp.clone();
                let language = TranscriptionLanguage::from_code(&language_default)
                    .unwrap_or(TranscriptionLanguage::Spanish);
                match transcribe_with_segments(&ctx, &audio_data, language, move |pct| {
                    tp2.store(pct, Ordering::SeqCst);
                }) {
                    Ok((text, segs)) => {
                        if text.trim().is_empty() {
                            ("(Whisper no detectó habla)".to_string(), segs)
                        } else {
                            (text, segs)
                        }
                    }
                    Err(e) => {
                        eprintln!("Error transcripción: {}", e);
                        (format!("Error transcripción: {}", e), Vec::new())
                    }
                }
            };

            is_transcribing.store(false, Ordering::SeqCst);
            transcribe_progress.store(-1, Ordering::SeqCst);

            *transcript.lock().unwrap() = raw_text.clone();

            // ── Paso 2: Mejora con Ollama (opcional) ─────────────────────────
            let ollama_model_used: Option<String>;
            let final_text = if ollama_enabled
                && !ollama_model.is_empty()
                && !raw_text.starts_with('(')
                && !raw_text.starts_with("Error")
            {
                is_improving.store(true, Ordering::SeqCst);
                ollama_progress.store(0, Ordering::SeqCst);
                *transcript.lock().unwrap() = format!("{}\n\n[Mejorando con Ollama…]", raw_text);

                let op = ollama_progress.clone();
                let improved = match ollama::improve_transcript(
                    &ollama_model,
                    &raw_text,
                    move |pct| {
                        op.store(pct, Ordering::SeqCst);
                    },
                    None,
                ) {
                    Ok(t) => {
                        eprintln!("[ollama] mejora completada ({} chars)", t.len());
                        t
                    }
                    Err(e) => {
                        eprintln!("[ollama] error: {}", e);
                        raw_text.clone()
                    }
                };

                is_improving.store(false, Ordering::SeqCst);
                ollama_progress.store(-1, Ordering::SeqCst);
                ollama_model_used = Some(ollama_model.clone());
                improved
            } else {
                ollama_model_used = None;
                raw_text.clone()
            };

            // ── Guardar archivo ──────────────────────────────────────────────
            if !final_text.starts_with('(') && !final_text.starts_with("Error") {
                // Format datetime for filename and DB
                let now = chrono_local_now();
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let filename = format!("grabacion_{}.txt", ts);
                let wav_filename = format!("grabacion_{}.wav", ts);
                let path = format!("{}/{}", folder, filename);
                let wav_path = format!("{}/{}", folder, wav_filename);

                // Save transcript
                if std::fs::write(&path, &final_text).is_ok() {
                    eprintln!("[transcribe] guardado en {}", path);

                    // Save WAV audio
                    if !audio_data.is_empty() {
                        let spec = WavSpec {
                            channels: 1,
                            sample_rate: 16000,
                            bits_per_sample: 16,
                            sample_format: hound::SampleFormat::Int,
                        };
                        if let Ok(mut writer) = WavWriter::create(&wav_path, spec) {
                            for sample in &audio_data {
                                let sample_i16 = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                                let _ = writer.write_sample(sample_i16);
                            }
                            drop(writer);
                            eprintln!("[audio] WAV guardado en {}", wav_path);
                        } else {
                            eprintln!("[audio] Error creando archivo WAV");
                        }
                    }

                    // Insert into DB
                    if let Ok(db) = Database::open(&db_path) {
                        let recording_id = db.insert_recording(
                            &filename,
                            &path,
                            &now,
                            duration_secs,
                            ollama_model_used.is_some(),
                            ollama_model_used.as_deref(),
                            None, // title - will be generated later
                            None, // tags - will be generated later
                        );

                        if let Ok(rid) = recording_id {
                            // Save transcript segments with timestamps
                            if !segments.is_empty() {
                                for segment in &segments {
                                    if let Err(e) = db.insert_segment(
                                        rid,
                                        segment.start_sec,
                                        segment.end_sec,
                                        &segment.text,
                                    ) {
                                        eprintln!("[segment] Error saving: {}", e);
                                    }
                                }
                                eprintln!("[segment] Saved {} segments", segments.len());
                            }

                            // Save pending highlights after recording is created
                            let highlights_to_save = pending_highlights.lock().unwrap().clone();
                            for (timestamp, label) in highlights_to_save {
                                if let Err(e) =
                                    db.insert_highlight(rid, timestamp, label.as_deref())
                                {
                                    eprintln!("[highlight] Error saving: {}", e);
                                } else {
                                    eprintln!("[highlight] Saved at {:.2}s", timestamp);
                                }
                            }

                            // Generate title and tags with Ollama if available
                            if ollama_enabled
                                && !ollama_model.is_empty()
                                && !final_text.starts_with('(')
                                && !final_text.starts_with("Error")
                            {
                                eprintln!("[ollama] Generando título y tags...");

                                // Generate title
                                let title_prompt = format!(
                                    "Genera un título corto (máximo 6 palabras) que resuma el siguiente texto de una reunión. Solo responde con el título, nada más:\n\n{}",
                                    &final_text[..final_text.len().min(2000)]
                                );

                                match ollama::improve_transcript(
                                    &ollama_model,
                                    &title_prompt,
                                    |_| {},
                                    None,
                                ) {
                                    Ok(title) => {
                                        let clean_title =
                                            title.trim().replace('"', "").replace('\n', " ");
                                        eprintln!("[ollama] Título generado: {}", clean_title);

                                        // Generate tags
                                        let tags_prompt = format!(
                                            "Analiza el siguiente texto y genera exactamente 5 tags separados por comas que describan: 1) El sentimiento general, 2-4) Los temas principales discutidos, 5) El tipo de reunión. Solo responde con los 5 tags separados por comas, nada más:\n\n{}",
                                            &final_text[..final_text.len().min(3000)]
                                        );

                                        match ollama::improve_transcript(
                                            &ollama_model,
                                            &tags_prompt,
                                            |_| {},
                                            None,
                                        ) {
                                            Ok(tags) => {
                                                let clean_tags = tags
                                                    .trim()
                                                    .replace('"', "")
                                                    .replace('\n', ", ");
                                                eprintln!(
                                                    "[ollama] Tags generados: {}",
                                                    clean_tags
                                                );

                                                // Update recording with title and tags
                                                if let Err(e) = db.update_recording_title_and_tags(
                                                    rid,
                                                    Some(&clean_title),
                                                    Some(&clean_tags),
                                                ) {
                                                    eprintln!(
                                                        "[db] Error updating title/tags: {}",
                                                        e
                                                    );
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("[ollama] Error generando tags: {}", e);
                                                // Still save the title even if tags failed
                                                if let Err(e) = db.update_recording_title_and_tags(
                                                    rid,
                                                    Some(&clean_title),
                                                    None,
                                                ) {
                                                    eprintln!("[db] Error updating title: {}", e);
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("[ollama] Error generando título: {}", e);
                                    }
                                }
                            }
                        }

                        recordings_dirty.store(true, Ordering::SeqCst);
                    }
                }
            }

            *transcript.lock().unwrap() = final_text;
        });
    }

    // ── Settings tab ─────────────────────────────────────────────────────────

    fn show_settings_tab(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_source("settings_scroll")
            .show(ui, |ui| {
                ui.add_space(24.0);

                // Panel frame with better spacing
                let frame = egui::Frame::none().inner_margin(egui::Margin::symmetric(24.0, 16.0));

                frame.show(ui, |ui| {
                    // ── Section: Modelo Whisper ──────────────────────────────────
                    section_header(ui, "🎙  Modelo de transcripción (Whisper)");
                    ui.add_space(12.0);

                    if self.available_models.is_empty() {
                        ui.label(
                            RichText::new("No se encontraron modelos en models/")
                                .size(14.0)
                                .color(ACCENT_RED),
                        );
                    } else {
                        let current_name = self
                            .available_models
                            .get(self.selected_model_index)
                            .map(|(n, _)| n.as_str())
                            .unwrap_or("—");

                        egui::ComboBox::from_id_source("whisper_model_combo")
                            .selected_text(RichText::new(current_name).size(15.0))
                            .width(ui.available_width())
                            .show_ui(ui, |ui| {
                                for (i, (name, _)) in self.available_models.iter().enumerate() {
                                    let selected = i == self.selected_model_index;
                                    if ui
                                        .selectable_label(selected, RichText::new(name).size(15.0))
                                        .clicked()
                                        && i != self.selected_model_index
                                    {
                                        self.selected_model_index = i;
                                        self.model_changed = true;
                                    }
                                }
                            });
                    }

                    if self.model_changed {
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("⚠ El modelo nuevo se aplica al reiniciar la app.")
                                .size(12.0)
                                .color(Color32::from_rgb(255, 200, 60)),
                        );
                    }

                    ui.add_space(24.0);
                    ui.separator();
                    ui.add_space(16.0);

                    // ── Section: Dispositivo de entrada ──────────────────────────
                    section_header(ui, "🎤  Dispositivo de entrada (Micrófono)");
                    ui.add_space(12.0);

                    if self.input_devices.is_empty() {
                        ui.label(
                            RichText::new("No se encontraron dispositivos de entrada")
                                .size(14.0)
                                .color(TEXT_DIM),
                        );
                    } else {
                        let current_input = self
                            .input_devices
                            .get(self.selected_input_index)
                            .map(|(n, _)| n.as_str())
                            .unwrap_or("—");

                        egui::ComboBox::from_id_source("input_device_combo")
                            .selected_text(RichText::new(current_input).size(15.0))
                            .width(ui.available_width())
                            .show_ui(ui, |ui| {
                                for (i, (name, _)) in self.input_devices.iter().enumerate() {
                                    let selected = i == self.selected_input_index;
                                    if ui
                                        .selectable_label(selected, RichText::new(name).size(15.0))
                                        .clicked()
                                    {
                                        self.selected_input_index = i;
                                    }
                                }
                            });
                    }

                    ui.add_space(24.0);
                    ui.separator();
                    ui.add_space(16.0);

                    // ── Section: Dispositivo de salida ────────────────────────────
                    section_header(ui, "🔊  Dispositivo de salida (Audio del sistema)");
                    ui.add_space(12.0);

                    if self.output_devices.is_empty() {
                        ui.label(
                            RichText::new("No se encontraron dispositivos de salida")
                                .size(14.0)
                                .color(TEXT_DIM),
                        );
                    } else {
                        let current_output = self
                            .output_devices
                            .get(self.selected_output_index)
                            .map(|(n, _)| n.as_str())
                            .unwrap_or("—");

                        egui::ComboBox::from_id_source("output_device_combo")
                            .selected_text(RichText::new(current_output).size(15.0))
                            .width(ui.available_width())
                            .show_ui(ui, |ui| {
                                for (i, (name, _)) in self.output_devices.iter().enumerate() {
                                    let selected = i == self.selected_output_index;
                                    if ui
                                        .selectable_label(selected, RichText::new(name).size(15.0))
                                        .clicked()
                                    {
                                        self.selected_output_index = i;
                                    }
                                }
                            });
                    }

                    ui.label(
                    RichText::new(
                        "  La captura del sistema usa el monitor del sink de PulseAudio/PipeWire.",
                    )
                    .size(12.0)
                    .color(TEXT_MUTED),
                );

                    ui.add_space(24.0);
                    ui.separator();
                    ui.add_space(16.0);

                    // ── Section: Ollama ───────────────────────────────────────────
                    section_header(ui, "✨  Mejora con Ollama");
                    ui.add_space(12.0);

                    if !self.ollama_available {
                        status_badge(
                            ui,
                            "Ollama no detectado en localhost:11434",
                            Color32::from_rgb(200, 60, 60),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(
                                "Instala Ollama (ollama.com) y asegúrate de que esté corriendo.",
                            )
                            .size(12.0)
                            .color(TEXT_DIM),
                        );
                    } else {
                        status_badge(ui, "Ollama disponible", ACCENT_GREEN);
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            let toggle_text = if self.ollama_enabled {
                                RichText::new("Activado").size(15.0).color(ACCENT_GREEN)
                            } else {
                                RichText::new("Desactivado").size(15.0).color(TEXT_DIM)
                            };
                            ui.checkbox(&mut self.ollama_enabled, toggle_text);
                        });

                        if self.ollama_enabled {
                            ui.add_space(10.0);
                            ui.label(
                                RichText::new("Modelo para mejorar transcripción")
                                    .size(13.0)
                                    .color(TEXT_DIM),
                            );
                            ui.add_space(4.0);

                            if self.ollama_models.is_empty() {
                                ui.label(
                                    RichText::new("No hay modelos instalados en Ollama")
                                        .size(14.0)
                                        .color(ACCENT_RED),
                                );
                            } else {
                                let current_name = self
                                    .ollama_models
                                    .get(self.ollama_selected_index)
                                    .cloned()
                                    .unwrap_or_default();

                                egui::ComboBox::from_id_source("ollama_model_combo")
                                    .selected_text(RichText::new(&current_name).size(15.0))
                                    .width(ui.available_width())
                                    .show_ui(ui, |ui| {
                                        for (i, name) in self.ollama_models.iter().enumerate() {
                                            let selected = i == self.ollama_selected_index;
                                            if ui
                                                .selectable_label(
                                                    selected,
                                                    RichText::new(name).size(15.0),
                                                )
                                                .clicked()
                                            {
                                                self.ollama_selected_index = i;
                                            }
                                        }
                                    });
                            }

                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(
                                    "Después de transcribir, Ollama corregirá ortografía,\n\
                                 puntuación y errores de reconocimiento.",
                                )
                                .size(12.0)
                                .color(TEXT_MUTED),
                            );
                            ui.add_space(8.0);
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("↻  Actualizar modelos").size(14.0),
                                    )
                                    .fill(Color32::from_rgb(35, 45, 60))
                                    .stroke(Stroke::new(1.0, BORDER))
                                    .rounding(6.0),
                                )
                                .clicked()
                            {
                                self.ollama_models = ollama::list_models();
                                self.ollama_selected_index = self
                                    .ollama_selected_index
                                    .min(self.ollama_models.len().saturating_sub(1));
                            }
                        }
                    }

                    ui.add_space(24.0);
                    ui.separator();
                    ui.add_space(16.0);

                    // ── Section: Resúmenes ──────────────────────────────────────────
                    section_header(ui, "✨  Configuración de resúmenes");
                    ui.add_space(12.0);

                    if self.ollama_available && self.ollama_enabled {
                        // Summary model
                        ui.label(
                            RichText::new("Modelo para resúmenes:")
                                .size(13.0)
                                .color(TEXT_DIM),
                        );
                        ui.add_space(4.0);
                        if !self.ollama_models.is_empty() {
                            let current_summary_model = &self.settings.summary_model;
                            let summary_idx = self
                                .ollama_models
                                .iter()
                                .position(|m| m == current_summary_model)
                                .unwrap_or(0);
                            egui::ComboBox::from_id_source("summary_model_combo")
                                .selected_text(RichText::new(current_summary_model).size(14.0))
                                .width(ui.available_width())
                                .show_ui(ui, |ui| {
                                    for (i, name) in self.ollama_models.iter().enumerate() {
                                        let selected = i == summary_idx;
                                        if ui
                                            .selectable_label(
                                                selected,
                                                RichText::new(name).size(14.0),
                                            )
                                            .clicked()
                                        {
                                            self.settings.summary_model = name.clone();
                                        }
                                    }
                                });
                        }

                        ui.add_space(12.0);

                        // Streaming mode
                        ui.label(RichText::new("Modo streaming:").size(13.0).color(TEXT_DIM));
                        ui.add_space(4.0);
                        egui::ComboBox::from_id_source("stream_mode_combo")
                            .selected_text(
                                RichText::new(&self.settings.summary_stream_mode).size(14.0),
                            )
                            .width(150.0)
                            .show_ui(ui, |ui| {
                                for mode in ["auto", "stream", "non_stream"] {
                                    let selected = self.settings.summary_stream_mode == mode;
                                    if ui
                                        .selectable_label(selected, RichText::new(mode).size(14.0))
                                        .clicked()
                                    {
                                        self.settings.summary_stream_mode = mode.to_string();
                                    }
                                }
                            });

                        ui.add_space(12.0);

                        // Thinking policy
                        ui.label(
                            RichText::new("Política de thinking:")
                                .size(13.0)
                                .color(TEXT_DIM),
                        );
                        ui.add_space(4.0);
                        egui::ComboBox::from_id_source("thinking_policy_combo")
                            .selected_text(
                                RichText::new(&self.settings.summary_thinking_policy).size(14.0),
                            )
                            .width(200.0)
                            .show_ui(ui, |ui| {
                                for policy in ["hide_thinking", "store_but_hide", "show_for_debug"]
                                {
                                    let selected = self.settings.summary_thinking_policy == policy;
                                    let label = match policy {
                                        "hide_thinking" => "Ocultar siempre",
                                        "store_but_hide" => "Guardar pero ocultar",
                                        "show_for_debug" => "Mostrar (debug)",
                                        _ => policy,
                                    };
                                    if ui
                                        .selectable_label(selected, RichText::new(label).size(14.0))
                                        .clicked()
                                    {
                                        self.settings.summary_thinking_policy = policy.to_string();
                                    }
                                }
                            });
                    } else {
                        ui.label(
                            RichText::new("Activa Ollama para configurar resúmenes")
                                .size(12.0)
                                .color(TEXT_DIM),
                        );
                    }

                    ui.add_space(24.0);
                    ui.separator();
                    ui.add_space(16.0);

                    // ── Section: Custom Prompts ─────────────────────────────────────
                    section_header(ui, "📝  Prompts personalizados");
                    ui.add_space(12.0);

                    ui.label(
                        RichText::new("Prompt personalizado (Ejecutivo):")
                            .size(12.0)
                            .color(TEXT_DIM),
                    );
                    ui.add_sized(
                        egui::vec2(ui.available_width(), 60.0),
                        egui::TextEdit::multiline(&mut self.settings.custom_prompt_executive)
                            .font(FontId::proportional(12.0))
                            .hint_text("Ej: Enfócate en los puntos clave de la reunión..."),
                    );

                    ui.add_space(8.0);

                    ui.label(
                        RichText::new("Prompt personalizado (Tareas):")
                            .size(12.0)
                            .color(TEXT_DIM),
                    );
                    ui.add_sized(
                        egui::vec2(ui.available_width(), 60.0),
                        egui::TextEdit::multiline(&mut self.settings.custom_prompt_tasks)
                            .font(FontId::proportional(12.0))
                            .hint_text("Ej: Lista solo las tareas asignadas..."),
                    );

                    ui.add_space(8.0);

                    ui.label(
                        RichText::new("Prompt personalizado (Decisiones):")
                            .size(12.0)
                            .color(TEXT_DIM),
                    );
                    ui.add_sized(
                        egui::vec2(ui.available_width(), 60.0),
                        egui::TextEdit::multiline(&mut self.settings.custom_prompt_decisions)
                            .font(FontId::proportional(12.0))
                            .hint_text("Ej: Extrae solo las decisiones tomadas..."),
                    );

                    ui.add_space(24.0);
                    ui.separator();
                    ui.add_space(16.0);

                    // ── Section: Idioma ────────────────────────────────────────────
                    section_header(ui, "🌐  Idioma por defecto");
                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Idioma:").size(13.0).color(TEXT_DIM));
                        ui.add_space(8.0);
                        egui::ComboBox::from_id_source("language_combo")
                            .selected_text(
                                RichText::new(&self.settings.language_default).size(14.0),
                            )
                            .width(120.0)
                            .show_ui(ui, |ui| {
                                for lang in ["es", "en"] {
                                    let selected = self.settings.language_default == lang;
                                    if ui
                                        .selectable_label(selected, RichText::new(lang).size(14.0))
                                        .clicked()
                                    {
                                        self.settings.language_default = lang.to_string();
                                    }
                                }
                            });
                    });

                    ui.add_space(24.0);
                    ui.separator();
                    ui.add_space(16.0);

                    // ── Section: Hotkeys ────────────────────────────────────────────
                    section_header(ui, "⌨️  Atajos de teclado");
                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Iniciar/Detener:").size(13.0).color(TEXT_DIM));
                        ui.add_space(8.0);
                        ui.add_sized(
                            egui::vec2(180.0, 24.0),
                            egui::TextEdit::singleline(&mut self.settings.hotkey_start_stop)
                                .font(FontId::proportional(14.0)),
                        );
                    });

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Highlight:").size(13.0).color(TEXT_DIM));
                        ui.add_space(8.0);
                        ui.add_sized(
                            egui::vec2(180.0, 24.0),
                            egui::TextEdit::singleline(&mut self.settings.hotkey_highlight)
                                .font(FontId::proportional(14.0)),
                        );
                    });

                    ui.add_space(24.0);
                    ui.separator();
                    ui.add_space(16.0);

                    // ── Section: Carpeta de grabaciones ────────────────────────────
                    section_header(ui, "📁  Carpeta de grabaciones");
                    ui.add_space(12.0);

                    ui.add(
                        egui::TextEdit::singleline(&mut self.settings.recordings_folder)
                            .desired_width(f32::INFINITY)
                            .font(FontId::proportional(15.0)),
                    );

                    ui.add_space(20.0);

                    // ── Save button ────────────────────────────────────────────────
                    ui.vertical_centered(|ui| {
                        let save_btn = egui::Button::new(
                            RichText::new("  Guardar configuración  ")
                                .size(16.0)
                                .color(Color32::WHITE),
                        )
                        .fill(ACCENT_BLUE)
                        .rounding(8.0);

                        if ui
                            .add(save_btn)
                            .on_hover_text("Guardar la configuración actual")
                            .clicked()
                        {
                            self.settings.input_device_id = self
                                .input_devices
                                .get(self.selected_input_index)
                                .map(|(_, id)| id.clone());
                            self.settings.output_device_id = self
                                .output_devices
                                .get(self.selected_output_index)
                                .map(|(_, id)| id.clone());

                            if let Some((_, path)) =
                                self.available_models.get(self.selected_model_index)
                            {
                                self.settings.whisper_model = path.clone();
                            }

                            self.settings.ollama_enabled = self.ollama_enabled;
                            self.settings.ollama_model = self
                                .ollama_models
                                .get(self.ollama_selected_index)
                                .cloned()
                                .unwrap_or_default();

                            self.model_changed = false;

                            if let Err(e) = self.settings.save() {
                                self.config_save_notification =
                                    Some((format!("Error: {}", e), true));
                            } else {
                                self.config_save_notification =
                                    Some(("✅ Configuración guardada".to_string(), false));
                            }
                        }
                    });

                    // Show notification
                    if let Some((msg, is_error)) = &self.config_save_notification {
                        ui.add_space(8.0);
                        ui.label(RichText::new(msg).size(14.0).color(if *is_error {
                            ACCENT_RED
                        } else {
                            ACCENT_GREEN
                        }));
                    }

                    ui.add_space(16.0);
                });
            });
    }

    // ── About tab ────────────────────────────────────────────────────────────

    fn show_about_tab(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_source("about_scroll")
            .show(ui, |ui| {
                ui.add_space(24.0);

                // ── Logo / title block ────────────────────────────────────────
                ui.vertical_centered(|ui| {
                    // App name with gradient-like effect using two labels
                    ui.label(
                        RichText::new("MeetWhisperer")
                            .size(32.0)
                            .strong()
                            .color(TEXT_PRIMARY),
                    );
                    ui.add_space(6.0);

                    // Version badge
                    let version = format!("  v{}  ", env!("CARGO_PKG_VERSION"));
                    let (badge_rect, _) = ui.allocate_exact_size(
                        egui::vec2(80.0, 26.0),
                        Sense::hover(),
                    );
                    ui.painter().rect_filled(badge_rect, 13.0, ACCENT_BLUE);
                    ui.painter().text(
                        badge_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &version,
                        FontId::proportional(13.0),
                        Color32::WHITE,
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Transcripción local de audio · 100% offline")
                            .size(14.0)
                            .color(TEXT_DIM),
                    );
                });

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(16.0);

                // ── Feature cards ─────────────────────────────────────────────
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("Características").size(16.0).color(TEXT_DIM),
                    );
                });
                ui.add_space(10.0);

                let features = [
                    ("🎙", "Captura de audio del sistema", "Graba cualquier sonido que reproduzca tu computador via PulseAudio / PipeWire"),
                    ("🤖", "Whisper offline", "Transcripción local con modelos GGML — ningún dato sale de tu máquina"),
                    ("✨", "Mejora con Ollama", "Corrección opcional de ortografía y puntuación usando LLMs locales"),
                    ("🗄", "Historial SQLite", "Registro de grabaciones con fecha, duración y acceso rápido a los archivos"),
                ];

                for (icon, title, desc) in &features {
                    about_feature_card(ui, icon, title, desc);
                    ui.add_space(6.0);
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(16.0);

                // ── Tech stack ────────────────────────────────────────────────
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("Stack tecnológico").size(15.0).color(TEXT_DIM));
                    ui.add_space(10.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
                        for (label, color) in [
                            ("Rust", Color32::from_rgb(222, 90, 40)),
                            ("egui 0.27", ACCENT_BLUE),
                            ("whisper-rs 0.15", Color32::from_rgb(34, 160, 100)),
                            ("PulseAudio", Color32::from_rgb(140, 90, 220)),
                            ("Ollama", Color32::from_rgb(200, 160, 40)),
                            ("SQLite", Color32::from_rgb(80, 160, 200)),
                        ] {
                            tech_badge(ui, label, color);
                        }
                    });
                });

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(16.0);

                // ── Author block ──────────────────────────────────────────────
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("Desarrollado por")
                            .size(13.0)
                            .color(TEXT_MUTED),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Gustavo Gutiérrez")
                            .size(18.0)
                            .strong()
                            .color(TEXT_PRIMARY),
                    );
                    ui.label(
                        RichText::new("Bogotá, Colombia")
                            .size(13.0)
                            .color(TEXT_DIM),
                    );
                });

                ui.add_space(20.0);
            });
    }
}

// ── Recording row renderer ────────────────────────────────────────────────────

fn show_recording_row(ui: &mut egui::Ui, entry: &RecordingEntry) {
    let frame = egui::Frame::none()
        .fill(BG_CARD)
        .stroke(Stroke::new(1.0, BORDER))
        .rounding(6.0)
        .inner_margin(egui::Margin::symmetric(10.0, 8.0));

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            // Left: filename + metadata
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(&entry.filename)
                        .size(13.0)
                        .color(TEXT_PRIMARY)
                        .strong(),
                );
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&entry.created_at)
                            .size(11.0)
                            .color(TEXT_MUTED),
                    );
                    ui.label(
                        RichText::new(format!("⏱ {}", entry.duration_display()))
                            .size(11.0)
                            .color(TEXT_DIM),
                    );
                    if entry.ollama_used {
                        let model = entry.ollama_model.as_deref().unwrap_or("Ollama");
                        ui.label(
                            RichText::new(format!("✨ {}", model))
                                .size(11.0)
                                .color(ACCENT_PURPLE),
                        );
                    }
                });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let open_btn =
                    egui::Button::new(RichText::new("Abrir").size(12.0).color(ACCENT_BLUE))
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::new(1.0, ACCENT_BLUE))
                        .rounding(4.0);

                if ui.add(open_btn).clicked() {
                    // Open the .txt file with the system's default app
                    let _ = std::process::Command::new("xdg-open")
                        .arg(&entry.filepath)
                        .spawn();
                }
            });
        });
    });
    ui.add_space(2.0);
}

// ── UI helpers ────────────────────────────────────────────────────────────────

fn format_time_simple(secs: f64) -> String {
    let total = secs as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", m, s)
    }
}

/// Format seconds to SRT timestamp: HH:MM:SS,mmm
fn format_srt_timestamp(secs: f64) -> String {
    let total_ms = (secs * 1000.0) as u64;
    let hours = total_ms / 3_600_000;
    let minutes = (total_ms % 3_600_000) / 60_000;
    let seconds = (total_ms % 60_000) / 1_000;
    let millis = total_ms % 1_000;
    format!("{:02}:{:02}:{:02},{:03}", hours, minutes, seconds, millis)
}

/// Format seconds to WebVTT timestamp: HH:MM:SS.mmm
fn format_vtt_timestamp(secs: f64) -> String {
    let total_ms = (secs * 1000.0) as u64;
    let hours = total_ms / 3_600_000;
    let minutes = (total_ms % 3_600_000) / 60_000;
    let seconds = (total_ms % 60_000) / 1_000;
    let millis = total_ms % 1_000;
    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
}

fn section_header(ui: &mut egui::Ui, title: &str) {
    let frame = egui::Frame::none()
        .fill(BG_PANEL)
        .stroke(Stroke::new(1.0, BORDER))
        .rounding(6.0)
        .inner_margin(egui::Margin::symmetric(10.0, 6.0));
    frame.show(ui, |ui| {
        ui.label(RichText::new(title).size(15.0).strong().color(TEXT_PRIMARY));
    });
    ui.add_space(2.0);
}

fn status_badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("●").size(13.0).color(color));
        ui.label(RichText::new(text).size(14.0).color(TEXT_PRIMARY));
    });
}

fn about_feature_card(ui: &mut egui::Ui, icon: &str, title: &str, desc: &str) {
    let frame = egui::Frame::none()
        .fill(BG_CARD)
        .stroke(Stroke::new(1.0, BORDER))
        .rounding(8.0)
        .inner_margin(egui::Margin::symmetric(14.0, 10.0));
    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(icon).size(24.0));
            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.label(RichText::new(title).size(15.0).strong().color(TEXT_PRIMARY));
                ui.label(RichText::new(desc).size(12.0).color(TEXT_DIM));
            });
        });
    });
}

fn tech_badge(ui: &mut egui::Ui, label: &str, color: Color32) {
    let text = RichText::new(label).size(12.0).color(Color32::WHITE);
    let btn = egui::Button::new(text)
        .fill(color.gamma_multiply(0.6))
        .stroke(Stroke::new(1.0, color))
        .rounding(12.0);
    ui.add(btn);
}

/// Returns a formatted local datetime string (ISO 8601 without timezone).
fn chrono_local_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Manual UTC-5 (Colombia, no DST) — safe fallback without chrono dep
    let offset_secs: i64 = -5 * 3600;
    let local_secs = secs as i64 + offset_secs;
    let s = local_secs.unsigned_abs();
    let sec = s % 60;
    let min = (s / 60) % 60;
    let hour = (s / 3600) % 24;
    let days = s / 86400;
    // Approximate Gregorian date from epoch days
    let (y, mo, d) = days_to_ymd(days);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        y, mo, d, hour, min, sec
    )
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Gregorian calendar algorithm (valid for dates after 1970-01-01)
    let z = days + 719468;
    let era = z / 146097;
    let doe = z % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    (y, mo, d)
}

// ── Waveform with gradient ────────────────────────────────────────────────────

fn grad_color(t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    for i in 0..GRAD.len() - 1 {
        let (t0, c0) = GRAD[i];
        let (t1, c1) = GRAD[i + 1];
        if t <= t1 {
            let f = (t - t0) / (t1 - t0);
            return Color32::from_rgb(
                lerp_u8(c0.r(), c1.r(), f),
                lerp_u8(c0.g(), c1.g(), f),
                lerp_u8(c0.b(), c1.b(), f),
            );
        }
    }
    GRAD.last().unwrap().1
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

fn draw_waveform_gradient(painter: &Painter, rect: Rect, samples: &[f32]) {
    let width = rect.width() as usize;
    if width == 0 || samples.is_empty() {
        return;
    }

    let mid_y = rect.center().y;
    let half_h = rect.height() * 0.45;

    let peak = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
    let scale = if peak > 0.005 { 0.8 / peak } else { 1.0 };

    let step = (samples.len() as f32 / width as f32).max(1.0);

    for col in 0..width {
        let start = (col as f32 * step) as usize;
        let end = ((col as f32 + 1.0) * step) as usize;
        let end = end.min(samples.len());
        if start >= end {
            break;
        }

        let chunk = &samples[start..end];
        let max_s = chunk.iter().cloned().fold(f32::NEG_INFINITY, f32::max) * scale;
        let min_s = chunk.iter().cloned().fold(f32::INFINITY, f32::min) * scale;

        let top_y = mid_y - max_s.clamp(-1.0, 1.0) * half_h;
        let bot_y = mid_y - min_s.clamp(-1.0, 1.0) * half_h;

        let t = col as f32 / width as f32;
        let color = grad_color(t);

        let x = rect.left() + col as f32;
        let draw_top = top_y.min(mid_y - 1.5);
        let draw_bot = bot_y.max(mid_y + 1.5);

        painter.line_segment(
            [Pos2::new(x, draw_top), Pos2::new(x, draw_bot)],
            Stroke::new(1.5, color),
        );
    }
}
