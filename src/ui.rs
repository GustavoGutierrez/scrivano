//! UI module for MeetWhisperer desktop application.

use std::{
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use eframe::egui::{self, Color32, FontId, Painter, Pos2, Rect, RichText, Sense, Stroke};
use whisper_rs::WhisperContext;

use crate::audio::spawn_system_audio_recorder;
use crate::audio_devices::{get_input_devices, get_output_devices, scan_models, AppSettings};
use crate::database::{Database, RecordingEntry};
use crate::ollama;
use crate::transcription::transcribe;

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
    last_recording_duration: f64,
    // ── Database & history ────────────────────────────────────────────────────
    db: Option<Database>,
    recordings: Vec<RecordingEntry>,
    recordings_dirty: Arc<AtomicBool>,
    show_recordings: bool,
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
            last_recording_duration: 0.0,
            db,
            recordings,
            recordings_dirty: Arc::new(AtomicBool::new(false)),
            show_recordings: true,
        }
    }

    fn reload_recordings(&mut self) {
        if let Some(db) = &self.db {
            self.recordings = db.list_recordings().unwrap_or_default();
        }
        self.recordings_dirty.store(false, Ordering::SeqCst);
    }
}

// ── eframe::App ───────────────────────────────────────────────────────────────

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.style_mut(|s| {
            s.visuals.panel_fill = BG_DARK;
            s.visuals.window_fill = BG_DARK;
            s.visuals.override_text_color = Some(TEXT_PRIMARY);
            s.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
            s.visuals.widgets.inactive.bg_fill = BG_PANEL;
            s.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
        });

        // Keep repainting at ~30fps while recording / processing
        if self.recording.load(Ordering::SeqCst)
            || self.is_transcribing.load(Ordering::SeqCst)
            || self.is_improving.load(Ordering::SeqCst)
        {
            ctx.request_repaint_after(Duration::from_millis(33));
        }

        // Reload recordings list if dirty
        if self.recordings_dirty.load(Ordering::SeqCst) {
            self.reload_recordings();
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
                    (Tab::Recording, "Grabación", "⏺"),
                    (Tab::Settings, "Configuración", "⚙"),
                    (Tab::About, "Acerca de", "ℹ"),
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

        ui.add_space(16.0);

        // ── Big action button ────────────────────────────────────────────────
        ui.vertical_centered(|ui| {
            if is_busy {
                let (label, pct) = if is_transcribing {
                    let p = if transcribe_pct >= 0 { transcribe_pct } else { 0 };
                    (format!("⏳  Transcribiendo...  {}%", p), p)
                } else {
                    let p = if ollama_pct >= 0 { ollama_pct } else { 0 };
                    (format!("✨  Mejorando con Ollama...  {}%", p), p)
                };
                ui.add_enabled_ui(false, |ui| {
                    ui.add_sized(
                        egui::vec2(320.0, 60.0),
                        egui::Button::new(RichText::new(&label).size(18.0).color(TEXT_DIM))
                            .fill(BG_PANEL),
                    );
                });
                // Progress bar
                ui.add_space(8.0);
                let bar_width = 320.0_f32;
                let bar_height = 6.0_f32;
                let (bar_rect, _) = ui.allocate_exact_size(
                    egui::vec2(bar_width, bar_height),
                    Sense::hover(),
                );
                ui.painter().rect_filled(bar_rect, 3.0, BG_PANEL);
                let fill_w = bar_rect.width() * (pct as f32 / 100.0).clamp(0.0, 1.0);
                if fill_w > 0.0 {
                    let fill_rect = Rect::from_min_size(bar_rect.min, egui::vec2(fill_w, bar_height));
                    let bar_color = if is_transcribing { ACCENT_BLUE } else { ACCENT_PURPLE };
                    ui.painter().rect_filled(fill_rect, 3.0, bar_color);
                }
            } else if is_recording {
                let btn = egui::Button::new(
                    RichText::new("⏹  Detener y transcribir")
                        .size(20.0)
                        .color(Color32::WHITE),
                )
                .fill(ACCENT_RED)
                .stroke(Stroke::new(2.0, ACCENT_RED_HOVER))
                .rounding(10.0);

                if ui.add_sized(egui::vec2(300.0, 60.0), btn).clicked() {
                    self.last_recording_duration = self.recording_start
                        .map(|s| s.elapsed().as_secs_f64())
                        .unwrap_or(0.0);
                    self.stop_and_transcribe();
                }
            } else {
                let btn = egui::Button::new(
                    RichText::new("⏺  Iniciar grabación")
                        .size(20.0)
                        .color(Color32::WHITE),
                )
                .fill(ACCENT_GREEN)
                .stroke(Stroke::new(2.0, ACCENT_GREEN_HOVER))
                .rounding(10.0);

                if ui.add_sized(egui::vec2(300.0, 60.0), btn).clicked() {
                    self.recording_start = Some(std::time::Instant::now());
                    self.start_recording();
                }
            }
        });

        ui.add_space(12.0);

        // ── Waveform / spectrum ──────────────────────────────────────────────
        let wave_height = 110.0;
        let desired_size = egui::vec2(ui.available_width(), wave_height);
        let (response, painter) = ui.allocate_painter(desired_size, Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 8.0, BG_PANEL);

        let samples = self.waveform_buffer.lock().unwrap().clone();

        if is_recording && samples.len() >= 2 {
            draw_waveform_gradient(&painter, rect, &samples);

            let t = ui.input(|i| i.time);
            let blink = ((t * 2.5).sin() * 0.5 + 0.5) as f32;
            let dot = Color32::from_rgba_premultiplied(230, (40.0 + 200.0 * blink) as u8, 40, 255);
            painter.circle_filled(Pos2::new(rect.left() + 14.0, rect.top() + 14.0), 6.0, dot);
            painter.text(
                Pos2::new(rect.left() + 26.0, rect.top() + 7.0),
                egui::Align2::LEFT_TOP,
                "REC",
                FontId::monospace(12.0),
                dot,
            );

            let elapsed = self.recording_start
                .map(|s| s.elapsed().as_secs_f32())
                .unwrap_or(0.0);
            let mins = (elapsed / 60.0) as u32;
            let secs = (elapsed % 60.0) as u32;
            painter.text(
                Pos2::new(rect.right() - 8.0, rect.top() + 7.0),
                egui::Align2::RIGHT_TOP,
                format!("{:02}:{:02}", mins, secs),
                FontId::monospace(14.0),
                TEXT_DIM,
            );
        } else {
            let mid_y = rect.center().y;
            if !is_busy {
                painter.line_segment(
                    [
                        Pos2::new(rect.left() + 8.0, mid_y),
                        Pos2::new(rect.right() - 8.0, mid_y),
                    ],
                    Stroke::new(1.5, Color32::from_rgb(50, 58, 70)),
                );
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Presiona Iniciar grabación",
                    FontId::proportional(14.0),
                    TEXT_MUTED,
                );
            }
        }

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(6.0);

        // ── Transcript area + Recordings history ─────────────────────────────
        // Split available height: transcript 45%, recordings list 55%
        let avail = ui.available_height();
        let transcript_h = if self.show_recordings { avail * 0.40 } else { avail - 4.0 };
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
            let arrow = if self.show_recordings { "▼" } else { "▶" };
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
                        for entry in &self.recordings {
                            show_recording_row(ui, entry);
                        }
                    }
                });
        }
    }

    fn start_recording(&mut self) {
        self.audio_buffer.lock().unwrap().clear();
        self.waveform_buffer.lock().unwrap().clear();
        self.transcript_edit = "Grabando...".to_string();
        *self.transcript.lock().unwrap() = "Grabando...".to_string();
        self.recording.store(true, Ordering::SeqCst);

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
            let raw_text = if audio_data.is_empty() {
                tp.store(100, Ordering::SeqCst);
                "(Buffer vacío — no se capturó audio)".to_string()
            } else {
                let tp2 = tp.clone();
                match transcribe(&ctx, &audio_data, move |pct| {
                    tp2.store(pct, Ordering::SeqCst);
                }) {
                    Ok(text) => {
                        if text.trim().is_empty() {
                            "(Whisper no detectó habla)".to_string()
                        } else {
                            text
                        }
                    }
                    Err(e) => {
                        eprintln!("Error transcripción: {}", e);
                        format!("Error transcripción: {}", e)
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
                    format!("{}\n\n[Mejorando con Ollama…]", raw_text);

                let op = ollama_progress.clone();
                let improved = match ollama::improve_transcript(&ollama_model, &raw_text, move |pct| {
                    op.store(pct, Ordering::SeqCst);
                }) {
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
                let path = format!("{}/{}", folder, filename);
                if std::fs::write(&path, &final_text).is_ok() {
                    eprintln!("[transcribe] guardado en {}", path);
                    // Insert into DB
                    if let Ok(db) = Database::open(&db_path) {
                        let _ = db.insert_recording(
                            &filename,
                            &path,
                            &now,
                            duration_secs,
                            ollama_model_used.is_some(),
                            ollama_model_used.as_deref(),
                        );
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
                ui.add_space(12.0);

                // ── Section: Modelo Whisper ──────────────────────────────────
                section_header(ui, "🎙  Modelo de transcripción (Whisper)");
                ui.add_space(8.0);

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
                                {
                                    if i != self.selected_model_index {
                                        self.selected_model_index = i;
                                        self.model_changed = true;
                                    }
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

                ui.add_space(16.0);

                // ── Section: Dispositivo de entrada ──────────────────────────
                section_header(ui, "🎤  Dispositivo de entrada (Micrófono)");
                ui.add_space(8.0);

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

                ui.add_space(16.0);

                // ── Section: Dispositivo de salida ────────────────────────────
                section_header(ui, "🔊  Dispositivo de salida (Audio del sistema)");
                ui.add_space(8.0);

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
                    RichText::new("  La captura del sistema usa el monitor del sink de PulseAudio/PipeWire.")
                        .size(12.0)
                        .color(TEXT_MUTED),
                );

                ui.add_space(16.0);

                // ── Section: Ollama ───────────────────────────────────────────
                section_header(ui, "✨  Mejora con Ollama");
                ui.add_space(8.0);

                if !self.ollama_available {
                    status_badge(
                        ui,
                        "Ollama no detectado en localhost:11434",
                        Color32::from_rgb(200, 60, 60),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Instala Ollama (ollama.com) y asegúrate de que esté corriendo.")
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

                ui.add_space(16.0);

                // ── Section: Carpeta de grabaciones ───────────────────────────
                section_header(ui, "📁  Carpeta de grabaciones");
                ui.add_space(8.0);

                ui.add(
                    egui::TextEdit::singleline(&mut self.settings.recordings_folder)
                        .desired_width(f32::INFINITY)
                        .font(FontId::proportional(15.0)),
                );

                ui.add_space(20.0);

                // ── Save button ────────────────────────────────────────────────
                ui.vertical_centered(|ui| {
                    let save_btn = egui::Button::new(
                        RichText::new("  Guardar configuración  ").size(16.0).color(Color32::WHITE),
                    )
                    .fill(ACCENT_BLUE)
                    .rounding(8.0);

                    if ui.add(save_btn).clicked() {
                        self.settings.input_device_id = self
                            .input_devices
                            .get(self.selected_input_index)
                            .map(|(_, id)| id.clone());
                        self.settings.output_device_id = self
                            .output_devices
                            .get(self.selected_output_index)
                            .map(|(_, id)| id.clone());

                        if let Some((_, path)) = self.available_models.get(self.selected_model_index) {
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
                            eprintln!("Error guardando: {}", e);
                        }
                    }
                });

                ui.add_space(16.0);
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
                        let model = entry
                            .ollama_model
                            .as_deref()
                            .unwrap_or("Ollama");
                        ui.label(
                            RichText::new(format!("✨ {}", model))
                                .size(11.0)
                                .color(ACCENT_PURPLE),
                        );
                    }
                });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let open_btn = egui::Button::new(
                    RichText::new("Abrir").size(12.0).color(ACCENT_BLUE),
                )
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
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, mo, d, hour, min, sec)
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
