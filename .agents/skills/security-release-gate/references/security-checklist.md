# Security checklist rationale (Scrivano)

Checklist resumido para releases/publicación basado en prácticas de GitHub, Rust y Snapcraft.

## GitHub repository security baseline

- Secret scanning y revisión de exposición accidental de credenciales.
- Dependency/security scanning y code scanning antes de release.
- Revisión previa a publicación de artefactos.

## Rust secure release baseline

- Conciencia del riesgo en dependencias y cambios de lockfile.
- Evitar filtración de secretos en código, config y artefactos.
- Verificar procedencia del artefacto publicado (build fresca del commit correcto).

## Snapcraft publishing baseline

- Preferir `strict` confinement sobre `classic` salvo excepción explícita.
- Verificar integridad/trazabilidad del paquete a subir.
- Confirmar que el binario empaquetado es el correcto para la versión objetivo.

## Repo-specific critical controls

1. Baseline de validación Rust (`fmt`, `clippy`, `test`) en verde.
2. Versiones alineadas: Cargo, Snap y About.
3. Payload snap (`snap/dist-bin/scrivano`) proveniente de `target/release/scrivano` recién compilado.
4. No publicar si hay findings críticos.
