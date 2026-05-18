use std::sync::atomic::Ordering;

use eframe::egui::{self, Color32, FontId, RichText, Sense, Stroke};

use super::components;
use super::spectrum;
use super::theme::*;
use super::App;

impl App {
    pub(super) fn show_recording_tab(&mut self, ui: &mut egui::Ui) {
        let is_recording = self.recording.load(Ordering::SeqCst);
        let is_paused = self.recording_paused.load(Ordering::SeqCst);
        let is_transcribing = self.is_transcribing.load(Ordering::SeqCst);
        let is_improving = self.is_improving.load(Ordering::SeqCst);
        let is_busy = is_transcribing || is_improving;

        let transcribe_pct = self.transcribe_progress.load(Ordering::SeqCst);
        let ollama_pct = self.ollama_progress.load(Ordering::SeqCst);

        ui.add_space(SPACING_ELEMENT + SPACING_MICRO);

        components::card_frame(ui).show(ui, |ui| {
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
                            format!("{}  Mejorando transcripción...  {}%", super::ICON_MAGIC, p),
                            p,
                        )
                    };
                    ui.add_enabled_ui(false, |ui| {
                        ui.add_sized(
                            egui::vec2(MAIN_ACTION_BUTTON_WIDTH, MAIN_ACTION_BUTTON_HEIGHT),
                            egui::Button::new(
                                RichText::new(&label).size(FONT_H2 - 2.0).color(TEXT_MOON),
                            )
                            .fill(BG_NEBULA),
                        );
                    });
                    ui.add_space(SPACING_ELEMENT);
                    let bar_color = if is_transcribing {
                        ACCENT_CYAN
                    } else {
                        ACCENT_PURPLE
                    };
                    components::progress_bar(ui, pct, bar_color, "Progreso");
                } else if self.is_stopping {
                    let elapsed = self
                        .stop_requested_time
                        .map(|t| t.elapsed().as_secs())
                        .unwrap_or(0);
                    let btn = components::accent_button(
                        format!("⏳  Deteniendo... ({}s)", 3 - elapsed.min(3)),
                        ACCENT_CRIMSON,
                        ACCENT_CRIMSON_HOVER,
                        egui::vec2(MAIN_ACTION_BUTTON_WIDTH, MAIN_ACTION_BUTTON_HEIGHT),
                    );
                    ui.add(btn);

                    if self
                        .stop_requested_time
                        .map(|t| t.elapsed().as_secs() >= 3)
                        .unwrap_or(true)
                    {
                        self.is_stopping = false;
                        self.stop_requested_time = None;
                        self.stop_and_transcribe();
                    }
                } else if is_recording {
                    let btn = components::accent_button(
                        format!("{}  Detener y transcribir", super::ICON_STOP),
                        ACCENT_CRIMSON,
                        ACCENT_CRIMSON_HOVER,
                        egui::vec2(MAIN_ACTION_BUTTON_WIDTH, MAIN_ACTION_BUTTON_HEIGHT),
                    );

                    if ui.add(btn).clicked() {
                        self.last_recording_duration = self
                            .recording_start
                            .map(|s| s.elapsed().as_secs_f64())
                            .unwrap_or(0.0);
                        self.is_stopping = true;
                        self.stop_requested_time = Some(std::time::Instant::now());
                    }

                    ui.add_space(SPACING_TIGHT);
                    ui.horizontal(|ui| {
                        let secondary_btn_width = (MAIN_ACTION_BUTTON_WIDTH - SPACING_TIGHT) / 2.0;

                        let pause_label = if is_paused {
                            format!("{}  Reanudar", super::ICON_PLAY)
                        } else {
                            format!("{}  Pausar", super::ICON_PAUSE)
                        };

                        let pause_btn = components::accent_button(
                            pause_label,
                            ACCENT_AMBER,
                            ACCENT_AMBER,
                            egui::vec2(secondary_btn_width, MAIN_ACTION_BUTTON_HEIGHT),
                        );

                        if ui.add(pause_btn).clicked() {
                            if is_paused {
                                self.resume_recording();
                            } else {
                                self.pause_recording();
                            }
                        }

                        let cancel_btn = components::accent_button(
                            format!("{}  Cancelar", super::ICON_DELETE),
                            ACCENT_CRIMSON,
                            ACCENT_CRIMSON_HOVER,
                            egui::vec2(secondary_btn_width, MAIN_ACTION_BUTTON_HEIGHT),
                        );

                        if ui.add(cancel_btn).clicked() {
                            self.cancel_recording();
                        }
                    });
                } else {
                    let btn = components::accent_button(
                        format!("{}  Iniciar grabación", super::ICON_RECORD),
                        ACCENT_EMERALD,
                        ACCENT_EMERALD_HOVER,
                        egui::vec2(MAIN_ACTION_BUTTON_WIDTH, MAIN_ACTION_BUTTON_HEIGHT),
                    );

                    if ui.add(btn).clicked() {
                        self.recording_start = Some(std::time::Instant::now());
                        self.start_recording();
                    }
                }

                ui.add_space(SPACING_ELEMENT);

                if is_recording {
                    let elapsed = self
                        .recording_start
                        .map(|s| s.elapsed().as_secs_f32())
                        .unwrap_or(0.0);
                    let mins = (elapsed / 60.0) as u32;
                    let secs = (elapsed % 60.0) as u32;

                    ui.horizontal(|ui| {
                        if is_paused {
                            ui.label(
                                RichText::new(format!("{} PAUSADO", super::ICON_PAUSE))
                                    .size(FONT_CAPTION)
                                    .color(ACCENT_AMBER)
                                    .strong(),
                            );
                        } else {
                            let t = ui.input(|i| i.time);
                            let blink = ((t * 2.5).sin() * 0.5 + 0.5) as f32;
                            let dot = ACCENT_CRIMSON.gamma_multiply(0.35 + (0.65 * blink));
                            ui.label(
                                RichText::new(format!("{} GRABANDO", super::ICON_RECORD))
                                    .size(FONT_CAPTION)
                                    .color(dot)
                                    .strong(),
                            );
                        }
                        ui.add_space(SPACING_ELEMENT + SPACING_MICRO);
                        ui.label(
                            RichText::new(format!("{:02}:{:02}", mins, secs))
                                .size(FONT_H3)
                                .color(TEXT_STARLIGHT)
                                .strong(),
                        );
                        ui.add_space(SPACING_ELEMENT + SPACING_MICRO);
                        let highlight_btn = egui::Button::new(
                            RichText::new(format!("{} Highlight", super::ICON_TAG))
                                .size(FONT_CAPTION)
                                .color(ACCENT_PURPLE),
                        )
                        .fill(BG_NEBULA)
                        .stroke(Stroke::new(1.5, ACCENT_PURPLE))
                        .rounding(ROUNDING_SMALL);

                        if ui
                            .add(highlight_btn)
                            .on_hover_text("Ctrl+Shift+H")
                            .clicked()
                        {
                            self.add_highlight_during_recording(None);
                        }
                    });

                    let chunk_state = self.chunk_ui_state.lock().unwrap().clone();
                    ui.add_space(SPACING_TIGHT);
                    ui.label(
                        RichText::new(format!(
                            "Chunks cerrados: {} · Activo: {} · Progreso: {}%",
                            chunk_state.closed_chunks,
                            chunk_state.active_chunk_index,
                            chunk_state.progress_percent()
                        ))
                        .size(FONT_CAPTION)
                        .color(TEXT_MOON),
                    );

                    if !chunk_state.failed_chunks.is_empty() {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                RichText::new("Chunks con error:")
                                    .size(FONT_CAPTION)
                                    .color(ACCENT_CRIMSON_HOVER),
                            );
                            for failed in &chunk_state.failed_chunks {
                                let retry_button = egui::Button::new(
                                    RichText::new(format!("Retry #{}", failed)).size(FONT_CAPTION),
                                )
                                .fill(BG_NEBULA)
                                .stroke(Stroke::new(1.0, ACCENT_CRIMSON_HOVER));

                                if ui.add(retry_button).clicked() {
                                    self.chunk_ui_state
                                        .lock()
                                        .unwrap()
                                        .mark_retry_requested(*failed);
                                }
                            }
                        });
                    }
                } else if !is_busy {
                    ui.label(
                        RichText::new("Presiona Iniciar grabación para comenzar")
                            .size(FONT_CAPTION)
                            .color(TEXT_DUST),
                    );
                }
            });
        });

        if is_recording {
            ui.add_space(SPACING_ELEMENT);
            components::card_frame(ui).show(ui, |ui| {
                ui.label(
                    RichText::new(format!("{} Audio en tiempo real", super::ICON_AUDIO))
                        .size(FONT_CAPTION)
                        .color(TEXT_MOON),
                );
                ui.add_space(SPACING_TIGHT);

                let samples = self.waveform_buffer.lock().unwrap();
                let (rect, _resp) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), SPECTRUM_CANVAS_HEIGHT),
                    Sense::hover(),
                );
                let painter = ui.painter();
                spectrum::paint_spectrum_bars(
                    &samples,
                    &mut self.spectrum_bars,
                    &mut self.spectrum_peak,
                    rect,
                    painter,
                    ui.input(|i| i.time),
                );
            });
        }

        ui.add_space(SPACING_ELEMENT + SPACING_MICRO);

        ui.label(
            RichText::new("Transcripción")
                .size(FONT_CAPTION)
                .color(TEXT_MOON)
                .strong(),
        );
        ui.add_space(SPACING_MICRO + 2.0);

        components::card_frame(ui).show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_source("transcript_scroll")
                .max_height(TRANSCRIPT_MAX_HEIGHT)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.transcript_edit)
                            .desired_width(f32::INFINITY)
                            .desired_rows(5)
                            .font(FontId::proportional(FONT_BODY + 1.0))
                            .text_color(TEXT_STARLIGHT)
                            .interactive(true),
                    );
                });
        });

        ui.add_space(SPACING_ELEMENT + SPACING_MICRO);

        let arrow = if self.show_recordings {
            super::ICON_EXPAND
        } else {
            super::ICON_COLLAPSE
        };
        let count = self.recordings.len();
        let header = format!("{} Grabaciones recientes  ({})", arrow, count);

        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Button::new(
                        RichText::new(&header)
                            .size(FONT_CAPTION)
                            .color(TEXT_MOON)
                            .strong(),
                    )
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::NONE),
                )
                .clicked()
            {
                self.show_recordings = !self.show_recordings;
            }
        });

        if self.show_recordings {
            ui.add_space(SPACING_MICRO + 2.0);
            egui::ScrollArea::vertical()
                .id_source("recordings_scroll")
                .show(ui, |ui| {
                    if self.recordings.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(SPACING_ELEMENT);
                            ui.label(
                                RichText::new("No hay grabaciones aún")
                                    .size(FONT_CAPTION)
                                    .color(TEXT_DUST),
                            );
                        });
                    } else {
                        let expanded_id = self.expanded_recording_id;
                        let recordings = self.recordings.clone();

                        for entry in recordings {
                            let is_expanded = expanded_id == Some(entry.id);
                            let recording_id = entry.id;

                            egui::Frame::group(ui.style())
                                .fill(BG_STARDUST)
                                .stroke(Stroke::new(1.0, BG_ECLIPSE))
                                .rounding(ROUNDING_SMALL + 2.0)
                                .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let expand_btn = egui::Button::new(
                                            RichText::new(if is_expanded {
                                                super::ICON_EXPAND
                                            } else {
                                                super::ICON_COLLAPSE
                                            })
                                            .size(FONT_BODY)
                                            .color(TEXT_MOON),
                                        )
                                        .fill(Color32::TRANSPARENT)
                                        .stroke(Stroke::NONE)
                                        .min_size(egui::vec2(ICON_BUTTON_SIZE, ICON_BUTTON_SIZE));

                                        if ui.add(expand_btn).clicked() {
                                            self.expanded_recording_id = if is_expanded {
                                                None
                                            } else {
                                                Some(recording_id)
                                            };
                                        }

                                        ui.add_space(SPACING_MICRO);

                                        let title =
                                            entry.title.as_deref().unwrap_or(&entry.filename);
                                        let truncated = if title.len() > 40 {
                                            format!("{}...", &title[..37])
                                        } else {
                                            title.to_string()
                                        };

                                        ui.label(
                                            RichText::new(truncated)
                                                .size(FONT_CAPTION)
                                                .color(TEXT_STARLIGHT)
                                                .strong(),
                                        );

                                        ui.add_space(SPACING_TIGHT);
                                        ui.label(
                                            RichText::new(format!(
                                                "⏱ {}",
                                                entry.duration_display()
                                            ))
                                            .size(FONT_CAPTION)
                                            .color(TEXT_MOON),
                                        );

                                        ui.add_space(SPACING_TIGHT);
                                        if entry.has_transcript {
                                            ui.label(
                                                RichText::new(super::ICON_FILE)
                                                    .size(FONT_CAPTION)
                                                    .color(ACCENT_EMERALD),
                                            )
                                            .on_hover_text("Tiene transcripción");
                                        }
                                        if entry.has_summaries {
                                            ui.add_space(SPACING_MICRO);
                                            ui.label(
                                                RichText::new(super::ICON_MAGIC)
                                                    .size(FONT_CAPTION)
                                                    .color(ACCENT_PURPLE),
                                            )
                                            .on_hover_text("Tiene resúmenes");
                                        }

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                #[cfg(feature = "audio-playback")]
                                                {
                                                    // Evitar duplicación visual: cuando la fila está expandida
                                                    // ya existe el reproductor interno completo.
                                                    if !is_expanded {
                                                        let player = self.audio_player.as_ref();
                                                        let is_playing = player
                                                            .map(|p| p.is_playing())
                                                            .unwrap_or(false);
                                                        let is_paused = player
                                                            .map(|p| p.is_paused())
                                                            .unwrap_or(false);
                                                        let is_current_item = self
                                                            .current_playing_id
                                                            == Some(entry.id);

                                                        let (icon, hover_text) =
                                                            if is_current_item && is_playing {
                                                                (super::ICON_PAUSE, "Pausar")
                                                            } else if is_current_item && is_paused {
                                                                (super::ICON_PLAY, "Reanudar")
                                                            } else {
                                                                (super::ICON_PLAY, "Reproducir")
                                                            };

                                                        let play_btn = egui::Button::new(
                                                            RichText::new(icon)
                                                                .size(FONT_CAPTION)
                                                                .color(ACCENT_EMERALD),
                                                        )
                                                        .fill(Color32::TRANSPARENT)
                                                        .stroke(Stroke::new(1.0, ACCENT_EMERALD))
                                                        .rounding(ROUNDING_SMALL)
                                                        .min_size(egui::vec2(
                                                            ICON_BUTTON_SIZE,
                                                            ICON_BUTTON_SIZE,
                                                        ));

                                                        if ui
                                                            .add(play_btn)
                                                            .on_hover_text(hover_text)
                                                            .clicked()
                                                        {
                                                            self.toggle_playback(&entry);
                                                        }
                                                    }
                                                }

                                                let delete_btn = egui::Button::new(
                                                    RichText::new(super::ICON_DELETE)
                                                        .size(FONT_CAPTION)
                                                        .color(ACCENT_CRIMSON),
                                                )
                                                .fill(Color32::TRANSPARENT)
                                                .stroke(Stroke::new(1.0, ACCENT_CRIMSON))
                                                .rounding(ROUNDING_SMALL)
                                                .min_size(egui::vec2(
                                                    ICON_BUTTON_SIZE,
                                                    ICON_BUTTON_SIZE,
                                                ));

                                                if ui
                                                    .add(delete_btn)
                                                    .on_hover_text("Eliminar grabación")
                                                    .clicked()
                                                {
                                                    self.recording_to_delete = Some(entry.id);
                                                    self.show_delete_confirmation = true;
                                                }
                                            },
                                        );
                                    });

                                    if is_expanded {
                                        ui.add_space(SPACING_TIGHT);
                                        self.show_recording_row_expanded(ui, &entry);
                                    }
                                });

                            ui.add_space(SPACING_MICRO + 2.0);
                        }
                    }
                });
        }
    }
}
