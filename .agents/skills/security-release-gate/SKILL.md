---
name: security-release-gate
description: Gate obligatorio de seguridad para push a main, GitHub releases y publicación de snap. Bloquea publicación ante hallazgos críticos.
---

# Security Release Gate (Scrivano)

## Trigger

Cargar SIEMPRE antes de:
- push a `main`
- crear/publicar GitHub release
- publicar snap en cualquier canal

## Goal

Evitar que Scrivano publique artefactos con riesgo crítico de seguridad o integridad.

## Required checks

### 1) Secret leakage (repo + diff)

- Revisar cambios staged/pendientes y archivos trackeados por patrones de secretos (tokens, private keys, credenciales, `.env` sensibles).
- Si se detecta exposición real, **BLOCK**.

### 2) Validation baseline

Todos deben pasar:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test
```

Si alguno falla, **BLOCK**.

### 3) Dependency and code risk awareness

- Confirmar que no hay findings críticos abiertos en dependencias/code scanning (si hay integración CI/GitHub security).
- Si existe evidencia de hallazgo crítico no mitigado, **BLOCK**.

### 4) Version and artifact provenance

Verificar consistencia de versión entre:
- `Cargo.toml`
- `snap/snapcraft.yaml` (cuando aplica)
- fuente de versión en About (`src/ui/about.rs`, `env!("CARGO_PKG_VERSION")`)
- tag/release objetivo

Verificar que artefacto a publicar viene de build release fresca del commit actual.

Mismatch o artefacto sin trazabilidad, **BLOCK**.

### 5) Snap-specific security posture (si aplica)

- Revisar `confinement` en `snap/snapcraft.yaml`.
- `strict` es baseline esperado; `classic` requiere justificación y aprobación explícita.

Cambio no aprobado que debilita aislamiento, **BLOCK**.

## Severity policy

- **CRITICAL** -> bloquea publicación sí o sí.
- **IMPORTANT** -> no bloquea automáticamente, pero debe proponer remediación concreta antes de recomendar publish.
- **LOW/INFO** -> registrar, no bloquear.

## Output format

Responder siempre:

1. `Gate Decision: PASS | BLOCK`
2. `Critical Findings` (si hay)
3. `Important Findings` (si hay)
4. `Corrective Actions` (**solo para CRITICAL/IMPORTANT**)
5. `Evidence` (comandos/resultados usados)

## Stop conditions

No avanzar a push/tag/release/upload mientras `Gate Decision = BLOCK`.

## References

Ver `.agents/skills/security-release-gate/references/security-checklist.md`.
