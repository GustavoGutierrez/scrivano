use eframe::egui::{self, Color32, RichText, Sense, Stroke};

use super::components;
use super::theme::*;
use super::App;

impl App {
    pub(super) fn show_about_tab(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_source("about_scroll")
            .show(ui, |ui| {
                ui.add_space(SPACING_ELEMENT + SPACING_MICRO);

                let content_width = 920.0_f32.min(ui.available_width());
                let side_padding = ((ui.available_width() - content_width) * 0.5).max(0.0);

                ui.horizontal(|ui| {
                    ui.add_space(side_padding);
                    ui.vertical(|ui| {
                        ui.set_width(content_width);

                        components::card_frame(ui).show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(h1("Scrivano"));
                                ui.add_space(SPACING_TIGHT);

                                let version = format!("  v{}  ", env!("CARGO_PKG_VERSION"));
                                let (badge_rect, _) =
                                    ui.allocate_exact_size(egui::vec2(84.0, 26.0), Sense::hover());
                                ui.painter().rect_filled(badge_rect, 13.0, ACCENT_CYAN);
                                ui.painter().text(
                                    badge_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    &version,
                                    egui::FontId::proportional(FONT_CAPTION),
                                    TEXT_WHITE,
                                );
                                ui.add_space(SPACING_TIGHT);
                                ui.label(body("Transcripción local de audio · 100% offline"));
                            });
                        });

                        ui.add_space(SPACING_ELEMENT + SPACING_MICRO);

                        components::card_frame(ui).show(ui, |ui| {
                            ui.label(h3("Características"));
                            ui.add_space(SPACING_MICRO);
                            ui.label(caption(
                                "Funciones clave para capturar, transcribir y organizar reuniones.",
                            ));
                            ui.add_space(SPACING_ELEMENT);

                            let features = [
                                (
                                    super::ICON_MIC,
                                    "Captura de audio del sistema",
                                    "Graba cualquier sonido reproducido por tu equipo usando PulseAudio o PipeWire.",
                                    ACCENT_CYAN,
                                ),
                                (
                                    super::ICON_TRANSCRIPT,
                                    "Whisper offline",
                                    "Transcripción local con modelos GGML; tus datos nunca salen de tu máquina.",
                                    ACCENT_EMERALD,
                                ),
                                (
                                    super::ICON_MAGIC,
                                    "Mejora con Ollama",
                                    "Corrección opcional de ortografía y semántica con modelos locales.",
                                    ACCENT_PURPLE,
                                ),
                                (
                                    super::ICON_FOLDER,
                                    "Historial SQLite",
                                    "Consulta y gestiona grabaciones con fecha, duración y resúmenes en un solo lugar.",
                                    ACCENT_CYAN_DIM,
                                ),
                            ];

                            if ui.available_width() >= 760.0 {
                                egui::Grid::new("about_features_grid")
                                    .num_columns(2)
                                    .spacing(egui::vec2(SPACING_TIGHT + 2.0, SPACING_TIGHT + 2.0))
                                    .show(ui, |ui| {
                                        for (idx, (icon, title, desc, accent)) in
                                            features.iter().enumerate()
                                        {
                                            about_feature_card(ui, icon, title, desc, *accent);
                                            if idx % 2 == 1 {
                                                ui.end_row();
                                            }
                                        }
                                    });
                            } else {
                                for (icon, title, desc, accent) in features {
                                    about_feature_card(ui, icon, title, desc, accent);
                                    ui.add_space(SPACING_TIGHT);
                                }
                            }
                        });

                        ui.add_space(SPACING_ELEMENT + SPACING_MICRO);

                        ui.label(h3("Stack tecnológico"));
                        ui.add_space(SPACING_TIGHT);

                        components::card_frame(ui).show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(SPACING_MICRO + 2.0, SPACING_MICRO + 2.0);
                                for (label, color) in [
                                    ("Rust", ACCENT_CRIMSON),
                                    ("egui 0.27", ACCENT_CYAN),
                                    ("whisper-rs 0.15", ACCENT_EMERALD),
                                    ("PulseAudio", ACCENT_PURPLE),
                                    ("Ollama", ACCENT_AMBER),
                                    ("SQLite", ACCENT_CYAN_DIM),
                                ] {
                                    components::tech_badge(ui, label, color);
                                }
                            });
                        });

                        ui.add_space(SPACING_ELEMENT + SPACING_MICRO);

                        components::card_frame(ui).show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(caption("Desarrollado por"));
                                ui.add_space(SPACING_MICRO);
                                ui.label(h3("Gustavo Gutiérrez"));
                                ui.add_space(SPACING_MICRO / 2.0);
                                ui.label(caption("Bogotá, Colombia"));
                            });
                        });

                        ui.add_space(SPACING_CARD_PAD);
                    });
                });
            });
    }
}

fn about_feature_card(ui: &mut egui::Ui, icon: &str, title: &str, desc: &str, accent: Color32) {
    components::card_frame(ui).show(ui, |ui| {
        ui.set_min_height(118.0);
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                egui::Frame::none()
                    .fill(accent.gamma_multiply(0.22))
                    .stroke(Stroke::new(1.0, accent))
                    .rounding(ROUNDING_SMALL + 2.0)
                    .inner_margin(egui::Margin::symmetric(
                        SPACING_TIGHT,
                        SPACING_TIGHT - SPACING_MICRO / 2.0,
                    ))
                    .show(ui, |ui| {
                        ui.label(RichText::new(icon).size(FONT_H3).color(accent));
                    });
                ui.add_space(SPACING_TIGHT);
                ui.label(body(title).strong());
            });
            ui.add_space(SPACING_TIGHT);
            ui.label(caption(desc));
        });
    });
}
