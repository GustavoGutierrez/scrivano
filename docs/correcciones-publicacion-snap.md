# Correcciones de publicación Snap (qué funcionó y qué no)

Registro de hallazgos reales durante la publicación de Scrivano en Snap Store.

---

## Contexto

- Snap publicado: `scrivano` (canal `latest/stable`)
- URL: https://snapcraft.io/scrivano

---

## ✅ Lo que sí funcionó

1. **Build del snap**
   - `./build-snap.sh` y `snapcraft pack --destructive-mode` funcionaron.
   - Se generó correctamente `snap/scrivano_1.1.8_amd64.snap`.

2. **Subida de revisión a store**
   - `snapcraft upload --release=stable ...` funcionó.
   - La revisión quedó publicada en `stable`.

3. **Selftest del wrapper**
   - `scrivano --selftest` finalizó con `Wrapper selftest complete`.
   - Copia de modelos bundled (`ggml-tiny.bin`, `ggml-small-q5_1.bin`) correcta.
   - Conexión con PulseAudio correcta (interfaces conectadas).

4. **Icono dentro del paquete**
   - El `.snap` contiene `meta/gui/icon.png`.
   - Es decir: el icono estaba presente en el artefacto publicado.

---

## ❌ Lo que no funcionó inicialmente

1. **Icono no visible en Snap Store**
   - En la página pública aparecía el placeholder de ícono faltante.
   - Aunque el icono sí venía dentro del `.snap`, el listing no lo reflejaba.

2. **Screenshots no visibles en Snap Store**
   - No se mostraban capturas porque no se habían subido en el dashboard de listing.
   - Incluir archivos en el repo/snap no publica automáticamente screenshots.

3. **Confusión de rutas para assets**
   - Los assets que usa `snapcraft.yaml` para empaquetar icono vienen de `snap/gui/`.
   - Crear `gui/` en raíz del repo no afecta el snap si `source` apunta a `snap/gui/`.

---

## ✅ Correcciones aplicadas

1. **Forzar publicación de metadata de store**

```bash
snapcraft upload-metadata snap/scrivano_<VERSION>_amd64.snap --force
```

Esto sincroniza explícitamente:

- summary
- description
- icon

2. **Subir screenshots manualmente en dashboard**

- Dashboard: https://dashboard.snapcraft.io/snaps/scrivano/
- Sección: `Listing`
- Screenshots usadas desde `screenshots/`:
  - `1.png`, `2.png`, `3.png`, `4.png`

3. **Alinear fuente de icono al árbol del snap**

- Icono válido en `snap/gui/icon.png`
- Verificar que `snap/snapcraft.yaml` tenga:

```yaml
icon: gui/icon.png
```

---

## Observaciones importantes

1. `snapcraft upload` **no siempre** basta para que el listing muestre icono actualizado.
   - Recomendación: ejecutar siempre `snapcraft upload-metadata ... --force` después del upload.

2. Las screenshots son un flujo separado del paquete.
   - Se gestionan desde el dashboard de listing.

3. Puede haber delay por caché del sitio web.
   - Verificar después de unos minutos y refrescar.

---

## Comandos de verificación útiles

```bash
# Estado de canales y revisión activa
snapcraft status scrivano

# Historial de revisiones
snapcraft list-revisions scrivano

# Metadata visible desde cliente
snap info scrivano

# Verificar icono dentro del .snap (local)
unsquashfs -ll snap/scrivano_<VERSION>_amd64.snap | grep -E "meta/gui|icon"
```

---

## Recomendación operativa para próximos releases

Secuencia mínima segura:

1. Build snap
2. Probar local (`--dangerous` + `--selftest`)
3. `snapcraft upload --release=stable ...`
4. `snapcraft upload-metadata ... --force`
5. Revisar listing y screenshots en dashboard
