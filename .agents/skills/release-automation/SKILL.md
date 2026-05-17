---
name: release-automation
description: Automatizar releases de Scrivano con semver, notas basadas en cambios reales, validación y publicación segura.
---

# Release Automation Skill (Scrivano)

## Trigger

Usar cuando el usuario pida: `release`, `new version`, `publish release`, `tag release`, `nueva versión`.

## Non-negotiable gates

1. **Cargar y ejecutar** `.agents/skills/security-release-gate/SKILL.md` antes de push/tag/release.
2. Validar baseline:
   ```bash
   cargo fmt --check
   cargo clippy --all-targets --all-features
   cargo test
   ```
3. No continuar si no hay commits desde el último tag.

## Workflow

### 1) Detectar versión objetivo (semver)

```bash
git fetch --tags
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
CURRENT=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)
```

Regla:
- `BREAKING CHANGE` o `type!:` -> MAJOR
- `feat:` -> MINOR
- resto -> PATCH

### 2) Construir release notes DESDE cambios reales

Fuentes obligatorias:
- `git log ${LAST_TAG}..HEAD`
- `git diff --stat ${LAST_TAG}..HEAD`
- (si aplica) PRs/commits mergeados

Formato recomendado:
- `## Summary`
- `## User-visible changes`
- `## Internal quality changes`
- `## Validation`
- `## Install / Update notes`

**Prohibido**: placeholders genéricos tipo “auto-generated changes” sin listar cambios reales.

### 3) Evitar errores de quoting en notas

No armar notas largas inline con quoting frágil. Usar archivo temporal:

```bash
cat > /tmp/scrivano-release-notes.md <<'EOF'
## Summary
...
EOF
```

### 4) Version bump + tag

Actualizar al menos:
- `Cargo.toml` (`version`)
- `snap/snapcraft.yaml` (`version`) si la release incluye snap

Luego:
```bash
git add Cargo.toml snap/snapcraft.yaml
git commit -m "release: vX.Y.Z"
git tag -a "vX.Y.Z" -m "Release vX.Y.Z"
```

### 5) Publicar GitHub release

Camino principal:
```bash
gh release create "vX.Y.Z" --title "Scrivano vX.Y.Z" --notes-file /tmp/scrivano-release-notes.md
```

Fallback seguro (si faltan flags o edición compleja):
```bash
gh api repos/:owner/:repo/releases/tags/vX.Y.Z --jq '.id'
gh api repos/:owner/:repo/releases/<id> -X PATCH -f body@/tmp/scrivano-release-notes.md
```

## Stop conditions

DETENER release si pasa cualquiera de estas:
- Security gate marca findings **critical** sin resolver.
- `fmt/clippy/test` fallan.
- Mismatch de versión entre `Cargo.toml`, tag o release target.
- Release notes no reflejan cambios reales desde `LAST_TAG`.

## Output expected to user

- Tipo de bump detectado + por qué.
- Rango incluido (`LAST_TAG..HEAD`).
- Release notes finales (resumen breve + archivo fuente).
- Estado de validaciones y security gate.
