# Fase 6 post-release: Ollama block correction

Esta fase queda **intencionalmente diferida para después del release** de `chunked-long-transcription`.

## Objetivo

Mejorar la redacción/corrección del transcript usando Ollama por bloques, sin volver al patrón lento de enviar toda la reunión completa en un único prompt.

## Alcance propuesto

- Corregir bloques de transcript de 3–8 minutos.
- Usar un modelo local rápido como `qwen2.5:3b`.
- Mantener la salida final ordenada y sin reescribir timestamps.
- Ejecutar una pasada final liviana solo para título/tags/resumen.
- Permitir desactivar esta fase si Ollama no está disponible.

## SDD pendiente

Continuar con el cambio:

- `sdd/chunked-long-transcription/tasks`
- tareas diferidas: `6.1` y `6.2`

## Criterio de entrada

No empezar esta fase hasta que el release base de chunking/transcripción quede verificado y publicado.

## Riesgos a cuidar

- No bloquear UI durante la corrección.
- No superar recursos locales razonables.
- No perder el transcript crudo si la mejora falla.
- No mezclar corrección de contenido con cambios de timestamps.
