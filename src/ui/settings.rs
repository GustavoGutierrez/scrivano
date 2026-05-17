use eframe::egui::{self, Color32, FontId, RichText, Stroke};
use egui_phosphor::regular as icons;

use crate::ollama;

use super::components;
use super::theme::*;
use super::{settings_field_width, truncate_ui_text, App, SettingsSection};

impl App {
    pub(super) fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.set_width(SETTINGS_SIDEBAR_WIDTH);

        ui.add_space(SPACING_ELEMENT);
        ui.label(h3("Configuración").strong());
        ui.add_space(SPACING_ELEMENT);

        let sections = [
            (SettingsSection::Audio, "Audio", super::ICON_MIC),
            (
                SettingsSection::Transcription,
                "Transcripción",
                super::ICON_TRANSCRIPT,
            ),
            (SettingsSection::Ollama, "IA (Ollama)", super::ICON_MAGIC),
            (SettingsSection::Summaries, "Resúmenes", super::ICON_MAGIC),
            (SettingsSection::Prompts, "Prompts", super::ICON_FILE),
            (SettingsSection::System, "Sistema", super::ICON_SETTINGS),
        ];

        for (section, label, icon) in sections {
            let is_selected = self.settings_section == section;
            let (bg, fg) = if is_selected {
                (BG_STARDUST, ACCENT_CYAN)
            } else {
                (Color32::TRANSPARENT, TEXT_MOON)
            };

            let btn = egui::Button::new(
                RichText::new(format!("{} {}", icon, label))
                    .size(FONT_CAPTION)
                    .color(fg),
            )
            .fill(bg)
            .stroke(if is_selected {
                Stroke::new(1.0, ACCENT_CYAN)
            } else {
                Stroke::new(0.0, Color32::TRANSPARENT)
            })
            .rounding(ROUNDING_SMALL)
            .min_size(egui::vec2(140.0, 36.0));

            if ui.add(btn).clicked() {
                self.settings_section = section;
            }
            ui.add_space(SPACING_MICRO);
        }
    }

    pub(super) fn show_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(SPACING_TIGHT);

        egui::SidePanel::left("settings_sidebar")
            .resizable(false)
            .exact_width(SETTINGS_SIDEBAR_WIDTH)
            .frame(egui::Frame::none())
            .show_inside(ui, |ui| {
                self.render_sidebar(ui);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show_inside(ui, |ui| {
                ui.set_min_height(ui.available_height().max(640.0));

                egui::ScrollArea::vertical()
                    .id_source("settings_content_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let available = ui.available_width();
                        let content_width = SETTINGS_CONTENT_MAX_WIDTH.min(available);
                        let side_padding = ((available - content_width) * 0.5).max(0.0);

                        ui.add_space(SPACING_MICRO + 2.0);
                        ui.horizontal(|ui| {
                            ui.add_space(side_padding);
                            ui.vertical(|ui| {
                                ui.set_width(content_width);

                                match self.settings_section {
                                    SettingsSection::Audio => self.render_audio_section(ui),
                                    SettingsSection::Transcription => {
                                        self.render_transcription_section(ui)
                                    }
                                    SettingsSection::Ollama => self.render_ollama_section(ui),
                                    SettingsSection::Summaries => self.render_summary_section(ui),
                                    SettingsSection::Prompts => self.render_prompts_section(ui),
                                    SettingsSection::System => self.render_system_section(ui),
                                }

                                ui.add_space(SPACING_CARD_PAD);

                                components::card_frame(ui).show(ui, |ui| {
                                    ui.vertical_centered(|ui| {
                                        let save_btn = components::accent_button(
                                            "Guardar configuración",
                                            ACCENT_CYAN,
                                            ACCENT_CYAN_HOVER,
                                            egui::vec2(250.0, 42.0),
                                        );

                                        if ui.add(save_btn).clicked() {
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
                                                self.config_save_notification = Some((
                                                    "Configuración guardada".to_string(),
                                                    false,
                                                ));
                                            }
                                        }

                                        if let Some((msg, is_error)) =
                                            &self.config_save_notification
                                        {
                                            ui.add_space(SPACING_TIGHT);
                                            ui.label(RichText::new(msg).size(FONT_BODY).color(
                                                if *is_error {
                                                    ACCENT_CRIMSON
                                                } else {
                                                    ACCENT_EMERALD
                                                },
                                            ));
                                        }
                                    });
                                });

                                ui.add_space(SPACING_ELEMENT + SPACING_MICRO);
                            });
                        });
                    });
            });
    }

    pub(super) fn render_audio_section(&mut self, ui: &mut egui::Ui) {
        components::card_frame(ui).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(h3(format!("{} Audio", super::ICON_MIC)).strong());
                ui.add_space(SPACING_TIGHT);
                if self.input_devices.is_empty() {
                    components::status_badge(ui, "Sin dispositivos", ACCENT_AMBER);
                } else {
                    components::status_badge(ui, "Capturando correctamente", ACCENT_EMERALD);
                }
            });

            ui.add_space(SPACING_MICRO + 2.0);
            ui.label(caption(
                "Selecciona la fuente principal para grabar y el fallback de sistema.",
            ));

            ui.add_space(SPACING_ELEMENT + SPACING_MICRO / 2.0);
            if ui
                .add(
                    egui::Button::new(
                        RichText::new(format!(
                            "{} Actualizar dispositivos",
                            icons::ARROW_CLOCKWISE
                        ))
                        .size(FONT_CAPTION),
                    )
                    .fill(BG_NEBULA)
                    .stroke(Stroke::new(1.0, BG_ECLIPSE))
                    .rounding(ROUNDING_SMALL),
                )
                .clicked()
            {
                self.refresh_audio_devices();
            }

            ui.add_space(SPACING_ELEMENT + SPACING_MICRO / 2.0);
            ui.separator();
            ui.add_space(SPACING_ELEMENT + SPACING_MICRO / 2.0);

            ui.label(caption("Micrófono"));
            ui.add_space(SPACING_MICRO + 1.0);
            if self.input_devices.is_empty() {
                ui.label(body("No se encontraron").color(TEXT_MOON));
            } else {
                let current_input = self
                    .input_devices
                    .get(self.selected_input_index)
                    .map(|(n, _)| truncate_ui_text(n, 48))
                    .unwrap_or_else(|| "—".to_string());
                let field_width = settings_field_width(ui);
                let combo_resp = egui::ComboBox::from_id_source("input_device_settings")
                    .selected_text(RichText::new(current_input).size(FONT_BODY))
                    .width(field_width)
                    .show_ui(ui, |ui| {
                        for (i, (name, _)) in self.input_devices.iter().enumerate() {
                            let selected = i == self.selected_input_index;
                            let response = ui.selectable_label(
                                selected,
                                RichText::new(truncate_ui_text(name, 70)).size(FONT_BODY),
                            );
                            let response = response.on_hover_text(name);
                            if response.clicked() {
                                self.selected_input_index = i;
                            }
                        }
                    });
                combo_resp
                    .response
                    .on_hover_text("Dispositivo de captura principal");
            }

            ui.add_space(SPACING_ELEMENT);
            ui.label(caption("Salida de audio"));
            ui.add_space(SPACING_MICRO + 1.0);
            if self.output_devices.is_empty() {
                ui.label(body("No se encontraron").color(TEXT_MOON));
            } else {
                let current_output = self
                    .output_devices
                    .get(self.selected_output_index)
                    .map(|(n, _)| truncate_ui_text(n, 48))
                    .unwrap_or_else(|| "—".to_string());
                let field_width = settings_field_width(ui);
                egui::ComboBox::from_id_source("output_device_settings")
                    .selected_text(RichText::new(current_output).size(FONT_BODY))
                    .width(field_width)
                    .show_ui(ui, |ui| {
                        for (i, (name, _)) in self.output_devices.iter().enumerate() {
                            let selected = i == self.selected_output_index;
                            let response = ui.selectable_label(
                                selected,
                                RichText::new(truncate_ui_text(name, 70)).size(FONT_BODY),
                            );
                            let response = response.on_hover_text(name);
                            if response.clicked() {
                                self.selected_output_index = i;
                            }
                        }
                    });
            }

            ui.add_space(SPACING_ELEMENT - SPACING_MICRO / 2.0);
            ui.label(caption(
                "Se prioriza el micrófono seleccionado; si falla, se usa la salida de sistema.",
            ));
        });
    }

    pub(super) fn render_transcription_section(&mut self, ui: &mut egui::Ui) {
        components::card_frame(ui).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(h3(format!("{} Transcripción", super::ICON_TRANSCRIPT)).strong());
                ui.add_space(SPACING_TIGHT);
                components::status_badge(ui, "Activo", ACCENT_EMERALD);
            });

            ui.add_space(SPACING_ELEMENT);
            ui.label(caption("Modelo Whisper"));
            ui.add_space(SPACING_MICRO + 1.0);

            if self.available_models.is_empty() {
                ui.label(body("No se encontraron modelos en models/").color(ACCENT_CRIMSON));
            } else {
                let current_name = self
                    .available_models
                    .get(self.selected_model_index)
                    .map(|(n, _)| truncate_ui_text(n, 48))
                    .unwrap_or_else(|| "—".to_string());
                let field_width = settings_field_width(ui);
                egui::ComboBox::from_id_source("whisper_model_settings")
                    .selected_text(RichText::new(current_name).size(FONT_BODY))
                    .width(field_width)
                    .show_ui(ui, |ui| {
                        for (i, (name, _)) in self.available_models.iter().enumerate() {
                            let selected = i == self.selected_model_index;
                            let response = ui.selectable_label(
                                selected,
                                RichText::new(truncate_ui_text(name, 70)).size(FONT_BODY),
                            );
                            let response = response.on_hover_text(name);
                            if response.clicked() && i != self.selected_model_index {
                                self.selected_model_index = i;
                                self.model_changed = true;
                            }
                        }
                    });
            }

            if self.model_changed {
                ui.add_space(SPACING_TIGHT);
                ui.label(
                    caption("⚠ El modelo nuevo se aplica al reiniciar la aplicación.")
                        .color(ACCENT_AMBER),
                );
            }

            ui.add_space(SPACING_ELEMENT);
            ui.label(caption("Calidad: Alta · Latencia: Media"));
        });
    }

    pub(super) fn render_ollama_section(&mut self, ui: &mut egui::Ui) {
        components::card_frame(ui).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(h3(format!("{} IA (Ollama)", super::ICON_MAGIC)).strong());
                ui.add_space(SPACING_ELEMENT);
                let toggle_text = if self.ollama_enabled {
                    caption("Activado").color(ACCENT_EMERALD)
                } else {
                    caption("Desactivado")
                };
                ui.checkbox(&mut self.ollama_enabled, toggle_text);
            });

            ui.add_space(SPACING_TIGHT);
            if !self.ollama_available {
                components::status_badge(ui, "No disponible", ACCENT_AMBER);
                ui.add_space(SPACING_MICRO + 2.0);
                ui.label(caption(
                    "Instala Ollama y ejecuta el servicio local para habilitar IA.",
                ));
                return;
            }

            components::status_badge(ui, "Disponible", ACCENT_EMERALD);

            if !self.ollama_enabled {
                ui.add_space(SPACING_TIGHT);
                ui.label(caption(
                    "Activa la opción para seleccionar modelo y mejoras automáticas.",
                ));
                return;
            }

            ui.add_space(SPACING_ELEMENT);
            ui.label(caption("Modelo"));
            ui.add_space(SPACING_MICRO + 1.0);

            if self.ollama_models.is_empty() {
                ui.label(body("No hay modelos instalados").color(ACCENT_CRIMSON));
            } else {
                let current_name = self
                    .ollama_models
                    .get(self.ollama_selected_index)
                    .map(|name| truncate_ui_text(name, 48))
                    .unwrap_or_else(|| "—".to_string());
                let field_width = settings_field_width(ui);
                egui::ComboBox::from_id_source("ollama_model_settings")
                    .selected_text(RichText::new(current_name).size(FONT_BODY))
                    .width(field_width)
                    .show_ui(ui, |ui| {
                        for (i, name) in self.ollama_models.iter().enumerate() {
                            let selected = i == self.ollama_selected_index;
                            let response = ui.selectable_label(
                                selected,
                                RichText::new(truncate_ui_text(name, 70)).size(FONT_BODY),
                            );
                            let response = response.on_hover_text(name);
                            if response.clicked() {
                                self.ollama_selected_index = i;
                            }
                        }
                    });
            }

            ui.add_space(SPACING_TIGHT + SPACING_MICRO);
            if ui
                .add(
                    egui::Button::new(
                        RichText::new(format!("{} Actualizar modelos", icons::ARROW_CLOCKWISE))
                            .size(FONT_CAPTION),
                    )
                    .fill(BG_NEBULA)
                    .stroke(Stroke::new(1.0, BG_ECLIPSE))
                    .rounding(ROUNDING_SMALL),
                )
                .clicked()
            {
                self.ollama_models = ollama::list_models();
                self.ollama_selected_index = self
                    .ollama_selected_index
                    .min(self.ollama_models.len().saturating_sub(1));
            }

            ui.add_space(SPACING_ELEMENT);
            ui.separator();
            ui.add_space(SPACING_TIGHT + SPACING_MICRO);

            ui.label(caption("Funciones activas"));
            ui.add_space(SPACING_MICRO + 1.0);
            ui.label(caption(format!(
                "{} Corrección ortográfica",
                super::ICON_CHECK
            )));
            ui.label(caption(format!("{} Mejora semántica", super::ICON_CHECK)));
            ui.label(caption(format!("{} Limpieza de ruido", super::ICON_CHECK)));

            ui.add_space(SPACING_TIGHT);
            ui.label(caption("Uso estimado: Bajo").color(TEXT_DUST));
        });
    }

    pub(super) fn render_summary_section(&mut self, ui: &mut egui::Ui) {
        components::card_frame(ui).show(ui, |ui| {
            ui.label(h3(format!("{} Resúmenes", super::ICON_MAGIC)).strong());
            ui.add_space(SPACING_ELEMENT);

            if !(self.ollama_available && self.ollama_enabled) {
                ui.label(caption(
                    "Activa Ollama para configurar los resúmenes automáticos.",
                ));
                return;
            }

            ui.label(caption("Modelo para resúmenes"));
            ui.add_space(SPACING_MICRO + 1.0);
            if !self.ollama_models.is_empty() {
                let current_summary_model = &self.settings.summary_model;
                let summary_idx = self
                    .ollama_models
                    .iter()
                    .position(|m| m == current_summary_model)
                    .unwrap_or(0);
                let field_width = settings_field_width(ui);
                egui::ComboBox::from_id_source("summary_model_settings")
                    .selected_text(
                        RichText::new(truncate_ui_text(current_summary_model, 48)).size(FONT_BODY),
                    )
                    .width(field_width)
                    .show_ui(ui, |ui| {
                        for (i, name) in self.ollama_models.iter().enumerate() {
                            let selected = i == summary_idx;
                            let response = ui.selectable_label(
                                selected,
                                RichText::new(truncate_ui_text(name, 70)).size(FONT_BODY),
                            );
                            let response = response.on_hover_text(name);
                            if response.clicked() {
                                self.settings.summary_model = name.clone();
                            }
                        }
                    });
            }

            ui.add_space(SPACING_ELEMENT);
            ui.label(caption("Modo streaming"));
            ui.add_space(SPACING_MICRO + 1.0);
            egui::ComboBox::from_id_source("stream_mode_settings")
                .selected_text(RichText::new(&self.settings.summary_stream_mode).size(FONT_BODY))
                .width(settings_field_width(ui))
                .show_ui(ui, |ui| {
                    for mode in ["auto", "stream", "non_stream"] {
                        let selected = self.settings.summary_stream_mode == mode;
                        if ui
                            .selectable_label(selected, RichText::new(mode).size(FONT_BODY))
                            .clicked()
                        {
                            self.settings.summary_stream_mode = mode.to_string();
                        }
                    }
                });

            ui.add_space(SPACING_ELEMENT);
            ui.label(caption("Política de thinking"));
            ui.add_space(SPACING_MICRO + 1.0);
            egui::ComboBox::from_id_source("thinking_policy_settings")
                .selected_text(
                    RichText::new(match self.settings.summary_thinking_policy.as_str() {
                        "hide_thinking" => "Ocultar siempre",
                        "store_but_hide" => "Guardar pero ocultar",
                        "show_for_debug" => "Mostrar (debug)",
                        _ => &self.settings.summary_thinking_policy,
                    })
                    .size(FONT_BODY),
                )
                .width(settings_field_width(ui))
                .show_ui(ui, |ui| {
                    for policy in ["hide_thinking", "store_but_hide", "show_for_debug"] {
                        let selected = self.settings.summary_thinking_policy == policy;
                        let label = match policy {
                            "hide_thinking" => "Ocultar siempre",
                            "store_but_hide" => "Guardar pero ocultar",
                            "show_for_debug" => "Mostrar (debug)",
                            _ => policy,
                        };
                        if ui
                            .selectable_label(selected, RichText::new(label).size(FONT_BODY))
                            .clicked()
                        {
                            self.settings.summary_thinking_policy = policy.to_string();
                        }
                    }
                });
        });
    }

    pub(super) fn render_prompts_section(&mut self, ui: &mut egui::Ui) {
        components::card_frame(ui).show(ui, |ui| {
            ui.label(h3(format!("{} Prompts personalizados", super::ICON_FILE)).strong());
            ui.add_space(SPACING_MICRO + 2.0);
            ui.label(caption(
                "Edita cada plantilla según el tipo de resumen que quieras generar.",
            ));
            ui.add_space(SPACING_ELEMENT);

            egui::CollapsingHeader::new(body("Ejecutivo"))
                .default_open(false)
                .show(ui, |ui| {
                    ui.add_space(SPACING_MICRO + 2.0);
                    ui.add_sized(
                        egui::vec2(ui.available_width(), 84.0),
                        egui::TextEdit::multiline(&mut self.settings.custom_prompt_executive)
                            .font(FontId::proportional(FONT_CAPTION))
                            .hint_text(
                                "Ej: Enfócate en puntos clave, contexto y decisiones finales.",
                            ),
                    );
                });

            ui.add_space(SPACING_TIGHT);
            egui::CollapsingHeader::new(body("Tareas"))
                .default_open(false)
                .show(ui, |ui| {
                    ui.add_space(SPACING_MICRO + 2.0);
                    ui.add_sized(
                        egui::vec2(ui.available_width(), 84.0),
                        egui::TextEdit::multiline(&mut self.settings.custom_prompt_tasks)
                            .font(FontId::proportional(FONT_CAPTION))
                            .hint_text("Ej: Lista tareas con responsable, fecha y prioridad."),
                    );
                });

            ui.add_space(SPACING_TIGHT);
            egui::CollapsingHeader::new(body("Decisiones"))
                .default_open(false)
                .show(ui, |ui| {
                    ui.add_space(SPACING_MICRO + 2.0);
                    ui.add_sized(
                        egui::vec2(ui.available_width(), 84.0),
                        egui::TextEdit::multiline(&mut self.settings.custom_prompt_decisions)
                            .font(FontId::proportional(FONT_CAPTION))
                            .hint_text("Ej: Extrae acuerdos, alcance y fecha de ejecución."),
                    );
                });
        });
    }

    pub(super) fn render_system_section(&mut self, ui: &mut egui::Ui) {
        components::card_frame(ui).show(ui, |ui| {
            ui.label(h3(format!("{} Sistema", super::ICON_SETTINGS)).strong());

            ui.add_space(SPACING_ELEMENT);
            ui.label(caption("Idioma"));
            ui.add_space(SPACING_MICRO + 1.0);
            egui::ComboBox::from_id_source("language_settings")
                .selected_text(RichText::new(&self.settings.language_default).size(FONT_BODY))
                .width(settings_field_width(ui))
                .show_ui(ui, |ui| {
                    for lang in ["es", "en"] {
                        let selected = self.settings.language_default == lang;
                        if ui
                            .selectable_label(selected, RichText::new(lang).size(FONT_BODY))
                            .clicked()
                        {
                            self.settings.language_default = lang.to_string();
                        }
                    }
                });

            ui.add_space(SPACING_ELEMENT);
            ui.separator();
            ui.add_space(SPACING_ELEMENT);

            ui.label(caption("Atajos de teclado"));
            ui.add_space(SPACING_TIGHT);

            ui.label(caption("Iniciar/Detener"));
            ui.add_space(SPACING_MICRO);
            ui.add_sized(
                egui::vec2(settings_field_width(ui), 28.0),
                egui::TextEdit::singleline(&mut self.settings.hotkey_start_stop)
                    .font(FontId::proportional(FONT_BODY)),
            );

            ui.add_space(SPACING_TIGHT + SPACING_MICRO);
            ui.label(caption("Highlight"));
            ui.add_space(SPACING_MICRO);
            ui.add_sized(
                egui::vec2(settings_field_width(ui), 28.0),
                egui::TextEdit::singleline(&mut self.settings.hotkey_highlight)
                    .font(FontId::proportional(FONT_BODY)),
            );

            ui.add_space(SPACING_ELEMENT);
            ui.separator();
            ui.add_space(SPACING_ELEMENT);

            ui.label(caption("Carpeta de grabaciones"));
            ui.add_space(SPACING_MICRO + 1.0);
            ui.add(
                egui::TextEdit::singleline(&mut self.settings.recordings_folder)
                    .desired_width(f32::INFINITY)
                    .font(FontId::proportional(FONT_BODY)),
            );
        });
    }
}
