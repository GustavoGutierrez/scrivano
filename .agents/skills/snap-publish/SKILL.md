---
name: snap-publish
description: Publicar Scrivano en Snap Store verificando build fresca, versión embebida correcta y seguridad antes de subir.
---

# Snap Publish Skill (Scrivano)

## Trigger

Usar cuando el usuario pida: `publish snap`, `snap release`, `subir snap`, `publicar en store`.

## Mandatory preconditions

1. Ejecutar `.agents/skills/security-release-gate/SKILL.md` antes de upload.
2. Estar en branch/commit correcto de release.
3. Tener `snapcraft login` vigente.

## Authoritative payload path

El binario que viaja al snap se toma de:

- `snap/dist-bin/scrivano`  ← **path autoritativo para empaquetado**

## Workflow

### 1) Build fresca del binario release

```bash
cargo build --release
```

No usar binarios viejos ni descargados sin trazabilidad.

### 2) Copiar payload correcto al snap

```bash
cp target/release/scrivano snap/dist-bin/scrivano
chmod +x snap/dist-bin/scrivano
```

### 3) Verificar versión real del artefacto

Comprobar consistencia entre:
- `Cargo.toml` version
- `snap/snapcraft.yaml` version
- Binario compilado (`target/release/scrivano --version`, si está soportado)
- Fuente de versión de About (`env!("CARGO_PKG_VERSION")` en `src/ui/about.rs`)

Si no hay forma confiable de extraer versión runtime del binario, **detener** y corregir antes de publicar.

### 4) Empaquetar snap

```bash
cd snap
snapcraft clean
snapcraft pack
```

### 5) Verificación previa a upload

- Confirmar nombre/versión esperada del `.snap` generado.
- Verificar que no se alteró confinement esperado (`strict`) salvo decisión explícita.
- Revisar que el binario dentro del payload corresponde al build fresco del paso 1.

### 6) Upload por canales

Publicar en todos los canales acordados explícitamente (ejemplo: `candidate` y luego `stable`):

```bash
snapcraft upload --release=candidate <archivo.snap>
snapcraft upload --release=stable <archivo.snap>
```

## Stop conditions

DETENER publicación si:
- Security gate reporta findings **critical**.
- Falla baseline (`fmt/clippy/test`).
- Mismatch de versión (Cargo/Snap/binario/About).
- `confinement` cambió a `classic` sin aprobación explícita.
- Binario de `snap/dist-bin/scrivano` no viene de build release fresca.

## Notes

- `strict` confinement es más seguro que `classic`; no relajar sin necesidad real y revisión.
- Mantener trazabilidad del artefacto empaquetado evita publicar binarios incorrectos.
