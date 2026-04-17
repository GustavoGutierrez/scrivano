<p align="center">
  <img src="assets/favicons/favicon-512x512.png" alt="Scrivano Logo" width="128" height="128">
</p>

# Scrivano

[![Get it from the Snap Store](https://snapcraft.io/en/dark/install.svg)](https://snapcraft.io/scrivano)

### Instalación desde Snap Store

Después de instalar desde la Snap Store, concede los permisos de audio ejecutando:

```bash
sudo snap connect scrivano:audio-record
sudo snap connect scrivano:pulseaudio
```

<p align="center">
  <strong>Scrivano</strong> es una aplicación de escritorio escrita en Rust que graba el **audio del sistema en Linux** (por ejemplo, llamadas de Google Meet, videollamadas, conferencias online) y lo transcribe automáticamente usando **Whisper** en modo local, sin enviar tu audio a la nube.

La app captura directamente la salida de audio del sistema mediante PulseAudio/PipeWire, procesa el audio con modelos GGML de Whisper (`ggml-small.bin` o `ggml-medium.bin`) y muestra la transcripción en una interfaz sencilla construida con `egui`. Además, se integra en la bandeja del sistema para poder iniciar y detener grabaciones de forma rápida y discreta mientras trabajas.

---

## Estado del Proyecto

| Componente | Estado |
|------------|--------|
| Compilación | ✅ Estable |
| Tests | 40 tests (27 passed, 13 ignored) |
| Modelo Whisper | ✅ Carga correctamente (189.49 MB) |
| Interfaz GUI | ✅ Implementada con UX mejorada |
| Exportación | ✅ TXT, MD, JSON, SRT, VTT |
| Highlights | ✅ Botón durante grabación |
| Resúmenes | ✅ Ejecutivo, Tareas, Decisiones |
| Configuración Ollama | ✅ Host/Port personalizables |
| Ollama (opcional) | ✅ Soportado con thinking cleanup |

### Novedades Recientes
- **✨ UX Rediseñada**: Pantalla de configuración con tarjetas, padding y mejor organización
- **🔧 Configuración Ollama**: Ahora puedes configurar host y puerto personalizados
- **📥 Exportar desde Historial**: Botones de exportar y generar resumen en grabaciones anteriores
- **🎨 Mejor diseño visual**: Secciones con fondo, bordes redondeados y espaciado apropiado

---

## Interfaz de Usuario

### Pantalla Principal (Grabación)

```
┌─────────────────────────────────────────────────────────────┐
│  [Grabación] [Configuración] [Acerca de]                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│              ⏺  Iniciar grabación                          │
│                                                             │
│     ~~~~~~~~ Waveform Visualizer ~~~~~~~~                   │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  Transcripción:                                             │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ Texto transcrito aparece aquí...                        ││
│  └─────────────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────────────┤
│  📥 Exportar: [TXT] [Markdown] [JSON] [SRT] [VTT]          │
├─────────────────────────────────────────────────────────────┤
│  ✨ Generar Resumen: [📋 Ejecutivo] [✅ Tareas] [📝 Decisiones]│
├─────────────────────────────────────────────────────────────┤
│  ▼ Grabaciones recientes (12)                              │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ 📄 grabacion_001.txt                                    ││
│  │ 📅 2024-01-15  ⏱ 15:30  ✨ qwen3.5:4b                  ││
│  │ ─────────────────────────────────────────────────────   ││
│  │ [📥 Exportar] [✨ Resumen] [📄 Abrir]                   ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### Durante Grabación Activa

```
┌─────────────────────────────────────────────────────────────┐
│  [Grabación] [Configuración] [Acerca de]                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│              ⏹  Detener     |     ⭐ Highlight             │
│                                                             │
│     ● REC   00:05:23    (2 highlights marcados)            │
│                                                             │
│     ~~~~~~~~ waveform ~~~~~~~~                              │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Pantalla de Configuración (Rediseñada)

```
┌─────────────────────────────────────────────────────────────┐
│  [Grabación] [Configuración] [Acerca de]                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  🎙  Modelo de transcripción                               │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ Selecciona un modelo Whisper:                           ││
│  │ [ggml-small-q5_1.bin                    ▼]             ││
│  └─────────────────────────────────────────────────────────┘│
│                                                             │
│  🎤  Dispositivos de Audio                                 │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ Micrófono:                                              ││
│  │ [Monitor of Built-in Audio Analog Stereo ▼]             ││
│  │                                                         ││
│  │ Audio del sistema:                                      ││
│  │ [Monitor of Built-in Audio Analog Stereo ▼]             ││
│  │ La captura usa el monitor del sink de PulseAudio...     ││
│  └─────────────────────────────────────────────────────────┘│
│                                                             │
│  ✨  Configuración de Ollama                               │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ Servidor Ollama:                                        ││
│  │ Host: [localhost               ]  Puerto: [11434 ]     ││
│  │                                                         ││
│  │ ● Ollama disponible                                     ││
│  │                                                         ││
│  │ [🔄 Probar conexión]                                    ││
│  │                                                         ││
│  │ ☑ Habilitar mejora con Ollama                          ││
│  │ ─────────────────────────────────────────────────────   ││
│  │ Modelo para generar resúmenes:                          ││
│  │ [qwen3.5:4b                               ▼]           ││
│  │                                                         ││
│  │ [↻ Actualizar lista de modelos]                        ││
│  └─────────────────────────────────────────────────────────┘│
│                                                             │
│  📁  Carpeta de grabaciones                                │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ Ruta donde se guardarán las grabaciones:                ││
│  │ [/home/user/Scrivano/recordings                     ]  ││
│  └─────────────────────────────────────────────────────────┘│
│                                                             │
│              [  💾  Guardar configuración  ]                │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Características de la UI:**
- ✅ **Tarjetas con padding**: Cada sección tiene fondo, borde y margen interno de 16px
- ✅ **Espaciado consistente**: 20px entre secciones, 8-16px entre elementos
- ✅ **Configuración Ollama completa**: Host y puerto editables con botón de prueba
- ✅ **Combo boxes mejorados**: Ancho ajustado con padding interno
- ✅ **Botón de guardar prominente**: Tamaño aumentado con icono

---

## Características Implementadas

### ✅ Grabación de Audio (FR-001, FR-002)
- Captura de audio del sistema (loopback) + micrófono opcional
- Botón iniciar/detener grabación con indicador visual
- Estado en tiempo real con waveform animado
- Contador de tiempo y highlights durante grabación

### ✅ Highlights (FR-004)
- Botón "⭐ Highlight" durante la grabación
- Registro automático de timestamp
- Visualización del contador en UI

### ✅ Transcripción Local (FR-005, FR-007)
- Whisper embebido en Rust (whisper-rs)
- Segmentación temporal con start/end
- Soporte español e inglés
- Progreso visual durante transcripción

### ✅ Resúmenes con Ollama (FR-008, FR-009)
- **Resumen Ejecutivo**: Síntesis de puntos principales
- **Tareas**: Extracción de tareas asignadas
- **Decisiones**: Lista de decisiones tomadas
- Limpieza automática de respuestas "thinking"
- Soporte para modelos reasoning (deepseek-r1, qwen3, etc.)

### ✅ Exportación (FR-012)
- **Desde transcripción actual**: TXT, Markdown, JSON, SRT, WebVTT
- **Desde historial**: Botón "📥 Exportar" en cada grabación
- Formato Markdown con metadatos (fecha, duración)

### ✅ Configuración de Ollama (FR-014)
- **Host configurable**: Por defecto `localhost`, editable
- **Puerto configurable**: Por defecto `11434`, editable
- **Botón de prueba**: Verifica conectividad con el servidor
- **Lista de modelos**: Selección desde modelos instalados
- **Persistencia**: Guarda en settings.toml

### ✅ Historial de Grabaciones (FR-011)
- Lista de grabaciones con metadatos
- Filtros visuales por fecha y duración
- **Acciones por grabación**:
  - 📄 Abrir archivo
  - 📥 Exportar a Markdown
  - ✨ Generar resumen (si Ollama está disponible)

### ✅ Persistencia (FR-013)
- SQLite3 para metadatos
- Sistema de archivos para grabaciones
- Archivo de configuración TOML

### ✅ Offline (FR-015)
- Funciona sin Internet
- Whisper local no requiere red
- Ollama opcional

---

## Requisitos del Sistema

### Dependencias del Sistema

| Paquete | Para qué sirve |
|---------|----------------|
| `libpulse-dev` | Captura de audio del sistema |
| `libclang-dev` | Compilación de whisper-rs |
| `libasound2-dev` | Reproducción de audio (rodio) |
| `libxdo-dev` (opcional) | Icono en bandeja del sistema |

```bash
# Instalar dependencias en Ubuntu/Debian
# Básico (sin audio playback ni tray)
sudo apt install libpulse-dev libclang-dev

# Con reproducción de audio (requiere rodio)
sudo apt install libpulse-dev libasound2-dev libclang-dev

# Completo (con tray-icon)
sudo apt install libpulse-dev libasound2-dev libclang-dev libxdo-dev
```

### Modelos Whisper

Descarga modelos de https://github.com/ggml-org/whisper.cpp:

```bash
mkdir -p models
# Descarga uno de estos:
# - ggml-small.bin (~487 MB) - Recomendado
# - ggml-medium.bin (~1.5 GB) - Mejor calidad
```

---

## Compilar y Ejecutar

### Compilación

```bash
# Básico (sin features opcionales)
cargo build --release --no-default-features

# Con reproducción de audio (requiere libasound2-dev)
cargo build --release --features audio-playback

# Con bandeja del sistema (requiere libxdo-dev)
cargo build --release --features tray-icon

# Completo: audio + bandeja
cargo build --release --features "audio-playback tray-icon"
```

### Ejecución

```bash
./target/release/scrivano
```

---

## Flujo de Uso

### 1. Primera Ejecución
1. Abre Scrivano
2. Ve a **Configuración**
3. Selecciona tu modelo Whisper
4. Configura dispositivos de audio
5. (Opcional) Configura Ollama host/puerto y prueba conexión
6. Guarda configuración

### 2. Durante una Reunión
1. Haz clic en **⏺ Iniciar grabación**
2. El waveform muestra actividad de audio
3. Presiona **⭐ Highlight** en momentos importantes
4. Presiona **⏹ Detener** al terminar
5. Espera la transcripción (barra de progreso)

### 3. Después de la Transcripción
1. Revisa el texto transcrito
2. Exporta en el formato deseado (TXT, MD, JSON, SRT, VTT)
3. Genera un resumen (Ejecutivo, Tareas o Decisiones) si tienes Ollama
4. La grabación se guarda automáticamente en el historial

### 4. Desde el Historial
1. Haz clic en **▼ Grabaciones recientes** para expandir
2. Cada grabación muestra:
   - Nombre, fecha y duración
   - Botón **📥 Exportar** - Crea archivo Markdown
   - Botón **✨ Resumen** - Genera resumen con Ollama
   - Botón **📄 Abrir** - Abre el archivo original

---

## Problemas Conocidos

### Error "WinitEventLoop(NotSupported)"
- **Causa**: No hay servidor graphical disponible
- **Solución**: Usar en máquina con GUI o configurar Xvfb

### Error "unable to find library -lxdo"
- **Solución**: `sudo apt install libxdo-dev` o compilar sin features

### Timestamps en exportación
- **Estado**: Actualmente los archivos exportados muestran 00:00:00
- **Razón**: Los segmentos con timestamps de Whisper no se pasan correctamente desde el thread de transcripción
- **Workaround**: La transcripción se exporta completa con metadatos

---

## Tecnologías

- **Rust 2021** - Lenguaje
- **egui/eframe 0.27** - UI framework
- **whisper-rs 0.15** - Transcripción local
- **libpulse-binding** - Captura de audio
- **rusqlite** - Base de datos
- **Ollama** - Resúmenes (opcional)

---

## Estructura del Proyecto

```
Scrivano/
├── Cargo.toml
├── models/
│   └── ggml-*.bin              # Modelos Whisper
├── src/
│   ├── main.rs                 # Punto de entrada
│   ├── lib.rs                  # Exports
│   ├── audio.rs                # Captura de audio
│   ├── audio_devices.rs        # Configuración y settings
│   ├── transcription.rs        # Whisper integration
│   ├── ollama.rs               # Cliente Ollama
│   ├── summarization.rs        # Generación de resúmenes
│   ├── database.rs             # SQLite persistence
│   ├── export.rs               # Exportación multi-formato
│   └── ui.rs                   # Interfaz egui (1700+ líneas)
├── tests/
│   ├── transcription_tests.rs
│   └── export_tests.rs
└── README.md
```

---

## Contributing

### Reglas de Testing (OBLIGATORIO)

Todo cambio significativo debe incluir tests:

```bash
# Verificar antes de commit
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test
```

Ver `AGENTS.md` para requisitos completos de testing.

---

## Licencia

Desarrollado por Gustavo Gutiérrez - Bogotá, Colombia