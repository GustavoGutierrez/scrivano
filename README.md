<p align="center">
  <img src="assets/favicons/favicon-512x512.png" alt="Scrivano Logo" width="128" height="128">
</p>

# Scrivano

[![Get it from the Snap Store](https://snapcraft.io/en/dark/install.svg)](https://snapcraft.io/scrivano)

Scrivano es una app de escritorio en Rust para Linux que **graba audio del sistema** y lo **transcribe localmente con Whisper**, sin mandar audio a la nube.

Hoy está en una etapa madura: UI Nebula, historial persistente, exportación multi-formato, y mejoras opcionales con Ollama.

## Quick start

1. Instalá dependencias base:
   ```bash
   sudo apt install libpulse-dev libclang-dev
   ```
2. Bajá un modelo Whisper a `models/` (por ejemplo `ggml-small.bin`).
3. Ejecutá:
   ```bash
   cargo run --release
   ```
4. Si usás Snap:
   ```bash
   sudo snap connect scrivano:audio-record
   sudo snap connect scrivano:pulseaudio
   ```

## Estado actual

| Área | Estado actual |
|---|---|
| UI | ✅ Rediseño Nebula activo (sistema de diseño en `src/ui/theme.rs` + componentes reutilizables) |
| Grabación/transcripción | ✅ Captura PulseAudio/PipeWire + Whisper local |
| Flujo de stop | ✅ Detención robusta: countdown de UI + `join()` del thread + tail flush best-effort antes de transcribir |
| Control de sesión en grabación | ✅ Pausar, reanudar y cancelar durante grabación (incluye limpieza/cancelación de sesión de chunks) |
| Reproducción en historial | ✅ Toggle externo de play/pause/reanudar consistente por ítem (evita duplicación cuando la fila está expandida) |
| Exportación | ✅ TXT, Markdown, JSON, SRT y WebVTT |
| Historial | ✅ Persistencia SQLite + acciones desde UI |
| Snap | ✅ Disponible en Snap Store (`strict confinement`) |
| Releases GitHub | ✅ Versionadas en semver (Cargo + Snap alineados por proceso de release) |

### Baseline de validación

Estos comandos son la base mínima antes de publicar o mergear cambios relevantes:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test
```

## UI Nebula (estado real)

El frontend ya NO es "una sola pantalla monolítica". Aunque `src/ui.rs` sigue siendo el shell principal, la UI está separada por dominios:

- `src/ui/recording.rs`
- `src/ui/settings.rs`
- `src/ui/about.rs`
- `src/ui/components.rs`
- `src/ui/theme.rs`
- `src/ui/spectrum.rs`

Además, el proyecto impone reglas de diseño en `.agents/rules/design-system.md` para mantener coherencia visual, contraste y espaciado.

## Arquitectura (resumen)

| Capa | Responsabilidad | Archivos clave |
|---|---|---|
| Entrada/UI | Tabs, interacción de usuario, rendering egui | `src/ui.rs`, `src/ui/*` |
| Captura audio | Captura sistema/mic, chunks y sesión | `src/audio.rs`, `src/audio_chunker.rs`, `src/recording_session.rs` |
| Transcripción | Whisper local + segmentos | `src/transcription.rs` |
| IA opcional | Mejora/resumen con Ollama | `src/ollama.rs`, `src/summarization.rs` |
| Persistencia | Metadatos y consultas | `src/database.rs` |
| Exportación | Salida TXT/MD/JSON/SRT/VTT | `src/export.rs` |

## Build y ejecución

```bash
# Build release
cargo build --release

# Ejecutar binario
./target/release/scrivano

# Versión runtime (para gates de publicación)
./target/release/scrivano --version
./target/release/scrivano -V
```

También funciona con `cargo run`:

```bash
cargo run -- --version
```

### Features opcionales

| Feature | Requisito | Uso |
|---|---|---|
| `audio-playback` (default) | `libasound2-dev` | Reproducir audio en la app |
| `tray-icon` | `libxdo-dev` | Integración de bandeja |

```bash
cargo build --release --features "audio-playback tray-icon"
```

## Distribución

- **Snap Store**: https://snapcraft.io/scrivano
- **GitHub Releases**: https://github.com/GustavoGutierrez/scrivano/releases

Para mantenimiento de publicación, ver skills del repo:

- `.agents/skills/release-automation/SKILL.md`
- `.agents/skills/snap-publish/SKILL.md`
- `.agents/skills/security-release-gate/SKILL.md`

## Contribuir

Antes de abrir PR:

```bash
cargo fmt --check && cargo clippy --all-targets --all-features && cargo test
```

Guías completas para agentes y contribución técnica:

- `AGENTS.md`

## Licencia

MIT
