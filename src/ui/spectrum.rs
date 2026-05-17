//! Spectrum rendering helpers.

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

use super::theme::*;

const SPECTRUM_GRADIENT: [(f32, Color32); 5] = [
    (0.0, Color32::from_rgb(0, 255, 255)),
    (0.25, Color32::from_rgb(59, 130, 246)),
    (0.5, Color32::from_rgb(139, 92, 246)),
    (0.75, Color32::from_rgb(236, 72, 153)),
    (1.0, Color32::from_rgb(244, 63, 94)),
];

fn spectrum_color(t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    for i in 0..SPECTRUM_GRADIENT.len() - 1 {
        let (t0, c0) = SPECTRUM_GRADIENT[i];
        let (t1, c1) = SPECTRUM_GRADIENT[i + 1];
        if t >= t0 && t <= t1 {
            let p = (t - t0) / (t1 - t0);
            return Color32::from_rgb(
                (c0.r() as f32 * (1.0 - p) + c1.r() as f32 * p) as u8,
                (c0.g() as f32 * (1.0 - p) + c1.g() as f32 * p) as u8,
                (c0.b() as f32 * (1.0 - p) + c1.b() as f32 * p) as u8,
            );
        }
    }
    SPECTRUM_GRADIENT[SPECTRUM_GRADIENT.len() - 1].1
}

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

    painter.line_segment(
        [
            Pos2::new(rect.min.x + 5.0, center_y),
            Pos2::new(rect.right() - 5.0, center_y),
        ],
        Stroke::new(0.5, BG_ECLIPSE),
    );

    let mut target_values = vec![0.0f32; num_bars];
    if samples.len() >= 32 {
        let samples_per_band = samples.len() / num_bars;
        for (bar_idx, target_value) in target_values.iter_mut().enumerate() {
            let start = bar_idx * samples_per_band;
            let end = ((bar_idx + 1) * samples_per_band).min(samples.len());
            let mut sum_sq = 0.0;
            let mut count = 0;
            for sample in samples.iter().take(end).skip(start) {
                sum_sq += sample * sample;
                count += 1;
            }
            let rms = if count > 0 {
                (sum_sq / count as f32).sqrt()
            } else {
                0.0
            };
            *target_value = (rms * 12.0).min(1.0).powf(0.6);
        }
    } else {
        let t = time as f32 * 3.0;
        for (i, target_value) in target_values.iter_mut().enumerate() {
            let x = i as f32 / num_bars as f32;
            *target_value = ((t + x * 10.0).sin() * 0.15 + 0.15).max(0.05);
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
            painter.rect_filled(
                Rect::from_min_max(
                    Pos2::new(x, center_y - bar_h),
                    Pos2::new(x + actual_bar_width, center_y - 1.0),
                ),
                ROUNDING_SMALL,
                bar_color,
            );
            painter.rect_filled(
                Rect::from_min_max(
                    Pos2::new(x, center_y + 1.0),
                    Pos2::new(x + actual_bar_width, center_y + bar_h),
                ),
                ROUNDING_SMALL,
                bar_color,
            );

            if peak_h > bar_h + 3.0 {
                let peak_y = center_y - peak_h;
                let peak_rect = Rect::from_min_max(
                    Pos2::new(x, peak_y),
                    Pos2::new(x + actual_bar_width, peak_y + 3.0),
                );
                painter.rect_filled(peak_rect, 1.5, color);
            }
        } else {
            let dot_rect = Rect::from_center_size(
                Pos2::new(x + actual_bar_width / 2.0, center_y),
                Vec2::new(actual_bar_width, 2.0),
            );
            painter.rect_filled(dot_rect, 1.0, color);
        }
    }
}
