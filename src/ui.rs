//! UI module for Scrivano desktop application.

use std::{
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use eframe::egui::{self, Color32, FontId, RichText, Stroke};
// Icon library: Phosphor Icons (https://phosphoricons.com/)
// This library provides high-quality SVG icons for the UI.
// Browse all available icons at: https://phosphoricons.com/
use egui_phosphor::regular as icons;
use hound::{WavSpec, WavWriter};
use whisper_rs::WhisperContext;

use crate::audio::{spawn_system_audio_recorder, ChunkSink};
use crate::audio_chunker::AudioChunker;
use crate::audio_devices::{get_input_devices, get_output_devices, scan_models, AppSettings};
use crate::database::{Database, NewRecording, RecordingEntry, Summary};
use crate::ollama;
use crate::ollama_block;
use crate::recording_session::{ManifestEntry, RecordingSession};
use crate::transcription::{transcribe_with_segments, TranscriptionLanguage};
use std::collections::HashMap;

#[path = "ui/about.rs"]
mod about;
#[path = "ui/components.rs"]
mod components;
#[path = "ui/recording.rs"]
mod recording;
#[path = "ui/settings.rs"]
mod settings;
#[path = "ui/spectrum.rs"]
mod spectrum;
#[path = "ui/theme.rs"]
mod theme;

use theme::*;

// ── Theme aliases (for incremental migration) ───────────────────────────────
const BG_DARK: Color32 = BG_VOID;
const BG_PANEL: Color32 = BG_NEBULA;
const BG_CARD: Color32 = BG_STARDUST;
const ACCENT_RED: Color32 = ACCENT_CRIMSON;
const ACCENT_RED_HOVER: Color32 = ACCENT_CRIMSON_HOVER;
const ACCENT_GREEN: Color32 = ACCENT_EMERALD;
const ACCENT_GREEN_HOVER: Color32 = ACCENT_EMERALD_HOVER;
const ACCENT_BLUE: Color32 = ACCENT_CYAN;
const TEXT_PRIMARY: Color32 = TEXT_STARLIGHT;
const TEXT_DIM: Color32 = TEXT_MOON;
const TEXT_MUTED: Color32 = TEXT_DUST;
const BORDER: Color32 = BG_ECLIPSE;
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
const ICON_INFO: &str = icons::INFO;
const ICON_TAG: &str = icons::TAG;

// ── App struct ────────────────────────────────────────────────────────────────

#[derive(PartialEq, Clone)]
enum Tab {
    Recording,
    Settings,
    About,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum SettingsSection {
    Audio,
    Transcription,
    Ollama,
    Summaries,
    Prompts,
    System,
}

type PendingHighlight = (f64, Option<String>);
type PendingHighlights = Arc<Mutex<Vec<PendingHighlight>>>;

pub struct App {
    pub recording: Arc<AtomicBool>,
    pub audio_buffer: Arc<Mutex<Vec<f32>>>,
    pub waveform_buffer: Arc<Mutex<Vec<f32>>>,
    pub transcript: Arc<Mutex<String>>,
    transcript_edit: String,
    whisper_ctx: Arc<WhisperContext>,
    active_tab: Tab,
    settings_section: SettingsSection,
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
    pending_highlights: PendingHighlights,
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
    // ── Stopping delay ───────────────────────────────────────────────────────
    is_stopping: bool, // true when waiting to stop recording
    stop_requested_time: Option<std::time::Instant>, // when stop was requested
    chunk_ui_state: Arc<Mutex<ChunkUiState>>,
    chunk_pipeline: Option<Arc<Mutex<ChunkPipeline>>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ChunkUiState {
    active_chunk_index: u32,
    closed_chunks: u32,
    successful_chunks: u32,
    failed_chunks: Vec<u32>,
    retry_target: Option<u32>,
}

impl ChunkUiState {
    fn on_chunk_rotated(&mut self, chunk_index: u32) {
        self.active_chunk_index = chunk_index;
    }

    fn on_chunk_closed(&mut self, chunk_index: u32, success: bool) {
        self.closed_chunks += 1;
        if success {
            self.successful_chunks += 1;
            self.failed_chunks.retain(|c| *c != chunk_index);
            if self.retry_target == Some(chunk_index) {
                self.retry_target = None;
            }
        } else if !self.failed_chunks.contains(&chunk_index) {
            self.failed_chunks.push(chunk_index);
        }
    }

    fn mark_retry_requested(&mut self, chunk_index: u32) {
        if self.failed_chunks.contains(&chunk_index) {
            self.retry_target = Some(chunk_index);
        }
    }

    fn progress_percent(&self) -> i32 {
        if self.closed_chunks == 0 {
            return 0;
        }
        ((self.successful_chunks * 100) / self.closed_chunks) as i32
    }
}

struct ChunkPipeline {
    session: RecordingSession,
    chunker: AudioChunker,
    ui_state: Arc<Mutex<ChunkUiState>>,
}

impl ChunkPipeline {
    fn new(session: RecordingSession, ui_state: Arc<Mutex<ChunkUiState>>) -> Self {
        Self {
            chunker: AudioChunker::new(&session.session_dir, 16_000, 25, 5),
            session,
            ui_state,
        }
    }

    fn push_samples(&mut self, samples: &[f32]) {
        let closed = match self.chunker.push_samples(samples) {
            Ok(closed) => closed,
            Err(_) => return,
        };

        for chunk in closed {
            let filename = chunk
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string();

            let entry = ManifestEntry {
                chunk_index: chunk.chunk_index,
                filename,
                start_sample: chunk.start_sample,
                end_sample: chunk.end_sample,
            };
            let append_ok = self.session.append(&entry).is_ok();

            let mut ui_state = self.ui_state.lock().unwrap();
            ui_state.on_chunk_rotated(chunk.chunk_index + 1);
            ui_state.on_chunk_closed(chunk.chunk_index, append_ok);
        }
    }
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
            .join("Scrivano")
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
            settings_section: SettingsSection::Audio,
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
            is_stopping: false,
            stop_requested_time: None,
            chunk_ui_state: Arc::new(Mutex::new(ChunkUiState::default())),
            chunk_pipeline: None,
        }
    }

    fn reload_recordings(&mut self) {
        if let Some(db) = &self.db {
            self.recordings = db.list_recordings().unwrap_or_default();
        }
        self.recordings_dirty.store(false, Ordering::SeqCst);
    }

    fn refresh_audio_devices(&mut self) {
        let current_input_id = self
            .input_devices
            .get(self.selected_input_index)
            .map(|(_, id)| id.clone())
            .or_else(|| self.settings.input_device_id.clone());

        let current_output_id = self
            .output_devices
            .get(self.selected_output_index)
            .map(|(_, id)| id.clone())
            .or_else(|| self.settings.output_device_id.clone());

        self.input_devices = get_input_devices()
            .into_iter()
            .map(|d| (d.name, d.id))
            .collect();

        self.output_devices = get_output_devices()
            .into_iter()
            .map(|d| (d.name, d.id))
            .collect();

        self.selected_input_index = current_input_id
            .as_ref()
            .and_then(|id| self.input_devices.iter().position(|(_, did)| did == id))
            .unwrap_or(0)
            .min(self.input_devices.len().saturating_sub(1));

        self.selected_output_index = current_output_id
            .as_ref()
            .and_then(|id| self.output_devices.iter().position(|(_, did)| did == id))
            .unwrap_or(0)
            .min(self.output_devices.len().saturating_sub(1));
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

        components::apply_nebula_theme(ctx);

        // Keep repainting at 60fps while recording / processing / playing audio for smooth animations
        // or when stopping to show countdown
        #[cfg(feature = "audio-playback")]
        let is_playing_audio = self
            .audio_player
            .as_ref()
            .map(|p| p.is_playing())
            .unwrap_or(false);
        #[cfg(not(feature = "audio-playback"))]
        let is_playing_audio = false;

        let needs_smooth_animation =
            self.recording.load(Ordering::SeqCst) || is_playing_audio || self.is_stopping;
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
            eprintln!("[ui] ===== SUMMARY GENERATION COMPLETE FLAG DETECTED =====");
            self.summary_generation_complete
                .store(false, Ordering::SeqCst);
            // Clear summaries cache to force reload on next view
            eprintln!("[ui] Clearing summaries cache...");
            self.summaries_cache.clear();
            eprintln!("[ui] Resúmenes recargados tras generación");
            // Request immediate repaint to show updated summaries
            ctx.request_repaint();
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
                    (Tab::About, "Acerca de", ICON_INFO),
                ] {
                    let active = self.active_tab == tab;
                    let (bg, fg) = if active {
                        (ACCENT_BLUE, Color32::WHITE)
                    } else {
                        (Color32::TRANSPARENT, TEXT_DIM)
                    };
                    let btn = egui::Button::new(
                        RichText::new(format!("{} {}", icon, label))
                            .size(FONT_BODY)
                            .color(fg),
                    )
                    .fill(bg)
                    .stroke(if active {
                        Stroke::new(0.0, Color32::TRANSPARENT)
                    } else {
                        Stroke::new(1.0, BORDER)
                    })
                    .rounding(ROUNDING_SMALL);
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
    // Recording tab implementation moved to src/ui/recording.rs

    fn show_recording_row_expanded(&mut self, ui: &mut egui::Ui, entry: &RecordingEntry) {
        // Load summaries if not cached
        if !self.summaries_cache.contains_key(&entry.id) {
            if let Some(db) = &self.db {
                match db.get_summaries_by_recording(entry.id) {
                    Ok(summaries) => {
                        self.summaries_cache.insert(entry.id, summaries);
                    }
                    Err(e) => {
                        eprintln!(
                            "[ui] Error cargando resúmenes para recording {}: {}",
                            entry.id, e
                        );
                    }
                }

                ui.add_space(8.0);
                ui.label(
                    RichText::new("El modelo generará resúmenes automáticamente.")
                        .size(11.0)
                        .color(TEXT_MUTED),
                );
            } else {
                eprintln!(
                    "[ui] No hay conexión a DB para cargar resúmenes (recording {})",
                    entry.id
                );
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
                                        RichText::new(format!("{} {}", ICON_TAG, t))
                                            .size(10.0)
                                            .color(ACCENT_BLUE),
                                    );
                                }
                            }
                        }

                        if entry.ollama_used {
                            ui.add_space(6.0);
                            ui.label(RichText::new(ICON_MAGIC).size(11.0).color(ACCENT_PURPLE));
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
                            if ui.button(format!("{} WAV", ICON_AUDIO)).clicked() {
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

                // Show summaries section
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                // Check if any summary generation failed recently
                let has_error = summaries.iter().any(|s| s.content.starts_with("ERROR:"));
                let has_empty = summaries.iter().any(|s| s.content.trim().is_empty());

                if has_error {
                    ui.label(RichText::new("❌ Error generando resúmenes - revisa la terminal").size(12.0).color(ACCENT_RED));
                } else if has_empty && !summaries.is_empty() {
                    ui.label(RichText::new("⚠️ Algunos resúmenes están vacíos").size(12.0).color(ACCENT_RED));
                }

                if !summaries.is_empty() {
                    ui.label(RichText::new("✨ Resúmenes generados").size(13.0).color(TEXT_PRIMARY).strong());
                    ui.add_space(8.0);

                    for summary in &summaries {
                        // Skip empty summaries but show error indicator
                        if summary.content.trim().is_empty() || summary.content.starts_with("ERROR:") {
                            let error_msg = if summary.content.starts_with("ERROR:") {
                                summary.content.clone()
                            } else {
                                format!("ERROR: Resumen '{}' vacío", summary.template)
                            };
                            ui.label(RichText::new(&error_msg).size(11.0).color(ACCENT_RED));
                            continue;
                        }
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
                            if summary.is_thinking_model
                                && self.settings.summary_thinking_policy == "show_for_debug"
                            {
                                if let Some(raw_thinking) = &summary.raw_thinking {
                                    let mut debug_thinking = raw_thinking.clone();
                                    ui.label(
                                        RichText::new("🧠 Proceso de thinking:")
                                            .size(11.0)
                                            .color(TEXT_DIM),
                                    );
                                    ui.add_sized(
                                        egui::vec2(ui.available_width(), 80.0),
                                        egui::TextEdit::multiline(&mut debug_thinking)
                                            .font(FontId::proportional(10.0))
                                            .text_color(Color32::from_rgb(100, 120, 140)),
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
                            let is_playing = player.map(|p| p.is_playing()).unwrap_or(false);
                            let is_paused = player.map(|p| p.is_paused()).unwrap_or(false);
                            let is_current_item = self.current_playing_id == Some(entry.id);

                            // Only calculate elapsed time for the currently playing item
                            let elapsed = if is_current_item && (is_playing || is_paused) {
                                player.map(|p| p.get_elapsed_secs()).unwrap_or(0.0)
                            } else {
                                0.0
                            };
                            let total = entry.duration_secs;

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
                            match player.play(&wav_path) {
                                Ok(_) => {
                                    self.current_playing_id = Some(entry.id);
                                }
                                Err(e) => {
                                    eprintln!("[playback] Error starting playback: {e}");
                                }
                            }
                        } else {
                            eprintln!("[playback] WAV file not found: {}", wav_path);
                        }
                    }
                } else {
                    // Play new file - use .wav file
                    if std::path::Path::new(&wav_path).exists() {
                        match player.play(&wav_path) {
                            Ok(_) => {
                                self.current_playing_id = Some(entry.id);
                            }
                            Err(e) => {
                                eprintln!("[playback] Error starting playback: {e}");
                            }
                        }
                    } else {
                        eprintln!("[playback] WAV file not found: {}", wav_path);
                    }
                }
            } else {
                eprintln!("[playback] Audio player backend is not available");
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
            .join("Scrivano")
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
        eprintln!("[summary] ===== INICIANDO GENERACIÓN =====");
        eprintln!(
            "[summary] Recording ID: {}, Template: {}",
            recording_id, template
        );

        if !self.ollama_available {
            eprintln!("[summary] ERROR: Ollama no está disponible");
            return;
        }

        let entry = self
            .recordings
            .iter()
            .find(|e| e.id == recording_id)
            .cloned();
        if let Some(entry) = entry {
            eprintln!(
                "[summary] Found entry: {} ({})",
                entry.filename, entry.filepath
            );
            let transcript = std::fs::read_to_string(&entry.filepath).unwrap_or_default();
            eprintln!("[summary] Transcript length: {} chars", transcript.len());
            if transcript.is_empty() {
                eprintln!("[summary] ERROR: Transcripción vacía");
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
                    .or_default()
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
                .join("Scrivano")
                .join("recordings.db");
            let generating_clone = self.generating_summaries.clone();
            let complete_flag = self.summary_generation_complete.clone();

            std::thread::spawn(move || {
                let client = crate::ollama::OllamaClient::new("localhost", 11434);

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
                        eprintln!("[summary] ===== RESUMEN GENERADO =====");
                        eprintln!(
                            "[summary] Template: {}, Recording: {}",
                            template_string, recording_id_copy
                        );
                        eprintln!(
                            "[summary] Content length: {} chars",
                            summary_result.content.len()
                        );
                        eprintln!(
                            "[summary] Content preview: '{}'",
                            summary_result
                                .content
                                .chars()
                                .take(100)
                                .collect::<String>()
                                .replace('\n', " ")
                        );
                        eprintln!(
                            "[summary] Is thinking model: {}, Has raw_thinking: {}",
                            summary_result.is_thinking_model,
                            summary_result.raw_thinking.is_some()
                        );

                        if summary_result.content.is_empty() {
                            eprintln!("[summary] WARNING: Content is EMPTY!");
                        }

                        // Save summary to database
                        eprintln!("[summary] Opening database at {:?}", db_path);
                        if let Ok(db) = Database::open(&db_path) {
                            eprintln!("[summary] Database opened successfully");
                            match db.insert_summary(
                                recording_id_copy,
                                &template_string,
                                &summary_result.content,
                                Some(&summary_result.model_name),
                                summary_result.is_thinking_model,
                                summary_result.raw_thinking.as_deref(),
                            ) {
                                Ok(id) => {
                                    eprintln!("[summary] Guardado en BD exitosamente, ID: {}", id);
                                }
                                Err(e) => {
                                    eprintln!("[summary] ERROR guardando en BD: {}", e);
                                }
                            }
                        } else {
                            eprintln!("[summary] ERROR: No se pudo abrir la base de datos");
                        }
                    }
                    Err(e) => {
                        eprintln!("[summary] ===== ERROR GENERANDO RESUMEN =====");
                        eprintln!("[summary] Error: {:?}", e);
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
                eprintln!("[summary] Setting complete flag to trigger UI reload");
                complete_flag.store(true, Ordering::SeqCst);
                eprintln!("[summary] ===== THREAD COMPLETE =====");
            });
        } else {
            eprintln!(
                "[summary] ERROR: No se encontró la grabación con ID {}",
                recording_id
            );
        }
    }

    fn start_recording(&mut self) {
        self.audio_buffer.lock().unwrap().clear();
        self.waveform_buffer.lock().unwrap().clear();
        self.waveform_buffer.lock().unwrap().clear();
        self.pending_highlights.lock().unwrap().clear();
        *self.chunk_ui_state.lock().unwrap() = ChunkUiState::default();
        self.transcript_edit = "Grabando...".to_string();
        *self.transcript.lock().unwrap() = "Grabando...".to_string();
        self.recording.store(true, Ordering::SeqCst);
        self.recording_start_timestamp = Some(chrono_local_now());

        let output_monitor = self
            .output_devices
            .get(self.selected_output_index)
            .map(|(_, id)| id.clone())
            .unwrap_or_default();

        let input_source = self
            .input_devices
            .get(self.selected_input_index)
            .map(|(_, id)| id.clone())
            .unwrap_or_default();

        let source_name = choose_capture_source(&input_source, &output_monitor);

        let chunk_sessions_root =
            std::path::PathBuf::from(&self.settings.recordings_folder).join("chunk_sessions");
        let session_seed = format!(
            "{}-{}",
            self.recording_start_timestamp
                .as_deref()
                .unwrap_or("recording-session"),
            std::process::id()
        );

        let chunk_sink: Option<ChunkSink> =
            match RecordingSession::open(&chunk_sessions_root, &session_seed) {
                Ok(session) => {
                    let pipeline = Arc::new(Mutex::new(ChunkPipeline::new(
                        session,
                        self.chunk_ui_state.clone(),
                    )));
                    self.chunk_pipeline = Some(pipeline.clone());

                    Some(Arc::new(move |samples: &[f32]| {
                        if let Ok(mut guard) = pipeline.lock() {
                            guard.push_samples(samples);
                        }
                    }))
                }
                Err(_) => {
                    self.chunk_pipeline = None;
                    None
                }
            };

        eprintln!("[ui] Grabando desde: {:?}", source_name);

        spawn_system_audio_recorder(
            self.recording.clone(),
            self.audio_buffer.clone(),
            self.waveform_buffer.clone(),
            source_name,
            chunk_sink,
        );
    }

    fn stop_and_transcribe(&mut self) {
        self.recording.store(false, Ordering::SeqCst);
        if let Some(pipeline) = self.chunk_pipeline.take() {
            if let Ok(guard) = pipeline.lock() {
                let _ = guard.session.finalize();
            }
        }

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
            .join("Scrivano")
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
                *transcript.lock().unwrap() =
                    format!("{}\n\n[Mejorando transcripción por bloques…]", raw_text);

                let op = ollama_progress.clone();
                let improved = match ollama_block::improve_transcript_blocks(
                    &ollama_model,
                    &raw_text,
                    &segments,
                    5, // 5-minute blocks
                    move |block_idx, total_blocks, block_pct| {
                        // Map block progress to overall percentage for UI
                        let overall: i32 = if total_blocks > 0 {
                            ((block_idx as i32) * 100 / total_blocks as i32)
                                + (block_pct / total_blocks as i32)
                        } else {
                            100
                        };
                        op.store(overall.clamp(0, 100), Ordering::SeqCst);
                    },
                    None,
                ) {
                    Ok(t) => {
                        eprintln!("[ollama_block] mejora completada ({} chars)", t.len());
                        t
                    }
                    Err(e) => {
                        eprintln!("[ollama_block] error: {}", e);
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
                        let recording_id = db.insert_recording(NewRecording {
                            filename: &filename,
                            filepath: &path,
                            created_at: &now,
                            duration_secs,
                            ollama_used: ollama_model_used.is_some(),
                            ollama_model: ollama_model_used.as_deref(),
                            title: None, // title - will be generated later
                            tags: None,  // tags - will be generated later
                        });

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
                                    "Genera un título corto (máximo 6 palabras) que resuma la siguiente reunión. Solo responde con el título, sin comillas, sin explicaciones:\n\n{}",
                                    &final_text[..final_text.len().min(1500)]
                                );

                                match ollama::generate_text(&ollama_model, &title_prompt, |_| {}) {
                                    Ok(title) => {
                                        let clean_title =
                                            title.trim().replace('"', "").replace('\n', " ");
                                        eprintln!("[ollama] Título generado: {}", clean_title);

                                        // Generate tags
                                        let tags_prompt = format!(
                                            "Genera exactamente 5 palabras clave (tags) separadas por comas para esta reunión. Los tags deben describir: sentimiento, 3 temas principales, tipo de reunión. Solo las 5 palabras separadas por comas, sin explicaciones:\n\n{}",
                                            &final_text[..final_text.len().min(1500)]
                                        );

                                        match ollama::generate_text(
                                            &ollama_model,
                                            &tags_prompt,
                                            |_| {},
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

    // Settings tab implementation moved to src/ui/settings.rs

    // About tab implementation moved to src/ui/about.rs
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

fn choose_capture_source(input_source: &str, output_monitor: &str) -> String {
    let input = input_source.trim();
    let output = output_monitor.trim();

    let input_usable = !input.is_empty() && input != "default" && input != "default.monitor";

    if input_usable {
        return input.to_string();
    }

    if !output.is_empty() {
        return output.to_string();
    }

    "default".to_string()
}

fn settings_field_width(ui: &egui::Ui) -> f32 {
    SETTINGS_FIELD_MAX_WIDTH.min(ui.available_width())
}

fn truncate_ui_text(text: &str, max_chars: usize) -> String {
    let mut chars = text.chars();
    let preview: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{}...", preview)
    } else {
        preview
    }
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

#[cfg(test)]
mod tests {
    use super::{choose_capture_source, ChunkUiState};

    #[test]
    fn choose_capture_source_prefers_input() {
        let selected = choose_capture_source(
            "alsa_input.usb-Mic-00.analog-stereo",
            "alsa_output.pci-0000_00_1f.3.analog-stereo.monitor",
        );
        assert_eq!(selected, "alsa_input.usb-Mic-00.analog-stereo");
    }

    #[test]
    fn choose_capture_source_falls_back_to_output_monitor() {
        let selected = choose_capture_source("default", "alsa_output.pci.monitor");
        assert_eq!(selected, "alsa_output.pci.monitor");
    }

    #[test]
    fn choose_capture_source_defaults_when_empty() {
        let selected = choose_capture_source("", "");
        assert_eq!(selected, "default");
    }

    #[test]
    fn chunk_ui_state_tracks_active_progress_and_failures() {
        let mut state = ChunkUiState::default();

        state.on_chunk_rotated(0);
        state.on_chunk_closed(0, true);
        state.on_chunk_rotated(1);
        state.on_chunk_closed(1, false);

        assert_eq!(state.active_chunk_index, 1);
        assert_eq!(state.closed_chunks, 2);
        assert_eq!(state.successful_chunks, 1);
        assert_eq!(state.failed_chunks, vec![1]);
        assert_eq!(state.progress_percent(), 50);
    }

    #[test]
    fn chunk_ui_state_retry_targets_only_failed_chunk() {
        let mut state = ChunkUiState::default();
        state.on_chunk_closed(2, false);

        assert_eq!(state.retry_target, None);
        state.mark_retry_requested(2);
        assert_eq!(state.retry_target, Some(2));

        state.on_chunk_closed(2, true);
        assert!(!state.failed_chunks.contains(&2));
        assert_eq!(state.retry_target, None);
    }
}
