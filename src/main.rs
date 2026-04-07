mod audio;
mod audio_devices;
mod database;
mod ollama;
mod transcription;
mod ui;

use image::{ImageBuffer, Rgba};
use meet_whisperer::audio_devices::AppSettings;
use meet_whisperer::transcription::init_whisper;
use meet_whisperer::ui::App;
use tray_icon::{Icon, TrayIconBuilder};

fn create_tray_icon() -> tray_icon::TrayIcon {
    let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(16, 16);
    for pixel in img.pixels_mut() {
        pixel.0 = [30, 180, 60, 255];
    }
    let icon = Icon::from_rgba(img.into_raw(), 16, 16).expect("Failed to create tray icon");

    TrayIconBuilder::new()
        .with_icon(icon)
        .with_tooltip("MeetWhisperer")
        .build()
        .expect("Failed to build tray icon")
}

fn main() -> eframe::Result<()> {
    let settings = AppSettings::load();
    let model_path = settings.whisper_model.clone();

    eprintln!("[main] Cargando modelo: {}", model_path);
    let ctx = init_whisper(&model_path);

    let _tray = create_tray_icon();

    eframe::run_native(
        "MeetWhisperer",
        eframe::NativeOptions::default(),
        Box::new(move |_cc| Box::new(App::new(ctx, settings))),
    )
}
