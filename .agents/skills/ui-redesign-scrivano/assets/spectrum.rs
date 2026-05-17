//! Enhanced audio spectrum visualizer for Scrivano
//!
//! Paints 48 vertical bars with the Nebula spectrum gradient
//! (cyan → blue → purple → magenta → rose), smooth animation,
//! peak hold indicators, and rounded bar tops.
//!
//! Usage: Copy to `src/ui/spectrum.rs`, call `paint_spectrum_bars()`
//! from the recording tab renderer.

use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use super::theme::*;

/// Paint the audio spectrum bars in the given rect
///
/// - `samples`: raw PCM data (Vec<f32>)
/// - `smooth_values`: mutable smoothed bar heights (48 elements)
/// - `peak_values`: mutable peak hold values (48 elements)
/// - `rect`: the allocated area to paint in
/// - `painter`: egui Painter for the region
pub fn paint_spectrum_bars(
    samples: &[f32],
    smooth_values: &mut [f32],
    peak_values: &mut [f32],
    rect: Rect,
    painter: &egui::Painter,
    time: f64,
) {
    let num_bars = 48;
    let max_bar_height = 35.0;
    let center_y = rect.center().y;
    let bar_width = (rect.width() - 20.0) / num_bars as f32;
    let gap = bar_width * 0.25;
    let actual_bar_width = bar_width - gap;

    // Center reference line
    painter.line_segment(
        [
            Pos2::new(rect.min.x + 5.0, center_y),
            Pos2::new(rect.right() - 5.0, center_y),
        ],
        Stroke::new(0.5, BG_ECLIPSE),
    );

    // Compute target values from audio samples
    let mut target_values = vec![0.0f32; num_bars];

    if samples.len() >= 32 {
        let samples_per_band = samples.len() / num_bars;
        for bar_idx in 0..num_bars {
            let start = bar_idx * samples_per_band;
            let end = ((bar_idx + 1) * samples_per_band).min(samples.len());
            let mut sum_sq = 0.0;
            let mut count = 0;
            for i in start..end {
                sum_sq += samples[i] * samples[i];
                count += 1;
            }
            let rms = if count > 0 { (sum_sq / count as f32).sqrt() } else { 0.0 };
            target_values[bar_idx] = (rms * 12.0).min(1.0).powf(0.6);
        }
    } else {
        // Idle animation — gentle sine waves
        let t = time as f32 * 3.0;
        for i in 0..num_bars {
            let x = i as f32 / num_bars as f32;
            target_values[i] = ((t + x * 10.0).sin() * 0.15 + 0.15).max(0.05);
        }
    }

    // Smoothing and peak hold
    let smoothing = 0.35;
    let peak_decay = 0.92;

    for i in 0..num_bars {
        let current = smooth_values[i];
        let target = target_values[i];
        let smoothed = current + (target - current) * smoothing;
        smooth_values[i] = smoothed;

        if smoothed > peak_values[i] {
            peak_values[i] = smoothed;
        } else {
            peak_values[i] *= peak_decay;
        }

        let bar_h = smoothed * max_bar_height;
        let peak_h = peak_values[i] * max_bar_height;
        let x = rect.min.x + 10.0 + i as f32 * bar_width;

        let color = spectrum_color(i as f32 / num_bars as f32);
        let brightness = 0.5 + smoothed * 0.5;
        let bar_color = Color32::from_rgb(
            (color.r() as f32 * brightness) as u8,
            (color.g() as f32 * brightness) as u8,
            (color.b() as f32 * brightness) as u8,
        );

        if bar_h > 1.0 {
            // Upper bar
            painter.rect_filled(
                Rect::from_min_max(
                    Pos2::new(x, center_y - bar_h),
                    Pos2::new(x + actual_bar_width, center_y - 1.0),
                ),
                ROUNDING_SMALL, // Rounded tops
                bar_color,
            );
            // Lower bar (mirror)
            painter.rect_filled(
                Rect::from_min_max(
                    Pos2::new(x, center_y + 1.0),
                    Pos2::new(x + actual_bar_width, center_y + bar_h),
                ),
                ROUNDING_SMALL,
                bar_color,
            );

            // Peak hold dot (if significantly above bar)
            if peak_h > bar_h + 3.0 {
                let peak_y = center_y - peak_h;
                let peak_rect = Rect::from_min_max(
                    Pos2::new(x, peak_y),
                    Pos2::new(x + actual_bar_width, peak_y + 3.0),
                );
                painter.rect_filled(peak_rect, 1.5, color);
            }
        } else {
            // Minimal bar — just a dot on the centerline
            let dot_rect = Rect::from_center_size(
                Pos2::new(x + actual_bar_width / 2.0, center_y),
                Vec2::new(actual_bar_width, 2.0),
            );
            painter.rect_filled(dot_rect, 1.0, color);
        }
    }
}

/// Paint a circular/radial spectrum variant (optional P1 feature)
///
/// Bars radiate outward from the center in a circle.
/// Not yet integrated into the main UI — kept for future use.
#[allow(dead_code)]
pub fn paint_spectrum_circular(
    samples: &[f32],
    smooth_values: &mut [f32],
    peak_values: &mut [f32],
    rect: Rect,
    painter: &egui::Painter,
    _time: f64,
) {
    let num_bars = 64;
    let center = rect.center();
    let max_radius = rect.width().min(rect.height()) / 2.0 - 5.0;
    let min_radius = max_radius * 0.25;

    if smooth_values.len() < num_bars {
        return;
    }

    // Compute target values
    let mut target_values = vec![0.0f32; num_bars];
    if samples.len() >= 32 {
        let samples_per_band = samples.len() / num_bars;
        for bar_idx in 0..num_bars {
            let start = bar_idx * samples_per_band;
            let end = ((bar_idx + 1) * samples_per_band).min(samples.len());
            let mut sum_sq = 0.0;
            let mut count = 0;
            for i in start..end {
                sum_sq += samples[i] * samples[i];
                count += 1;
            }
            let rms = if count > 0 { (sum_sq / count as f32).sqrt() } else { 0.0 };
            target_values[bar_idx] = (rms * 10.0).min(1.0).powf(0.5);
        }
    }

    let smoothing = 0.35;
    let peak_decay = 0.92;

    for i in 0..num_bars {
        let current = smooth_values[i];
        let target = target_values[i];
        let smoothed = current + (target - current) * smoothing;
        smooth_values[i] = smoothed;

        if smoothed > peak_values[i] {
            peak_values[i] = smoothed;
        } else {
            peak_values[i] *= peak_decay;
        }

        let angle = (i as f32 / num_bars as f32) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
        let bar_h = smoothed * (max_radius - min_radius);

        let inner = Pos2::new(
            center.x + angle.cos() * min_radius,
            center.y + angle.sin() * min_radius,
        );
        let outer = Pos2::new(
            center.x + angle.cos() * (min_radius + bar_h),
            center.y + angle.sin() * (min_radius + bar_h),
        );

        let color = spectrum_color(i as f32 / num_bars as f32);
        painter.line_segment([inner, outer], Stroke::new(2.5, color));
    }
}

/// Create the spectrum container frame
pub fn spectrum_container(ui: &mut Ui) -> (Rect, egui::Painter) {
    let (rect, _resp) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 90.0), Sense::hover());
    let painter = ui.painter_at(rect);
    (rect, painter)
}
