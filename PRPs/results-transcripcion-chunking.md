# Resultados de validación — chunked-long-transcription

Fecha: 2026-05-16
Modo de evidencia: **criterio de release aceptado** (benchmark real acotado + extrapolación documentada)

## Estado

- Se aceptó como gate de release: benchmark real acotado + extrapolación documentada.
- El benchmark real 10/30/60 min queda explícitamente diferido a post-release/performance-lab.

## Evidencia de compatibilidad legacy (ruta de library/history)

Se agregó y ejecutó `tests/legacy_library_compatibility_tests.rs` para cubrir la ruta funcional usada por biblioteca/historial:

- `legacy_runtime_recording_is_visible_in_library_query_path`
  - Crea una DB con esquema legacy (sin columnas nuevas)
  - Abre con `Database::open()` (migración runtime)
  - Verifica `list_recordings()` + `get_segments_by_recording()`
- `legacy_runtime_recording_remains_usable_for_transcript_export`
  - Parte del mismo esquema legacy
  - Verifica que los segmentos legacy se exportan correctamente (`export_to_txt`)

Resultado: **2 passed, 0 failed**.

## Supuestos del surrogate

- `chunk_seconds = 25`
- `overlap_seconds = 5`
- `post_stop_workers = 1`
- `baseline_rtf = 1.0` (1 segundo de audio ≈ 1 segundo de transcripción en baseline)
- Modelo conservador usado en `src/perf_metrics.rs`:
  - Baseline post-stop = `duración_sesión * baseline_rtf`
  - Chunked post-stop ≈ `chunk_seconds + overlap_seconds` (audio remanente al detener) `* baseline_rtf / workers`

## Matriz 10/30/60 (surrogate)

| Sesión | Espera post-stop baseline (ms) | Espera post-stop chunked (ms) | Reducción |
|---|---:|---:|---:|
| 10 min | 600000 | 30000 | 95.00% |
| 30 min | 1800000 | 30000 | 98.33% |
| 60 min | 3600000 | 30000 | 99.17% |

## Benchmark real acotado (medido en esta sesión)

Artefacto generado: `target/perf-baseline/real-bounded-benchmark.json` (actualización incremental por duración para no perder evidencia si se aborta)

Modelo: `models/ggml-medium-q5_0.bin`
Señal: audio sintético seno 220Hz @16kHz (no dataset de voz real)
Comando: `cargo test --test transcription_tests integration_real_bounded_benchmark_writes_report -- --ignored --nocapture`

| Duración audio | Tiempo medido (ms) | RTF |
|---|---:|---:|
| 15 s | 10396 | 0.6931 |
| 30 s | 10224 | 0.3408 |
| 60 s | 19869 | 0.3312 |
| 180 s *(extendido opcional)* | 63473 | 0.3526 |
| 300 s *(extendido opcional)* | 105760 | 0.3525 |

## Extrapolación documentada del gate

Con `chunk_seconds=25`, `overlap_seconds=5`, `workers=1` y `baseline_rtf=1.0` (modelo conservador del surrogate):

- backlog post-stop chunked ≈ `30s` de audio pendiente.
- espera baseline 60 min = `3600s`.
- reducción estimada = `(3600 - 30) / 3600 = 99.17%`.

Interpretación:
- El benchmark real acotado confirma throughput empírico local (RTF aprox. 0.33–0.69 según ventana).
- La extrapolación conservadora mantiene margen MUY superior al objetivo de 40% para 60 minutos.
- El benchmark real completo 10/30/60 min se conserva como validación post-release, no como bloqueo de release.

## Conclusión de validación

- Con el criterio aceptado por usuario (benchmark acotado + extrapolación), **el gate de release queda satisfecho**.
- La corrida real 10/30/60 min queda marcada como **post-release/performance-lab** y no bloquea este release.
