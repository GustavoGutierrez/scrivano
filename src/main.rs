mod audio;
mod audio_devices;
mod database;
mod ollama;
mod playback;
mod summarization;
mod transcription;
mod ui;

use scrivano::audio_devices::AppSettings;
use scrivano::transcription::init_whisper;
use scrivano::ui::App;

#[cfg(feature = "tray-icon")]
use tray_icon::{Icon, TrayIconBuilder};

#[cfg(feature = "tray-icon")]
fn create_tray_icon() -> tray_icon::TrayIcon {
    use image::{ImageBuffer, Rgba};
    let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(16, 16);
    for pixel in img.pixels_mut() {
        pixel.0 = [30, 180, 60, 255];
    }
    let icon = Icon::from_rgba(img.into_raw(), 16, 16).expect("Failed to create tray icon");

    TrayIconBuilder::new()
        .with_icon(icon)
        .with_tooltip("Scrivano")
        .build()
        .expect("Failed to build tray icon")
}

fn main() -> eframe::Result<()> {
    let settings = AppSettings::load();
    let model_path = settings.whisper_model.clone();

    eprintln!("[main] Cargando modelo: {}", model_path);
    let ctx = init_whisper(&model_path);

    #[cfg(feature = "tray-icon")]
    let _tray = create_tray_icon();

    #[cfg(not(feature = "tray-icon"))]
    eprintln!("[main] Tray icon deshabilitado (compila sin feature 'tray-icon')");

    eframe::run_native(
        "Scrivano",
        eframe::NativeOptions::default(),
        Box::new(move |_cc| Box::new(App::new(ctx, settings))),
    )
}
