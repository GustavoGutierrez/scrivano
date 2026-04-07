# MeetWhisperer

MeetWhisperer es una aplicación de escritorio escrita en Rust que graba el **audio del sistema en Linux** (por ejemplo, llamadas de Google Meet, videollamadas, conferencias online) y lo transcribe automáticamente usando **Whisper** en modo local, sin enviar tu audio a la nube.

La app captura directamente la salida de audio del sistema mediante PulseAudio/PipeWire, procesa el audio con modelos GGML de Whisper (`ggml-small.bin` o `ggml-medium.bin`) y muestra la transcripción en una interfaz sencilla construida con `egui`. Además, se integra en la bandeja del sistema para poder iniciar y detener grabaciones de forma rápida y discreta mientras trabajas. [web:24][web:65][web:107]

---

## Características

- Grabación del **audio del sistema** (salida por defecto de PulseAudio/PipeWire), ideal para:
  - Google Meet, Zoom, Teams, Jitsi.
  - Video‑clases, webinars, conferencias online.
- Transcripción local con **Whisper** usando modelos GGML (`ggml-small.bin` / `ggml-medium.bin`). [web:63][web:65]
- Soporte para **español e inglés mezclados** en la misma conversación (Whisper en modo `language=auto`). [web:75][web:79]
- Interfaz gráfica minimalista con `egui`:
  - Botón “Iniciar grabación”.
  - Botón “Detener grabación y transcribir”.
  - Área de texto con la transcripción resultante.
- Integración con la **bandeja del sistema** mediante `tray-icon`:
  - Icono en la barra de estado.
  - Posibilidad de mantener la ventana minimizada y controlar la grabación desde la bandeja. [web:29][web:38]
- Funciona **totalmente offline** una vez descargado el modelo de Whisper. [web:65]

---

## Requisitos

### Sistema operativo

- Linux con:
  - **PulseAudio o PipeWire** para la captura de audio del sistema. [web:107][web:110]

### Hardware recomendado

- CPU x86_64 relativamente reciente.
- Mínimo **8 GB de RAM** (recomendado si usas `ggml-medium.bin`). [web:42][web:46]
- Opcional: GPU con 8 GB de VRAM si quieres usar versiones aceleradas de Whisper (por ejemplo, `faster-whisper` fuera de este binario). [web:52][web:88]

---

## Modelos de Whisper soportados

MeetWhisperer está pensado para usar los modelos GGML de `whisper.cpp`. [web:65][web:109]

Modelos recomendados:

- **`ggml-small.bin` (multilingüe)**  
  - Buena calidad en español e inglés.  
  - Consumo de memoria moderado. [web:46][web:78]

- **`ggml-medium.bin` (multilingüe)**  
  - Mejor precisión que `small`, especialmente en frases largas y ruido moderado.  
  - Recomendado si tienes ≥ 8 GB de RAM disponible. [web:46][web:78]

### Descarga de modelos

Puedes descargar los modelos con el script oficial de `whisper.cpp`: [web:109]

```bash
git clone https://github.com/ggml-org/whisper.cpp
cd whisper.cpp/models

# Para ggml-small.bin
./download-ggml-model.sh small

# Para ggml-medium.bin
./download-ggml-model.sh medium

Esto dejará los modelos en whisper.cpp/models/ggml-small.bin y whisper.cpp/models/ggml-medium.bin. [web:109][web:65]

Después, cópialos a la carpeta models/ de tu proyecto:

mkdir -p models
cp /ruta/a/whisper.cpp/models/ggml-small.bin models/
# o
cp /ruta/a/whisper.cpp/models/ggml-medium.bin models/

---

## Compilar y empaquetar

### Requisitos del entorno de desarrollo

| Herramienta | Instalación |
|-------------|-------------|
| Rust (stable) | `curl https://sh.rustup.rs -sSf \| sh` |
| `fakeroot` | `sudo apt install fakeroot` |
| `dpkg-deb` | `sudo apt install dpkg` *(viene con Ubuntu)* |
| `libpulse-dev` | `sudo apt install libpulse-dev` |
| LLVM 14 | Se descarga automáticamente en el primer build |

> **LLVM 14** es necesario para compilar `whisper-rs`. El `Makefile` y el `dist.sh` lo descargan solos en `/tmp/` si no está presente.

---

## Makefile — referencia de comandos

```bash
make              # alias de make release
make release      # compila el binario optimizado (cargo build --release)
make build        # compila en modo debug (cargo build)
make deb          # compila + genera el paquete .deb
make deb-only     # genera el .deb sin recompilar
make dist         # genera el tarball portable .tar.gz
make install      # instala directamente en el sistema (requiere sudo)
make uninstall    # desinstala del sistema (requiere sudo)
make check        # inspecciona el .deb generado (info + contenido)
make clean        # elimina target/, dist/ y packaging/
make clean-pkg    # elimina solo dist/ y packaging/
make help         # muestra todos los comandos disponibles
```

### Flujo típico de desarrollo

```bash
# 1. Compilar y probar en debug
make build
./target/debug/meet-whisperer

# 2. Compilar release y probar
make release
./target/release/meet-whisperer

# 3. Generar el instalador .deb
make deb

# 4. Instalar y verificar
sudo dpkg -i packaging/meet-whisperer_0.1.0_amd64.deb
meet-whisperer
```

---

## Generar el paquete .deb

El script `package-deb.sh` automatiza la creación del instalador `.deb` para Ubuntu/Debian.

### Uso directo

```bash
# Compilar y empacar en un solo paso
./package-deb.sh

# Solo empacar (asume que target/release/meet-whisperer ya existe)
./package-deb.sh --no-build
```

### Qué hace el script paso a paso

| Paso | Acción |
|------|--------|
| 1 | Verifica que `dpkg-deb` y `cargo` estén instalados |
| 2 | Compila el binario en modo release con `cargo build --release` |
| 3 | Crea la estructura de directorios del paquete en `packaging/` |
| 4 | Copia el binario a `/opt/meet-whisperer/` |
| 5 | Copia las librerías compartidas (`ldd`) a `/opt/meet-whisperer/lib/` |
| 6 | Copia los modelos Whisper `.bin` a `/opt/meet-whisperer/models/` |
| 7 | Crea `run.sh` (lanzador que configura `LD_LIBRARY_PATH`) |
| 8 | Instala los íconos en `/usr/share/icons/hicolor/{16..512}px/` |
| 9 | Crea la entrada `.desktop` para el launcher del sistema |
| 10 | Genera `DEBIAN/control` con metadatos y dependencias |
| 11 | Genera `postinst`, `prerm`, `postrm` (hooks de instalación) |
| 12 | Calcula `md5sums` para verificación de integridad |
| 13 | Construye el `.deb` con `fakeroot dpkg-deb --build` |

### Resultado

```
packaging/
└── meet-whisperer_0.1.0_amd64.deb    ← instalador final (~670 MB)
```

### Instalar el .deb en Ubuntu

```bash
# Instalar
sudo dpkg -i packaging/meet-whisperer_0.1.0_amd64.deb

# Si faltan dependencias del sistema
sudo apt-get install -f

# Verificar instalación
meet-whisperer          # desde terminal
# o busca "MeetWhisperer" en el launcher de aplicaciones
```

### Desinstalar

```bash
sudo dpkg -r meet-whisperer          # desinstala (conserva config de usuario)
sudo dpkg --purge meet-whisperer     # desinstala + elimina /opt/meet-whisperer
```

### Estructura instalada en el sistema

```
/opt/meet-whisperer/
├── meet-whisperer          ← binario principal
├── run.sh                  ← lanzador con LD_LIBRARY_PATH
├── lib/                    ← librerías bundled (portabilidad entre distros)
│   ├── libpulse.so.0
│   ├── libstdc++.so.6
│   └── ...
└── models/
    ├── ggml-tiny.bin
    ├── ggml-small.bin
    └── ggml-small-q5_1.bin

/usr/bin/meet-whisperer     → symlink a /opt/meet-whisperer/run.sh
/usr/share/applications/meet-whisperer.desktop
/usr/share/icons/hicolor/
├── 16x16/apps/meet-whisperer.png
├── 32x32/apps/meet-whisperer.png
├── 64x64/apps/meet-whisperer.png
├── 128x128/apps/meet-whisperer.png
├── 256x256/apps/meet-whisperer.png
└── 512x512/apps/meet-whisperer.png
```

### Datos del usuario (no se instalan ni se borran con dpkg)

```
~/.config/meet-whisperer/
├── settings.toml           ← configuración (dispositivos, modelo, Ollama)
└── recordings.db           ← historial de grabaciones (SQLite)

~/Grabaciones/              ← transcripciones .txt (ruta configurable)
```

---

## Tarball portable (alternativa al .deb)

Si necesitas distribuir la app sin instalarla como paquete del sistema:

```bash
./dist.sh                   # compila + empaca en .tar.gz
# o
make dist
```

Resultado: `dist/meet-whisperer.tar.gz`

```bash
# Uso en otro equipo
tar -xzf meet-whisperer.tar.gz
./meet-whisperer/run.sh
```

