# Publicación de nuevas versiones de Scrivano en Snap Store

Guía operativa (end-to-end) para sacar una nueva versión: desde compilación local hasta publicación en la tienda.

---

## 1) Prerrequisitos

- Tener `snapcraft` instalado:

```bash
sudo snap install snapcraft --classic
```

- Sesión iniciada en Snapcraft:

```bash
snapcraft login
```

- Nombre del snap registrado (solo la primera vez):

```bash
snapcraft register scrivano
```

- Modelos requeridos presentes en el repo:
  - `models/ggml-tiny.bin`
  - `models/ggml-small-q5_1.bin`

- Icono del snap presente en:
  - `snap/gui/icon.png`

---

## 2) Actualizar versión

Antes de compilar, actualizar la versión en:

1. `Cargo.toml`
2. `snap/snapcraft.yaml`

Ejemplo:

- `Cargo.toml` → `version = "1.1.9"`
- `snap/snapcraft.yaml` → `version: '1.1.9'`

---

## 3) Compilar binario y construir el .snap

### Opción recomendada (script del proyecto)

Desde la raíz del repo:

```bash
./build-snap.sh
```

Este script:

1. Compila en release (`cargo build --release --features "audio-playback tray-icon"`)
2. Prepara `snap/dist-bin`, `snap/models` y wrapper
3. Construye el paquete con:

```bash
snapcraft pack --destructive-mode
```

Salida esperada:

- `snap/scrivano_<VERSION>_amd64.snap`

### Opción manual (si necesitas debug)

```bash
cargo build --release --features "audio-playback tray-icon"
mkdir -p snap/dist-bin snap/models snap/wrapper
cp target/release/scrivano snap/dist-bin/
cp models/ggml-tiny.bin snap/models/
cp models/ggml-small-q5_1.bin snap/models/
cd snap
snapcraft clean
snapcraft pack --destructive-mode
```

---

## 4) Prueba local antes de publicar

Instalar localmente:

```bash
sudo snap install --dangerous snap/scrivano_<VERSION>_amd64.snap
```

Selftest rápido:

```bash
scrivano --selftest
```

Esperado:

- `Wrapper selftest complete`
- Conexión a PulseAudio ok
- Modelos copiados correctamente al primer arranque

> Nota: Un warning de mount namespace (portal `/run/user/.../doc`) puede aparecer y no bloquear el funcionamiento.

---

## 5) Subir a Snap Store

Publicar en `stable`:

```bash
snapcraft upload --release=stable snap/scrivano_<VERSION>_amd64.snap
```

Canales alternativos:

```bash
snapcraft upload --release=edge snap/scrivano_<VERSION>_amd64.snap
snapcraft upload --release=beta snap/scrivano_<VERSION>_amd64.snap
snapcraft upload --release=candidate snap/scrivano_<VERSION>_amd64.snap
```

---

## 6) Publicar metadata (IMPORTANTE para icono)

Después del upload, forzar actualización de metadata de store (summary/description/icon):

```bash
snapcraft upload-metadata snap/scrivano_<VERSION>_amd64.snap --force
```

Esto evita que la página de Snap Store quede con ícono placeholder aunque el snap ya esté publicado.

---

## 7) Subir screenshots del listing

Las screenshots **no** se publican automáticamente desde el `.snap`.

Subir manualmente en:

- https://dashboard.snapcraft.io/snaps/scrivano/
- Tab: **Listing**

Imágenes recomendadas del proyecto:

- `screenshots/1.png`
- `screenshots/2.png`
- `screenshots/3.png`
- `screenshots/4.png`

---

## 8) Verificación post-publicación

```bash
snapcraft status scrivano
snapcraft list-revisions scrivano
snap info scrivano
```

Verificar también en web:

- https://snapcraft.io/scrivano

> La caché del sitio puede tardar unos minutos en reflejar cambios de icono/listing.

---

## 9) Checklist final

- [ ] Versión actualizada en `Cargo.toml` y `snap/snapcraft.yaml`
- [ ] Snap construido: `snap/scrivano_<VERSION>_amd64.snap`
- [ ] Selftest local OK
- [ ] Upload a canal correcto (`stable`/`candidate`/etc)
- [ ] `upload-metadata --force` ejecutado
- [ ] Screenshots subidas en dashboard
- [ ] Verificado en `snap info scrivano` y `https://snapcraft.io/scrivano`
