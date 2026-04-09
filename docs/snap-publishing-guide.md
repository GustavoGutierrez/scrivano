# Publicación de Scrivano en Snap Store - Guía Completa

Este documento detalla el proceso completo de publicación del snap de Scrivano, incluyendo problemas encontrados y soluciones aplicadas.

## Estado Actual

✅ Snap construido exitosamente: `snap/scrivano_1.1.8_amd64.snap`
✅ Snap probado localmente y funcionando
✅ Modelos Whisper empaquetados dentro del snap y copiados al primer inicio
✅ Icono y metadata configurados

## Próximos Pasos de Publicación

### Paso 1: Login en Snap Store

```bash
# Solo necesario la primera vez o si expiró la sesión
snapcraft login
```

Se abrirá un navegador para autenticación con Ubuntu One.

### Paso 2: Registrar el Nombre del Snap

```bash
# Solo necesario la primera vez
snapcraft register scrivano
```

Esto reserva el nombre "scrivano" en la tienda de snaps.

### Paso 3: Subir al Canal Estable

```bash
snapcraft upload --release=stable snap/scrivano_1.1.8_amd64.snap
```

Este comando:
- Sube el snap a la tienda
- Lo publica en el canal stable
- Hace que esté disponible públicamente

### Paso 4: Agregar Screenshots en el Dashboard

1. Ve a: https://dashboard.snapcraft.io/snaps/scrivano/
2. Click en la pestaña **"Listing"**
3. Sube las 4 screenshots desde `screenshots/`:
   - `screenshots/1.png`
   - `screenshots/2.png`
   - `screenshots/3.png`
   - `screenshots/4.png`
4. Click en **"Save changes"**

### Paso 5: Agregar Metadatos Adicionales (Opcional)

En el dashboard puedes configurar:
- Descripción larga (ya configurada en snapcraft.yaml)
- Icono (ya configurado)
- Screenshots
- Banner image (opcional)
- Categorías (recomendado: Audio, Productivity)
- Licencia (ya configurado: MIT)

### Paso 6: Verificar Publicación

```bash
# Verificar que el snap está publicado
snap info scrivano

# Instalar desde la tienda
sudo snap install scrivano

# Probar la instalación desde la tienda
scrivano
```

## Problemas Encontrados y Soluciones

### Problema 1: Campos no Permitidos en snapcraft.yaml

**Error**: Extra inputs are not permitted for fields: `developer`, `license-url`, `homepage`, `source`, `issues`

**Causa**: Core22 no permite estos campos en el snapcraft.yaml. Estos metadatos deben configurarse en el dashboard.

**Solución**: Eliminamos estos campos del snapcraft.yaml:
```yaml
# REMOVIDO - configurar en dashboard
# developer: Gustavo Gutiérrez
# license-url: https://...
# homepage: https://...
# source: https://...
# issues: https://...
```

### Problema 2: Error con LXD y Multipass

**Error**: Network-related operation failed in LXD/Multipass, permission denied issues

**Causa**: LXD y Multipass tienen problemas de permisos y red en configuraciones snap-to-snap.

**Solución**: Usar modo destructivo que construye directamente en el host:
```bash
snapcraft pack --destructive-mode
```

### Problema 3: Warning de Bash-Completion

**Error**: `snap is unusable due to missing files: path "bash-completion.bash" does not exist`

**Causa**: El snapcraft.yaml hacía referencia a un archivo de bash completion que no existe.

**Solución**: Remover la referencia del snapcraft.yaml:
```yaml
apps:
  scrivano:
    command: bin/scrivano-wrapper
    # REMOVIDO: completer: bash-completion.bash
```

### Problema 4: Plug y Slot con el Mismo Nombre

**Error**: `cannot have plug and slot with the same name: "audio-playback"`

**Causa**: El snap intentaba definir tanto un plug como un slot con el mismo nombre "audio-playback".

**Solución**: Remover el slot y agregar plugs necesarios:
```yaml
apps:
  scrivano:
    plugs:
      - audio-playback
      - audio-record    # Agregado para grabación
      - network
      - home
      - removable-media
    # REMOVIDO: slots: - audio-playback
```

### Problema 5: Conflicto de Directorio en Parts

**Error**: `Failed to copy 'gui': no such file or directory`

**Causa**: La parte `gui` intentaba hacer stage de un directorio que el plugin dump no creaba.

**Solución**: Usar la directiva `organize` para estructurar los archivos correctamente:
```yaml
gui:
  plugin: dump
  source: gui/
  source-type: local
  organize:
    icon.png: gui/icon.png
    '*': gui/
  prime:
    - gui/
```

### Problema 6: Warnings de Librerías Faltantes

**Warning**: Missing dependency para múltiples bibliotecas (libFLAC, libX11, libasound, etc.)

**Causa**: El binario de Scrivano está compilado dinámicamente contra bibliotecas del sistema.

**Solución**: Agregar las bibliotecas como stage-packages en el snap:
```yaml
scrivano:
  plugin: dump
  source: bin/
  stage-packages:
    - libflac8
    - libx11-xcb1
    - libx11-6
    - libxau6
    - libxdmcp6
    - libasound2
    - libasyncns0
    - libogg0
    - libopus0
    - libpulse0
    - libsndfile1
    - libvorbis0a
    - libvorbisenc2
    - libxcb1
```

**Nota**: En `strict` confinement estas bibliotecas deben ir empaquetadas como `stage-packages` dentro del snap.

### Problema 7: Metadatos Faltantes

**Warning**: Missing metadata fields: `title`, `contact`, `website`, `source-code`, `issues`

**Solución**: Agregar campos de metadata completos:
```yaml
title: Scrivano
contact: gustavo@example.com
website: https://github.com/GustavoGutierrez/scrivano
source-code: https://github.com/GustavoGutierrez/scrivano
issues: https://github.com/GustavoGutierrez/scrivano/issues
```

## Estructura del Snap

```
snap/
├── snapcraft.yaml          # Configuración del snap
├── bin/
│   ├── scrivano             # Binario de la aplicación
│   └── scrivano-wrapper     # Script que prepara modelos locales
├── models/
│   ├── ggml-tiny.bin
│   └── ggml-small-q5_1.bin
└── gui/
    └── icon.png             # Icono de la aplicación

screenshots/                 # Screenshots para la tienda
├── 1.png
├── 2.png
├── 3.png
└── 4.png
```

## Gestión de Modelos Whisper

El snap usa un wrapper script (`snap/bin/scrivano-wrapper`) que:

1. Verifica si existen los modelos en `$SNAP_USER_DATA/models/`
2. Si faltan, copia los modelos empaquetados desde `$SNAP/models/`
3. No depende de `wget/curl` en runtime
4. Funciona offline desde el primer inicio
5. Mantiene los modelos en una ruta escribible del usuario

## Comandos Útiles

### Revisar el Snap
```bash
# Ver contenido del snap
unsquashfs -l snap/scrivano_1.1.8_amd64.snap

# Ver metadata del snap
snap info snap/scrivano_1.1.8_amd64.snap
```

### Construir para Diferentes Arquitecturas
```bash
# AMD64 (x86_64)
snapcraft pack --destructive-mode

# ARM64 (requiere cross-compilation o VM)
snapcraft pack --target-arch=arm64
```

### Actualizar la Versión
```bash
# 1. Actualizar version en Cargo.toml
# 2. Actualizar version en snap/snapcraft.yaml
# 3. Ejecutar build script
./build-snap.sh
# 4. Subir nueva versión
snapcraft upload --release=stable snap/scrivano_1.1.9_amd64.snap
```

## Script de Build Automatizado

El archivo `build-snap.sh` automatiza todo el proceso:

```bash
#!/bin/bash
# Ver build-snap.sh para el código completo
```

Uso:
```bash
./build-snap.sh
```

El script:
1. Lee la versión de Cargo.toml
2. Construye el binario release
3. Prepara el directorio snap
4. Construye el snap con `--destructive-mode`
5. Muestra el resultado y próximos pasos

## Canales de Publicación

Snap Store tiene 4 canales:

- **stable**: Producción, usuarios finales (recomendado)
- **candidate**: Release candidates, testing final
- **beta**: Versiones beta, testers
- **edge**: Últimos cambios, development

Para publicar en diferentes canales:
```bash
# Stable (producción)
snapcraft upload --release=stable snap/scrivano_1.1.8_amd64.snap

# Beta (testing)
snapcraft upload --release=beta snap/scrivano_1.1.8_amd64.snap

# Edge (development)
snapcraft upload --release=edge snap/scrivano_1.1.8_amd64.snap
```

## Verificación Post-Publicación

### Desde la Terminal
```bash
# Ver información pública
snap info scrivano

# Ver revisiones publicadas
snapcraft list-revisions scrivano

# Ver estado de canales
snapcraft status scrivano
```

### Desde el Dashboard
- URL: https://dashboard.snapcraft.io/snaps/scrivano/
- Métricas de descargas
- Revisiones publicadas
- Estado de los canales
- Reviews de usuarios

## Troubleshooting

### Snap No Inicia

```bash
# Ver logs del snap
journalctl -u snap.scrivano

# Verificar confinement
snap info scrivano | grep confinement

# Verificar plugs conectados
snap connections scrivano
```

### Modelos No Se Descargan

```bash
# Verificar directorio de modelos
ls -la $HOME/snap/scrivano/current/models/

# Descarga manual
cd $HOME/snap/scrivano/current/models/
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small-q5_1.bin
```

### Error de Audio/PulseAudio

El snap requiere acceso al servidor PulseAudio:
```bash
# Verificar plugs conectados
snap connections scrivano

# Conectar manualmente si es necesario
snap connect scrivano:audio-playback :audio-playback
snap connect scrivano:audio-record :audio-record
snap connect scrivano:pulseaudio :pulseaudio
```

## Recursos Adicionales

- [Documentación de Snapcraft](https://snapcraft.io/docs)
- [Referencia de snapcraft.yaml](https://snapcraft.io/docs/snapcraft-yaml-reference)
- [Dashboard de Snap Store](https://dashboard.snapcraft.io/)
- [Debugging Snaps](https://snapcraft.io/docs/debugging-snaps)
- [Interfaces y Strict Confinement](https://snapcraft.io/docs/supported-interfaces)

## Historial de Versiones

| Versión | Fecha | Notas |
|---------|------|-------|
| 1.1.8 | 2026-04-08 | Primera publicación en Snap Store |
| | | - Modelos incluidos y copiados automáticamente |
| | | - Soporte para audio playback y recording |
| | | - Integración con Ollama |
| | | - Export a múltiples formatos |

## Checklist de Publicación

- [x] Versión actualizada en Cargo.toml
- [x] Versión actualizada en snap/snapcraft.yaml
- [x] Icono en snap/gui/icon.png
- [x] Screenshots en screenshots/
- [x] Wrapper script para copia de modelos empaquetados
- [x] Metadata completa (title, contact, website, issues)
- [x] Stage-packages para bibliotecas necesarias
- [x] Plugs necesarios configurados
- [x] Snap probado localmente
- [ ] Snap publicado en Snap Store
- [ ] Screenshots subidos al dashboard
- [ ] Publicación verificada con `snap info`

## Contacto y Soporte

- **Issues**: https://github.com/GustavoGutierrez/scrivano/issues
- **Repositorio**: https://github.com/GustavoGutierrez/scrivano
- **Licencia**: MIT
