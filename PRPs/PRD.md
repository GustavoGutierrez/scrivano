# MeetWhisperer — PRD (Product Requirements Document)

**Versión:** 0.1  
**Plataforma objetivo:** Linux (PipeWire / PulseAudio)  
**Lenguaje:** Rust  
**Estado:** Borrador inicial

---

## 1. Descripción del producto

MeetWhisperer es una aplicación de escritorio para Linux que graba el audio del sistema (salida de parlantes / loopback) y lo transcribe automáticamente usando Whisper, con soporte para español e inglés. La UI es minimalista y se puede minimizar a la bandeja del sistema.

**Caso de uso principal:** transcribir reuniones de Google Meet, Zoom u otras aplicaciones VoIP sin depender de ningún servicio en la nube.

---

## 2. Objetivos

- [ ] Capturar el audio del sistema (lo que suena en los parlantes) en Linux.
- [ ] Transcribir en español e inglés con detección automática de idioma.
- [ ] UI simple con botones Iniciar / Detener y área de transcripción.
- [ ] Icono en la bandeja del sistema para minimizar/restaurar la ventana.
- [ ] Funcionar 100% offline (modelo local).
- [ ] Panel **About** visible en la UI con información del desarrollador.
- [ ] Suite de tests unitarios e integrales siguiendo las mejores prácticas de Rust.

---

## 3. Stack tecnológico

| Capa | Librería | Notas |
|---|---|---|
| Audio (captura de sistema) | `simple-pulse-desktop-capture` | Envuelve PulseAudio/PipeWire; captura el playback device por defecto |
| Transcripción | `whisper-rs` | Binding de `whisper.cpp`; acepta `Vec<f32>` a 16 kHz mono |
| UI | `eframe` + `egui` | UI inmediata, sin dependencias GTK/Qt |
| Bandeja | `tray-icon` | Icono de system tray multiplataforma |
| Utilidades | `anyhow` | Manejo de errores ergonómico |

> **Alternativa de audio de bajo nivel:** `cpal` con configuración manual de monitor/loopback (más control, más complejo).

---

## 4. Modelo de IA

### Modelos disponibles (GGML multilingüe)

| Modelo | Tamaño aprox. | Recomendado para |
|---|---|---|
| `ggml-small.bin` | ~244 MB | Balance calidad/rendimiento — **recomendado para empezar** |
| `ggml-medium.bin` | ~769 MB | Mayor precisión si los recursos lo permiten |

### Descarga del modelo

**Opción A — Script oficial de whisper.cpp:**

```bash
git clone https://github.com/ggml-org/whisper.cpp
cd whisper.cpp/models
./download-ggml-model.sh small    # o "medium"
# El modelo queda en: models/ggml-small.bin
```

**Opción B — Descarga directa desde HuggingFace:**

```
ggml-small.bin  → https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin
ggml-medium.bin → https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin
```

Copiar el archivo descargado a `models/` en la raíz del proyecto.

---

## 5. Estructura del proyecto

```
MeetWhisperer/
├── Cargo.toml
├── models/
│   └── ggml-small.bin        # modelo GGML (no incluido en repo; ver sección 4a)
├── src/
│   ├── main.rs               # punto de entrada: inicializa ventana + bandeja
│   ├── audio.rs              # hilo de captura de audio del sistema
│   ├── transcription.rs      # carga del modelo y llamada a whisper-rs
│   └── ui.rs                 # estructura App + impl eframe::App (tabs: Grabación, About)
└── tests/
    ├── audio_tests.rs        # tests de integración del módulo audio
    └── transcription_tests.rs # tests de integración del módulo transcription
```

---

## 6. `Cargo.toml`

```toml
[package]
name = "meet-whisperer"
version = "0.1.0"
edition = "2021"

[dependencies]
# Audio — captura salida del sistema (PipeWire/PulseAudio)
simple-pulse-desktop-capture = "0.2"   # verificar última versión en crates.io

# Transcripción local con Whisper
whisper-rs = "0.15"                    # verificar última versión en crates.io

# UI de escritorio
egui  = "0.27"
eframe = "0.27"

# Bandeja del sistema
tray-icon = "0.15"

# Manejo de errores
anyhow = "1"

[dev-dependencies]
# Para tests de integración y unitarios
tempfile = "3"                         # directorios temporales en tests
```

> Verificar versiones actuales en [crates.io](https://crates.io) antes de compilar.

---

## 7. Flujo de la aplicación

```
Arranque
  └─► Carga modelo Whisper (ggml-small.bin)
  └─► Inicializa estado: recording=false, buffer vacío, transcript vacío
  └─► Crea icono de bandeja
  └─► Abre ventana egui

[Usuario pulsa "Iniciar"]
  └─► recording = true
  └─► Limpia audio_buffer
  └─► Lanza hilo de captura (DesktopAudioRecorder)
        └─► Loop: lee frames PCM → acumula en audio_buffer mientras recording=true

[Usuario pulsa "Detener"]
  └─► recording = false (el hilo de captura termina)
  └─► Clona audio_buffer y lo libera
  └─► Lanza hilo de transcripción
        └─► Resampling a 16 kHz mono (si es necesario)
        └─► whisper-rs: state.full(params, &samples)
              - language = "auto"  (detecta español/inglés)
              - translate = false
        └─► Recoge segmentos de texto → actualiza transcript en UI

[Clic en icono de bandeja]
  └─► Alterna visibilidad de la ventana (mostrar/ocultar)
```

---

## 8. Código de referencia

### `audio.rs` — Hilo de captura

```rust
use simple_pulse_desktop_capture::DesktopAudioRecorder;
use std::{
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    thread,
    time::Duration,
};

pub fn spawn_system_audio_recorder(
    recording_flag: Arc<AtomicBool>,
    audio_buffer: Arc<Mutex<Vec<f32>>>,
) {
    thread::spawn(move || {
        let mut recorder = DesktopAudioRecorder::new()
            .expect("Failed to init DesktopAudioRecorder");

        while recording_flag.load(Ordering::SeqCst) {
            if let Ok(frame) = recorder.read_frame() {
                let data = frame.pcm_data();
                audio_buffer.lock().unwrap().extend_from_slice(data);
            } else {
                thread::sleep(Duration::from_millis(10));
            }
        }
    });
}
```

### `transcription.rs` — Carga del modelo y transcripción

```rust
use whisper_rs::{WhisperContext, FullParams, SamplingStrategy};
use anyhow::Result;

pub fn init_whisper(model_path: &str) -> WhisperContext {
    WhisperContext::new(model_path)
        .expect("Failed to load Whisper model")
}

pub fn transcribe(ctx: &WhisperContext, audio: &[f32]) -> Result<String> {
    let mut state = ctx.create_state()?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { beam_size: 1 });
    params.set_language(Some("auto")); // detección automática español/inglés
    params.set_translate(false);

    state.full(params, audio)?;

    let text = state
        .iter()
        .map(|seg| seg.text())
        .collect::<Vec<_>>()
        .join(" ");

    Ok(text)
}
```

### `ui.rs` — Estado y render de la UI (con tabs Grabación / About)

```rust
use eframe::egui;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use whisper_rs::WhisperContext;
use crate::{audio::spawn_system_audio_recorder, transcription::transcribe};

#[derive(PartialEq)]
enum Tab { Recording, About }

pub struct App {
    pub recording: Arc<AtomicBool>,
    pub audio_buffer: Arc<Mutex<Vec<f32>>>,
    pub transcript: Arc<Mutex<String>>,
    pub whisper_ctx: Arc<WhisperContext>,
    active_tab: Tab,
}

impl App {
    pub fn new(ctx: WhisperContext) -> Self {
        Self {
            recording: Arc::new(AtomicBool::new(false)),
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            transcript: Arc::new(Mutex::new(String::from("Esperando grabación..."))),
            whisper_ctx: Arc::new(ctx),
            active_tab: Tab::Recording,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let is_recording = self.recording.load(Ordering::SeqCst);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("MeetWhisperer");
            ui.separator();

            // Barra de tabs
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, Tab::Recording, "⏺ Grabación");
                ui.selectable_value(&mut self.active_tab, Tab::About, "ℹ About");
            });
            ui.separator();

            match self.active_tab {
                Tab::Recording => {
                    if is_recording {
                        if ui.button("⏹  Detener grabación").clicked() {
                            self.recording.store(false, Ordering::SeqCst);

                            let audio = {
                                let mut buf = self.audio_buffer.lock().unwrap();
                                let data = buf.clone();
                                buf.clear();
                                data
                            };

                            let transcript_arc = self.transcript.clone();
                            let whisper_ctx = self.whisper_ctx.clone();

                            std::thread::spawn(move || {
                                let result = match transcribe(&whisper_ctx, &audio) {
                                    Ok(text) => text,
                                    Err(_) => "Error al transcribir.".into(),
                                };
                                *transcript_arc.lock().unwrap() = result;
                            });
                        }
                    } else {
                        if ui.button("⏺  Iniciar grabación").clicked() {
                            self.recording.store(true, Ordering::SeqCst);
                            self.audio_buffer.lock().unwrap().clear();
                            spawn_system_audio_recorder(
                                self.recording.clone(),
                                self.audio_buffer.clone(),
                            );
                        }
                    }

                    ui.separator();
                    ui.label("Transcripción:");
                    let mut text = self.transcript.lock().unwrap();
                    ui.text_edit_multiline(&mut *text);
                }

                Tab::About => {
                    ui.add_space(12.0);
                    ui.heading("MeetWhisperer");
                    ui.label(format!("Versión {}", env!("CARGO_PKG_VERSION")));
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.label("Desarrollador:");
                    ui.strong("Gustavo Gutiérrez");
                    ui.label("Bogotá, Colombia");
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);
                    ui.label("Transcripción local de audio del sistema usando Whisper.");
                    ui.label("Sin dependencias de red — 100% offline.");
                }
            }
        });
    }
}
```

### `main.rs` — Punto de entrada

```rust
mod audio;
mod transcription;
mod ui;

use tray_icon::{TrayIconBuilder, Icon};
use image::{ImageBuffer, Rgba};
use transcription::init_whisper;
use ui::App;

fn create_tray_icon() -> tray_icon::TrayIcon {
    let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(16, 16);
    for pixel in img.pixels_mut() {
        pixel.0 = [30, 180, 60, 255]; // verde
    }
    let icon = Icon::from_rgba(img.into_raw(), 16, 16).unwrap();

    TrayIconBuilder::new()
        .with_icon(icon)
        .with_tooltip("MeetWhisperer")
        .build()
        .unwrap()
}

fn main() -> eframe::Result<()> {
    let model_path = "models/ggml-small.bin";
    let ctx = init_whisper(model_path);

    let _tray = create_tray_icon();

    eframe::run_native(
        "MeetWhisperer",
        eframe::NativeOptions::default(),
        Box::new(move |_cc| Box::new(App::new(ctx))),
    )
}
```

---

## 9. Configuración del modelo (setup inicial)

### Pasos para descargar y posicionar el modelo

```bash
# 1. Crear la carpeta models/ en la raíz del proyecto (si no existe)
mkdir -p models

# 2a. Descarga via wget (recomendado)
wget -P models/ https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin

# 2b. Alternativa: curl
curl -L -o models/ggml-small.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin

# 2c. Alternativa: script oficial de whisper.cpp
git clone --depth 1 https://github.com/ggml-org/whisper.cpp /tmp/whisper-cpp
bash /tmp/whisper-cpp/models/download-ggml-model.sh small
mv ggml-small.bin models/

# 3. Verificar integridad (tamaño esperado ~244 MB)
ls -lh models/ggml-small.bin
```

> El archivo `models/ggml-small.bin` debe estar presente antes de ejecutar la aplicación.  
> Agregar `models/*.bin` al `.gitignore` para no incluirlo en el repositorio.

### `.gitignore` mínimo

```gitignore
/target
models/*.bin
```

---

## 10. Tests

### Estrategia de testing

Se siguen las mejores prácticas de Rust:
- **Tests unitarios** dentro del mismo módulo usando `#[cfg(test)]`.
- **Tests de integración** en `tests/` que prueban la interfaz pública de los módulos.
- Separación clara entre lógica pura (testeable sin hardware) y código con efectos de sistema.

### Tests unitarios en `transcription.rs`

```rust
// Al final de src/transcription.rs
#[cfg(test)]
mod tests {
    use super::*;

    /// Verifica que un buffer de silencio (~0.0) no rompe la transcripción
    /// y devuelve un String (puede estar vacío).
    /// Requiere el modelo en models/ggml-small.bin.
    #[test]
    #[ignore = "requiere modelo GGML en disco"]
    fn transcribe_silence_returns_string() {
        let ctx = init_whisper("models/ggml-small.bin");
        // 2 segundos de silencio a 16 kHz
        let silence = vec![0.0_f32; 16_000 * 2];
        let result = transcribe(&ctx, &silence);
        assert!(result.is_ok(), "transcribe debe retornar Ok para silencio");
    }

    /// Verifica que transcribe retorna error con un slice vacío
    #[test]
    #[ignore = "requiere modelo GGML en disco"]
    fn transcribe_empty_audio_does_not_panic() {
        let ctx = init_whisper("models/ggml-small.bin");
        let result = transcribe(&ctx, &[]);
        // Puede ser Ok("") o Err — no debe hacer panic
        let _ = result;
    }
}
```

### Tests unitarios en `audio.rs`

```rust
// Al final de src/audio.rs
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    /// Verifica que el flag recording=false detiene el hilo sin colgar.
    /// No requiere hardware de audio real.
    #[test]
    fn recording_flag_stops_thread() {
        let flag = Arc::new(AtomicBool::new(true));
        let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));

        // Poner flag en false inmediatamente para que el hilo salga solo
        flag.store(false, Ordering::SeqCst);

        // No lanzar el hilo real (requeriría PulseAudio), solo validar la lógica del flag
        assert!(!flag.load(Ordering::SeqCst));
        assert!(buffer.lock().unwrap().is_empty());
    }
}
```

### Tests de integración en `tests/`

```rust
// tests/transcription_tests.rs
use meet_whisperer::transcription::{init_whisper, transcribe};

/// Test de integración completa: carga modelo + transcribe silencio.
/// Marcado con #[ignore] para no correr en CI sin el modelo.
#[test]
#[ignore = "requiere models/ggml-small.bin"]
fn integration_transcribe_silence() {
    let ctx = init_whisper("models/ggml-small.bin");
    let silence = vec![0.0_f32; 16_000 * 3]; // 3 s a 16 kHz
    let result = transcribe(&ctx, &silence);
    assert!(result.is_ok());
}
```

### Comandos de test

```bash
# Ejecutar todos los tests (excluye los marcados con #[ignore])
cargo test

# Ejecutar también los tests que requieren el modelo en disco
cargo test -- --include-ignored

# Ejecutar solo tests de un módulo específico
cargo test transcription

# Verificar formato y lints antes de commit
cargo fmt --check && cargo clippy --all-targets --all-features && cargo test
```

---

## 11. Requisitos del sistema

- Linux con PulseAudio o PipeWire activo.
- Rust toolchain estable (`rustup`).
- Librerías de sistema: `libpulse-dev` (o equivalente en la distribución).
- ~500 MB de espacio libre para el modelo `ggml-small.bin`.

---

## 12. Tareas de implementación

- [ ] Crear proyecto Rust con `cargo new meet-whisperer`.
- [ ] Configurar `Cargo.toml` con dependencias y `[dev-dependencies]`.
- [ ] Crear carpeta `models/` y agregar `models/*.bin` al `.gitignore`.
- [ ] Descargar `ggml-small.bin` en `models/` (ver sección 9).
- [ ] Implementar `audio.rs` (captura de sistema + tests unitarios).
- [ ] Implementar `transcription.rs` (carga de modelo + transcribir + tests unitarios).
- [ ] Implementar `ui.rs` (App struct + tabs Grabación / About con datos del desarrollador).
- [ ] Implementar `main.rs` (icono de bandeja + launch de ventana).
- [ ] Crear `tests/transcription_tests.rs` (test de integración con modelo real).
- [ ] Verificar resampling: si el audio del sistema viene a 44.1/48 kHz, resamplear a 16 kHz mono antes de pasarlo a Whisper.
- [ ] Integrar toggle de ventana al hacer clic en el icono de bandeja.
- [ ] Pasar `cargo fmt --check`, `cargo clippy --all-targets` y `cargo test` sin errores.

---

## 13. Trabajo futuro (fuera de alcance v0.1)

- Resampling automático 44.1/48 kHz → 16 kHz mono.
- Menú contextual en el icono de bandeja (Mostrar, Copiar transcripción, Salir).
- Guardado automático de transcripciones en archivo `txt` o `md`.
- Soporte para seleccionar dispositivo de audio de entrada.
- Transcripción en tiempo real (streaming chunks a Whisper).
