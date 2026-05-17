//! Nebula Dark — Design System Constants for Scrivano

use eframe::egui::{Color32, FontId, RichText, Stroke};

pub const BG_VOID: Color32 = Color32::from_rgb(9, 10, 15);
pub const BG_NEBULA: Color32 = Color32::from_rgb(19, 22, 32);
pub const BG_STARDUST: Color32 = Color32::from_rgb(26, 31, 46);
pub const BG_STARDUST_HOVER: Color32 = Color32::from_rgb(32, 38, 55);
pub const BG_ECLIPSE: Color32 = Color32::from_rgb(42, 48, 69);

pub const ACCENT_CYAN: Color32 = Color32::from_rgb(0, 212, 255);
pub const ACCENT_CYAN_HOVER: Color32 = Color32::from_rgb(51, 222, 255);
pub const ACCENT_CYAN_DIM: Color32 = Color32::from_rgb(0, 160, 200);
pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(168, 85, 247);
pub const ACCENT_PURPLE_HOVER: Color32 = Color32::from_rgb(188, 115, 250);
pub const ACCENT_PURPLE_DIM: Color32 = Color32::from_rgb(130, 60, 210);
pub const ACCENT_EMERALD: Color32 = Color32::from_rgb(16, 185, 129);
pub const ACCENT_EMERALD_HOVER: Color32 = Color32::from_rgb(52, 211, 153);
pub const ACCENT_AMBER: Color32 = Color32::from_rgb(245, 158, 11);
pub const ACCENT_CRIMSON: Color32 = Color32::from_rgb(239, 68, 68);
pub const ACCENT_CRIMSON_HOVER: Color32 = Color32::from_rgb(248, 100, 100);

pub const TEXT_STARLIGHT: Color32 = Color32::from_rgb(241, 245, 249);
pub const TEXT_MOON: Color32 = Color32::from_rgb(148, 163, 184);
pub const TEXT_DUST: Color32 = Color32::from_rgb(100, 116, 139);
pub const TEXT_WHITE: Color32 = Color32::WHITE;

pub const FONT_H1: f32 = 28.0;
pub const FONT_H2: f32 = 20.0;
pub const FONT_H3: f32 = 16.0;
pub const FONT_BODY: f32 = 14.0;
pub const FONT_CAPTION: f32 = 12.0;

pub const SPACING_SECTION: f32 = 24.0;
pub const SPACING_CARD_PAD: f32 = 20.0;
pub const SPACING_ELEMENT: f32 = 12.0;
pub const SPACING_TIGHT: f32 = 8.0;
pub const SPACING_MICRO: f32 = 4.0;

pub const ROUNDING_CARD: f32 = 12.0;
pub const ROUNDING_BUTTON: f32 = 10.0;
pub const ROUNDING_SMALL: f32 = 6.0;
pub const ROUNDING_PILL: f32 = 20.0;

// Layout constraints
pub const SETTINGS_CONTENT_MAX_WIDTH: f32 = 760.0;
pub const SETTINGS_FIELD_MAX_WIDTH: f32 = 420.0;
pub const SETTINGS_SIDEBAR_WIDTH: f32 = 172.0;
pub const MAIN_ACTION_BUTTON_WIDTH: f32 = 320.0;
pub const MAIN_ACTION_BUTTON_HEIGHT: f32 = 56.0;
pub const TRANSCRIPT_MAX_HEIGHT: f32 = 150.0;
pub const SPECTRUM_HEIGHT: f32 = 90.0;
pub const SPECTRUM_CANVAS_HEIGHT: f32 = 80.0;
pub const ICON_BUTTON_SIZE: f32 = 24.0;

pub fn border_stroke() -> Stroke {
    Stroke::new(1.0, BG_ECLIPSE)
}

pub fn h1(text: impl Into<String>) -> RichText {
    RichText::new(text)
        .size(FONT_H1)
        .color(TEXT_STARLIGHT)
        .strong()
}

pub fn h2(text: impl Into<String>) -> RichText {
    RichText::new(text)
        .size(FONT_H2)
        .color(TEXT_STARLIGHT)
        .strong()
}

pub fn h3(text: impl Into<String>) -> RichText {
    RichText::new(text).size(FONT_H3).color(TEXT_STARLIGHT)
}

pub fn body(text: impl Into<String>) -> RichText {
    RichText::new(text).size(FONT_BODY).color(TEXT_STARLIGHT)
}

pub fn caption(text: impl Into<String>) -> RichText {
    RichText::new(text).size(FONT_CAPTION).color(TEXT_MOON)
}

pub fn muted(text: impl Into<String>) -> RichText {
    RichText::new(text).size(FONT_CAPTION).color(TEXT_DUST)
}

pub fn mono_font() -> FontId {
    FontId::proportional(FONT_BODY)
}
