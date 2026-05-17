//! Reusable UI component helpers for Scrivano — Nebula Dark Design System
//!
//! Each function is a drop-in replacement for common UI patterns.
//! All use the theme constants from `assets/theme.rs`.
//!
//! Usage: Copy this file to `src/ui/components.rs` and call from tab renderers.

use eframe::egui::{self, Color32, RichText, Sense, Stroke, Ui, Vec2};

use super::theme::*;

// ── Typography helpers ────────────────────────────────────────────────────────

/// Section heading with optional icon and status badge
pub fn section_header(
    ui: &mut Ui,
    icon: &str,
    title: &str,
    subtitle: Option<&str>,
) {
    ui.add_space(SPACING_ELEMENT);
    ui.horizontal(|ui| {
        ui.label(h1(format!("{} {}", icon, title)));
    });
    if let Some(sub) = subtitle {
        ui.add_space(SPACING_TIGHT);
        ui.label(caption(sub));
    }
    ui.add_space(SPACING_ELEMENT);
}

/// Field label with optional description
pub fn field_label(ui: &mut Ui, label: &str, description: Option<&str>) {
    ui.label(h3(label));
    if let Some(desc) = description {
        ui.label(caption(desc));
    }
    ui.add_space(SPACING_TIGHT);
}

// ── Card container ────────────────────────────────────────────────────────────

/// Create a themed card frame with standard padding and rounding
pub fn card_frame<'ui>(
    ui: &'ui mut Ui,
) -> egui::Frame {
    egui::Frame::group(ui.style())
        .fill(BG_STARDUST)
        .stroke(border_stroke())
        .rounding(ROUNDING_CARD)
        .inner_margin(egui::Margin::symmetric(SPACING_CARD_PAD, SPACING_CARD_PAD - 4.0))
}

/// Card with hover effect (slightly brighter fill on hover)
pub fn card_frame_interactive<'ui>(
    ui: &'ui mut Ui,
    hovered: bool,
) -> egui::Frame {
    let fill = if hovered { BG_STARDUST_HOVER } else { BG_STARDUST };
    egui::Frame::group(ui.style())
        .fill(fill)
        .stroke(if hovered {
            Stroke::new(BORDER_WIDTH, ACCENT_CYAN.gamma_multiply(0.4))
        } else {
            border_stroke()
        })
        .rounding(ROUNDING_CARD)
        .inner_margin(egui::Margin::symmetric(SPACING_CARD_PAD, SPACING_CARD_PAD - 4.0))
}

// ── Buttons ───────────────────────────────────────────────────────────────────

/// Primary action button with accent color
pub fn accent_button(
    label: impl Into<String>,
    color: Color32,
    hover_color: Color32,
    size: Vec2,
) -> egui::Button {
    egui::Button::new(
        RichText::new(label)
            .size(FONT_H3)
            .color(TEXT_WHITE)
            .strong(),
    )
    .fill(color)
    .stroke(Stroke::new(BORDER_WIDTH_ACTIVE, hover_color))
    .rounding(ROUNDING_BUTTON)
    .min_size(size)
}

/// Icon-only button (transparent background, colored border)
pub fn icon_button(
    icon: &str,
    color: Color32,
    size: f32,
) -> egui::Button {
    let btn_size = Vec2::splat(size);
    egui::Button::new(
        RichText::new(icon).size(size * 0.45).color(color),
    )
    .fill(Color32::TRANSPARENT)
    .stroke(Stroke::new(BORDER_WIDTH, color))
    .rounding(ROUNDING_SMALL)
    .min_size(btn_size)
}

/// Text button with hover accent
pub fn text_button(
    label: impl Into<String>,
    color: Color32,
) -> egui::Button {
    egui::Button::new(
        RichText::new(label).size(FONT_BODY).color(color),
    )
    .fill(Color32::TRANSPARENT)
    .stroke(Stroke::NONE)
}

/// Ghost button — transparent, no border, subtle text
pub fn ghost_button(
    label: impl Into<String>,
) -> egui::Button {
    egui::Button::new(
        RichText::new(label).size(FONT_BODY).color(TEXT_MOON),
    )
    .fill(Color32::TRANSPARENT)
    .stroke(Stroke::NONE)
}

// ── Status indicators ─────────────────────────────────────────────────────────

/// Colored badge pill with text
pub fn status_badge(ui: &mut Ui, text: &str, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.fonts(|f| f.layout(text.to_string(), mono_font(), Color32::WHITE, f32::INFINITY).rect.width()) + 20.0, 22.0),
        Sense::hover(),
    );
    ui.painter().rect_filled(rect, ROUNDING_PILL, color.gamma_multiply(0.22));
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        mono_font(),
        color,
    );
}

/// Recording indicator — pulsing red dot with "GRABANDO" text
pub fn recording_indicator(ui: &mut Ui, blink_intensity: f32) {
    ui.horizontal(|ui| {
        let dot_color = Color32::from_rgba_premultiplied(
            230,
            (40.0 + 200.0 * blink_intensity) as u8,
            40,
            255,
        );
        ui.label(
            RichText::new("● GRABANDO")
                .size(FONT_BODY)
                .color(dot_color)
                .strong(),
        );
    });
}

/// Timer display — formatted MM:SS
pub fn timer_display(ui: &mut Ui, elapsed_secs: f32) {
    let mins = (elapsed_secs / 60.0) as u32;
    let secs = (elapsed_secs % 60.0) as u32;
    ui.label(
        RichText::new(format!("{:02}:{:02}", mins, secs))
            .size(FONT_H2)
            .color(TEXT_STARLIGHT)
            .strong(),
    );
}

// ── Progress ──────────────────────────────────────────────────────────────────

/// Horizontal progress bar with label
pub fn progress_bar(
    ui: &mut Ui,
    pct: i32,
    color: Color32,
    label: &str,
) {
    let actual_pct = pct.max(0).min(100) as f32;
    let available = ui.available_width().min(320.0);
    let bar_h = 8.0;

    ui.vertical_centered(|ui| {
        ui.label(muted(format!("{} {}%", label, actual_pct as u32)));
        ui.add_space(SPACING_TIGHT);

        let (bar_rect, _) = ui.allocate_exact_size(
            Vec2::new(available, bar_h),
            Sense::hover(),
        );
        ui.painter().rect_filled(bar_rect, bar_h / 2.0, BG_NEBULA);

        let fill_w = bar_rect.width() * (actual_pct / 100.0).clamp(0.0, 1.0);
        if fill_w > 0.0 {
            let fill_rect = egui::Rect::from_min_size(
                bar_rect.min,
                Vec2::new(fill_w, bar_h),
            );
            ui.painter().rect_filled(fill_rect, bar_h / 2.0, color);
        }
    });
}

// ── Dividers ──────────────────────────────────────────────────────────────────

/// Subtle horizontal separator
pub fn divider(ui: &mut Ui) {
    ui.add_space(SPACING_ELEMENT);
    ui.separator();
    ui.add_space(SPACING_ELEMENT);
}

/// Colored accent divider
pub fn accent_divider(ui: &mut Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 2.0),
        Sense::hover(),
    );
    ui.painter().rect_filled(rect, 1.0, color);
}

// ── Tech badge ────────────────────────────────────────────────────────────────

/// Small rounded badge for technology labels (About page)
pub fn tech_badge(ui: &mut Ui, label: &str, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(
            ui.fonts(|f| f.layout(label.to_string(), mono_font(), Color32::WHITE, f32::INFINITY).rect.width()) + 20.0,
            24.0,
        ),
        Sense::hover(),
    );
    ui.painter().rect_filled(rect, ROUNDING_SMALL, color.gamma_multiply(0.25));
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        mono_font(),
        color,
    );
}

// ── Update egui visual style ──────────────────────────────────────────────────

/// Apply Nebula Dark theme to egui context
pub fn apply_nebula_theme(ctx: &egui::Context) {
    ctx.style_mut(|s| {
        s.visuals.panel_fill = BG_VOID;
        s.visuals.window_fill = BG_VOID;
        s.visuals.override_text_color = Some(TEXT_STARLIGHT);
        s.visuals.widgets.noninteractive.bg_stroke = border_stroke();
        s.visuals.widgets.inactive.bg_fill = BG_NEBULA;
        s.visuals.widgets.inactive.bg_stroke = border_stroke();
        s.visuals.widgets.hovered.bg_fill = BG_STARDUST_HOVER;
        s.visuals.widgets.hovered.bg_stroke = Stroke::new(BORDER_WIDTH, ACCENT_CYAN.gamma_multiply(0.4));
        s.visuals.widgets.active.bg_fill = BG_NEBULA;
        s.visuals.widgets.active.bg_stroke = Stroke::new(BORDER_WIDTH_ACTIVE, ACCENT_CYAN);
        s.visuals.widgets.open.bg_fill = BG_STARDUST;
        s.visuals.widgets.open.bg_stroke = Stroke::new(BORDER_WIDTH, ACCENT_CYAN);
        s.visuals.selection.bg_fill = ACCENT_CYAN.gamma_multiply(0.35);
        s.visuals.selection.stroke = Stroke::new(BORDER_WIDTH, ACCENT_CYAN);
        s.spacing.item_spacing = Vec2::new(SPACING_ELEMENT, SPACING_TIGHT);
        s.spacing.button_padding = Vec2::new(SPACING_ELEMENT, SPACING_TIGHT);
    });
}
