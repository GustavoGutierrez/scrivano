//! Reusable UI component helpers for Scrivano.

use eframe::egui::{self, Color32, RichText, Sense, Stroke, Ui, Vec2};

use super::theme::*;

pub fn section_header(ui: &mut Ui, icon: &str, title: &str, subtitle: Option<&str>) {
    ui.add_space(SPACING_ELEMENT);
    ui.label(h1(format!("{} {}", icon, title)));
    if let Some(sub) = subtitle {
        ui.add_space(SPACING_TIGHT);
        ui.label(caption(sub));
    }
    ui.add_space(SPACING_ELEMENT);
}

pub fn card_frame(ui: &Ui) -> egui::Frame {
    egui::Frame::group(ui.style())
        .fill(BG_STARDUST)
        .stroke(border_stroke())
        .rounding(ROUNDING_CARD)
        .inner_margin(egui::Margin::symmetric(
            SPACING_CARD_PAD,
            SPACING_CARD_PAD - 4.0,
        ))
}

pub fn accent_button(
    label: impl Into<String>,
    color: Color32,
    hover_color: Color32,
    size: Vec2,
) -> egui::Button<'static> {
    egui::Button::new(
        RichText::new(label)
            .size(FONT_H3)
            .color(TEXT_WHITE)
            .strong(),
    )
    .fill(color)
    .stroke(Stroke::new(2.0, hover_color))
    .rounding(ROUNDING_BUTTON)
    .min_size(size)
}

pub fn status_badge(ui: &mut Ui, text: &str, color: Color32) {
    let width = ui.fonts(|f| {
        f.layout(text.to_owned(), mono_font(), Color32::WHITE, f32::INFINITY)
            .rect
            .width()
    }) + 20.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 22.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, ROUNDING_PILL, color.gamma_multiply(0.22));
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        mono_font(),
        color,
    );
}

pub fn progress_bar(ui: &mut Ui, pct: i32, color: Color32, label: &str) {
    let actual_pct = pct.clamp(0, 100) as f32;
    let width = ui.available_width().min(320.0);
    let height = 8.0;

    ui.vertical_centered(|ui| {
        ui.label(muted(format!("{} {}%", label, actual_pct as u32)));
        ui.add_space(SPACING_TIGHT);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        ui.painter().rect_filled(rect, height / 2.0, BG_NEBULA);
        let fill_w = rect.width() * (actual_pct / 100.0).clamp(0.0, 1.0);
        if fill_w > 0.0 {
            let fill_rect = egui::Rect::from_min_size(rect.min, Vec2::new(fill_w, height));
            ui.painter().rect_filled(fill_rect, height / 2.0, color);
        }
    });
}

pub fn tech_badge(ui: &mut Ui, label: &str, color: Color32) {
    let width = ui.fonts(|f| {
        f.layout(label.to_owned(), mono_font(), Color32::WHITE, f32::INFINITY)
            .rect
            .width()
    }) + 20.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 24.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, ROUNDING_SMALL, color.gamma_multiply(0.25));
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        mono_font(),
        color,
    );
}

pub fn apply_nebula_theme(ctx: &egui::Context) {
    ctx.style_mut(|s| {
        s.visuals.panel_fill = BG_VOID;
        s.visuals.window_fill = BG_VOID;
        s.visuals.override_text_color = Some(TEXT_STARLIGHT);
        s.visuals.widgets.noninteractive.bg_stroke = border_stroke();
        s.visuals.widgets.inactive.bg_fill = BG_NEBULA;
        s.visuals.widgets.inactive.bg_stroke = border_stroke();
        s.visuals.widgets.hovered.bg_fill = BG_STARDUST_HOVER;
        s.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT_CYAN.gamma_multiply(0.4));
        s.visuals.widgets.active.bg_fill = BG_NEBULA;
        s.visuals.widgets.active.bg_stroke = Stroke::new(2.0, ACCENT_CYAN);
        s.visuals.widgets.open.bg_fill = BG_STARDUST;
        s.visuals.widgets.open.bg_stroke = Stroke::new(1.0, ACCENT_CYAN);
        s.visuals.selection.bg_fill = ACCENT_CYAN.gamma_multiply(0.35);
        s.visuals.selection.stroke = Stroke::new(1.0, ACCENT_CYAN);
        s.spacing.item_spacing = Vec2::new(SPACING_ELEMENT, SPACING_TIGHT);
        s.spacing.button_padding = Vec2::new(SPACING_ELEMENT, SPACING_TIGHT);
    });
}
