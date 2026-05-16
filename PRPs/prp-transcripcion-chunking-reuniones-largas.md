# PRP: Transcripción rápida y eficiente para reuniones largas

Scrivano hoy graba todo el audio en memoria y recién al detener clona el buffer completo para transcribirlo. Para reuniones largas, eso escala mal: más memoria, más espera al final y una única inferencia Whisper demasiado grande. La solución propuesta es pasar a una arquitectura de sesiones con chunks persistidos en disco, transcripción por chunk ordenada y mejora LLM incremental.

## 1. Project Overview

**Proyecto:** Pipeline de grabación por chunks + transcripción incremental para reuniones largas.
**Patrón:** B — cambio arquitectónico de performance, con impacto en audio, transcripción, DB y UI.
**Usuario objetivo:** personas que graban reuniones de 30 min a 3 h localmente, sin nube.
**Prioridad:** velocidad y consumo local controlado.
**Plataforma principal:** Ubuntu/Linux actual.
**Compatibilidad futura:** Linux genérico primero; Windows requiere abstracción de backend de audio.

## 2. Problem Statement

Usuarios de Scrivano enfrentan lentitud y consumo creciente cuando graban reuniones largas. Actualmente:

- `src/audio.rs` agrega todos los samples a `Arc<Mutex<Vec<f32>>>`.
- `src/ui.rs::stop_and_transcribe` clona todo el buffer (`buffer.lock().unwrap().clone()`).
- Whisper transcribe un único bloque de audio completo.
- El WAV se guarda recién al final.
- Ollama mejora todo el texto de una vez, con prompts largos y mayor latencia.

Esto causa espera larga post-grabación, presión de memoria y riesgo de perder todo si la app falla antes de guardar.

## 3. Success Criteria

### Métrica primaria

- Reducir el tiempo desde “detener grabación” hasta “transcripción completa visible” al menos **40%** en una reunión de 60 min frente al pipeline actual.

### Métricas secundarias

- Mantener uso de RAM adicional de audio por debajo de **256 MB** durante grabaciones de hasta 3 h.
- Mantener CPU/GPU por debajo de **60% promedio** durante grabación + transcripción en la máquina objetivo.
- No perder audio si la app se cierra después de cerrar al menos un chunk.
- Mantener orden cronológico exacto de segmentos.
- Tasa de duplicados por overlap menor a **1 segmento duplicado cada 30 min**.

### Baseline obligatoria antes de tocar código

Crear un benchmark inicial del pipeline actual y guardar resultados en `target/perf-baseline/*.json`:

| Caso | Modelo Whisper | Duración | Medir |
|------|----------------|----------|-------|
| B1 | actual `ggml-medium-q5_0.bin` si existe | 10 min sintético/real | RTF, RAM pico, CPU promedio |
| B2 | actual | 30 min | RTF, RAM pico, tiempo post-stop |
| B3 | actual | 60 min | RTF, RAM pico, tiempo post-stop |

**RTF:** `processing_seconds / audio_seconds`. Menor es mejor. `0.5` significa procesa 1 h de audio en 30 min.

## 4. User Stories

1. Cuando grabo una reunión larga, quiero que Scrivano guarde el audio progresivamente, para no perder todo si falla al final.
2. Cuando detengo la grabación, quiero que la transcripción aparezca más rápido, porque parte del trabajo ya está segmentado.
3. Cuando transcribo chunks con overlap, quiero un texto final sin duplicados, para que la reunión sea legible.
4. Cuando uso Ollama, quiero corrección rápida y local, para no consumir demasiada VRAM/RAM.
5. Cuando uso Linux o Windows, quiero que el modelo de datos sea portable aunque el backend de captura cambie.

## 5. Functional Requirements

### P0 — MVP obligatorio

- **FR-001:** Crear una sesión de grabación con ID hash estable: `recordings/<session_hash>/`.
- **FR-002:** Guardar chunks de audio mientras se graba en `recordings/<session_hash>/chunks/chunk_000001.wav`.
- **FR-003:** Crear `manifest.jsonl` con `session_hash`, `chunk_index`, `start_sec`, `end_sec`, `overlap_start_sec`, `sample_rate`, `sha256`, `path`, `status`.
- **FR-004:** Mantener solo un buffer pequeño en RAM: chunk activo + overlap + waveform UI.
- **FR-005:** Transcribir chunks cerrados en orden lógico usando timestamps absolutos.
- **FR-006:** Unir segmentos con deduplicación por overlap.
- **FR-007:** Guardar transcript final en `recordings/<session_hash>/transcript.txt` y segmentos en SQLite.
- **FR-008:** Mostrar progreso real por chunks: `chunk actual / total`, no progreso simulado.
- **FR-009:** Mantener compatibilidad con historial existente `.txt/.wav`.
- **FR-010:** Agregar tests unitarios para chunking, manifest y unión de segmentos.

### P1 — Performance real

- **FR-011:** Permitir transcripción background de chunks cerrados mientras continúa la grabación, con límite de recursos.
- **FR-012:** Agregar modo de performance: `Calidad`, `Balanceado`, `Rápido`.
- **FR-013:** Guardar métricas por sesión en `recordings/<session_hash>/metrics.json`.
- **FR-014:** Permitir reintentar solo chunks fallidos.
- **FR-015:** Mejorar transcript con Ollama por bloques, no con todo el texto de una vez.

### P2 — Futuro

- **FR-016:** Backend de audio abstracto para Windows.
- **FR-017:** VAD opcional para evitar transcribir silencios largos.
- **FR-018:** Diarización local opcional.

## 6. Non-Functional Requirements

### Performance

- Chunk recomendado inicial: **25 s de audio nuevo + 5 s de overlap** para transcripción.
- Guardado físico: chunks WAV sin duplicar overlap o con overlap marcado explícitamente en manifest. Para simplicidad MVP, se acepta chunk transcribible con overlap si la deduplicación está testeada.
- Threads Whisper: no usar `20` hilos por defecto. Calcular `min(8, floor(physical_cores * 0.6))`. En esta máquina i7-12700H con 14 núcleos físicos, `8` es razonable.
- No bloquear UI durante flush, transcripción ni Ollama.

### Recursos

- Máquina analizada: Intel i7-12700H, 20 threads, 30 GiB RAM, RTX 4070 Laptop 8 GiB VRAM.
- Objetivo: no superar 60% promedio de CPU/GPU. La app debe permitir bajar concurrencia si el sistema está cargado.
- Usar cola de trabajos con máximo inicial: `1` worker Whisper mientras se graba, `2` workers solo después de detener si el modelo/contexto lo permite sin duplicar memoria de modelo de forma excesiva.

### Fiabilidad

- Cada chunk cerrado debe tener hash SHA-256.
- `manifest.jsonl` se escribe append-only para recuperación.
- Si falla un chunk, el transcript final debe marcar hueco explícito y permitir retry.

### Privacidad

- Todo local. Sin APIs externas.
- No enviar audio ni texto a servicios remotos.

## 7. Technical Constraints

### Estado actual del código

- `src/audio.rs`: captura PulseAudio, resamplea a 16 kHz y acumula en `Vec<f32>`.
- `src/transcription.rs`: `transcribe_with_segments` usa `whisper-rs`, `Greedy`, `set_no_context(true)`, `set_single_segment(true)`, `set_audio_ctx(0)`, `set_n_threads(8)`.
- `src/ui.rs`: `stop_and_transcribe` clona audio completo, transcribe, mejora con Ollama, guarda `.txt` y `.wav`.
- `src/database.rs`: ya soporta `recordings`, `transcript_segments`, `highlights`, `summaries`.
- `src/audio_devices.rs`: busca modelos Whisper en `$SNAP_USER_DATA/models`, `$SNAP/models`, `models/` y junto al ejecutable.

### Diseño propuesto

Crear módulos nuevos:

- `src/recording_session.rs` — IDs, paths, manifest, estado recoverable.
- `src/audio_chunker.rs` — rotación de chunks y overlap.
- `src/chunk_transcription.rs` — cola de transcripción y join ordenado.
- `src/perf_metrics.rs` — RTF, tiempos, memoria estimada, uso de chunks.

Evitar meter esta lógica en `ui.rs`. UI solo orquesta estado y muestra progreso. Si metemos más lógica ahí, estamos construyendo un edificio cargando vigas en la ventana: funciona un rato, después se cae.

### Estrategia de chunks

Recomendación inicial:

```text
chunk_new_audio = 25s
overlap = 5s
sample_rate = 16_000
samples_new = 400_000 f32
samples_overlap = 80_000 f32
```

Motivo:

- Whisper trabaja naturalmente con ventanas de hasta ~30 s.
- El overlap evita cortar palabras en límites.
- 25+5 mantiene contexto sin inflar demasiado el trabajo.
- Referencias públicas de `whisper.cpp` describen modo streaming con sliding window y overlap; ejemplos prácticos usan 5 s con 1 s overlap para baja latencia, y rotación de ~25 s con overlap para flujos largos.

### Deduplicación de overlap

Regla MVP:

1. Cada segmento se convierte a tiempo absoluto: `absolute_start = chunk.logical_start + segment.start_sec - overlap_before`.
2. Si un segmento nuevo cae dentro de la zona ya cubierta por el chunk anterior:
   - descartar si texto normalizado similar y timestamp solapado;
   - conservar si extiende el final anterior con contenido nuevo.
3. Ordenar por `absolute_start` antes de persistir.

Agregar función pura testeable:

```rust
fn merge_transcript_segments(chunks: Vec<ChunkTranscript>) -> Vec<TranscriptSegment>
```

## 8. Data Requirements

### Estructura en disco

```text
recordings/
  2026-05-16_11-30_<hash>/
    manifest.jsonl
    metrics.json
    transcript.raw.txt
    transcript.final.txt
    chunks/
      chunk_000001.wav
      chunk_000002.wav
    chunk_transcripts/
      chunk_000001.json
      chunk_000002.json
```

### Hash de sesión

Usar SHA-256 de:

```text
created_at + selected_input_id + selected_output_id + random_nonce
```

Nombre visible corto: primeros 12 caracteres hex. El hash completo queda en manifest.

### SQLite

Agregar campos opcionales a `recordings`:

- `session_hash TEXT`
- `session_dir TEXT`
- `chunk_count INTEGER`
- `pipeline_version TEXT`

Crear tabla opcional `recording_chunks`:

- `id`, `recording_id`, `chunk_index`, `path`, `start_sec`, `end_sec`, `sha256`, `status`, `rtf`, `error`.

## 9. UI/UX Requirements

- Durante grabación: mostrar chunk actual, duración persistida y estado “guardando en disco”.
- Durante transcripción: mostrar `Transcribiendo chunk 3/18` y RTF parcial.
- Si hay error en un chunk: mostrar “Reintentar chunks fallidos”.
- En configuración: selector de perfil:
  - **Rápido:** Whisper `base`/`base-q5`, Ollama pequeño.
  - **Balanceado:** Whisper `small-q5`, Ollama `qwen2.5:3b`.
  - **Calidad:** Whisper `medium-q5`, Ollama `llama3.2:3b` o `qwen2.5:7b` si hay recursos.

## 10. Modelo recomendado

### Whisper

El default actual tiende a `ggml-medium-q5_0.bin`. Para reuniones largas, eso prioriza calidad pero penaliza latencia.

Recomendación:

1. **Default balanceado:** `ggml-small-q5_0.bin` o equivalente multilingual.
   - Mejor equilibrio para español.
   - Menos pesado que medium.
   - Calidad claramente superior a tiny/base para reuniones reales.
2. **Modo rápido:** `ggml-base-q5_0.bin`.
   - Mucho más veloz.
   - Aceptable si luego Ollama corrige redacción.
   - Peor con ruido, acentos o términos técnicos.
3. **Modo calidad:** mantener `ggml-medium-q5_0.bin`.
   - Solo cuando el usuario acepta mayor espera.

No recomiendo `tiny` como default para reuniones importantes. Es rápido, sí, pero la deuda la pagás después corrigiendo errores: eso NO es optimización, es mover el problema.

### Ollama

Objetivo: corrección/redacción local veloz, bajo consumo y buen español.

Recomendación primaria:

```bash
ollama pull qwen2.5:3b
```

Por qué:

- Buen soporte multilingüe.
- Mejor para español que muchos modelos chicos orientados a inglés.
- Tamaño razonable para 8 GiB VRAM / CPU moderno.
- Suficiente para limpiar transcripciones, puntuación, muletillas y formato.

Fallback ultra-rápido:

```bash
ollama pull gemma2:2b
```

Alternativa general:

```bash
ollama pull llama3.2:3b
```

Configuración sugerida para corrección por chunks:

```json
{
  "temperature": 0.1,
  "top_p": 0.8,
  "num_predict": 1024,
  "num_ctx": 4096,
  "num_thread": 6
}
```

Regla: Ollama no debe recibir una reunión completa de una vez. Corregir bloques de 3–8 min y hacer una pasada final liviana sobre títulos/resumen.

## 11. Benchmark & Test Plan

### Tests unitarios obligatorios

- `audio_chunker_rotates_at_expected_sample_count`
- `audio_chunker_keeps_overlap_samples`
- `manifest_appends_chunks_in_order`
- `session_hash_is_unique_and_path_safe`
- `merge_segments_keeps_chronological_order`
- `merge_segments_removes_overlap_duplicates`
- `failed_chunk_can_be_retried_without_reprocessing_all`
- `legacy_recordings_still_load`

### Tests de integración

- Generar audio sintético de 90 s con tonos/silencios y validar que se crean 4 chunks con 25 s + overlap.
- Simular transcripciones por chunk y validar transcript final ordenado.
- Crear una sesión incompleta y validar recuperación desde manifest.

### Benchmarks antes/después

Agregar `tests/perf_transcription.rs` o `benches/transcription_pipeline.rs` con tests `#[ignore]` si requieren modelo local.

Medir:

- `audio_duration_sec`
- `chunk_count`
- `whisper_model`
- `total_processing_sec`
- `post_stop_wait_sec`
- `rtf_total`
- `peak_audio_buffer_samples`
- `peak_audio_buffer_mb`
- `ollama_model`
- `ollama_processing_sec`
- `cpu_avg_percent` si está disponible
- `gpu_vram_peak_mb` si `nvidia-smi` está disponible

Comandos sugeridos:

```bash
cargo test audio_chunker -- --nocapture
cargo test chunk_transcription -- --nocapture
cargo test perf_transcription_10min -- --ignored --nocapture
```

No usar `cargo build` como validación final si el flujo del agente tiene restricción de no construir. Para este proyecto, validar con tests focalizados y `cargo fmt --check` cuando se implemente.

## 12. Linux/Windows Compatibility

### Ubuntu/Linux

Implementación directa sobre lo actual:

- Mantener PulseAudio/PipeWire vía `libpulse-binding`.
- Persistir chunks WAV con `hound`.
- Mantener `pactl` para enumeración.
- Compatible con Snap si se respeta `$SNAP_USER_DATA/models` y carpeta writable.

### Windows

No es “solo compilar y listo”. Hoy el proyecto depende fuerte de PulseAudio y `pactl`, que no existen nativamente en Windows.

Para Windows se requiere:

- Trait `AudioCaptureBackend`.
- Backend Linux: PulseAudio/PipeWire actual.
- Backend Windows: WASAPI loopback para sistema + micrófono.
- Reemplazar paths `$HOME` por `dirs::data_dir/config_dir` multiplataforma.
- Revisar feature flags de tray/audio playback.

La arquitectura de chunks sí es portable. Lo no portable es la captura.

## 13. Implementation Plan for Agent

### Fase 0 — Baseline

1. Crear generador de audio sintético 16 kHz mono.
2. Medir pipeline actual con 10/30/60 min.
3. Guardar JSON en `target/perf-baseline/`.

### Fase 1 — Core chunking sin UI compleja

1. Crear `recording_session.rs`.
2. Crear `audio_chunker.rs` con tests puros.
3. Cambiar recorder para enviar samples a `AudioChunkWriter` en vez de acumular todo.
4. Mantener `waveform_buffer` separado para UI.

### Fase 2 — Transcripción por chunks

1. Crear `chunk_transcription.rs`.
2. Extraer función de transcripción reusable que acepte `WhisperParamsProfile`.
3. Transcribir chunks en orden.
4. Implementar deduplicación por overlap.
5. Persistir segmentos con timestamps absolutos.

### Fase 3 — Ollama eficiente

1. Dividir mejora en bloques de transcript.
2. Usar `qwen2.5:3b` como sugerencia default si está instalado.
3. Reducir `num_predict` para corrección de chunks.
4. Agregar pasada final para título/tags/resumen, no para reescribir toda la reunión.

### Fase 4 — UI y recuperación

1. Mostrar estado de chunks.
2. Permitir reintento.
3. Detectar sesiones incompletas al iniciar.
4. Mantener historial legacy.

### Fase 5 — Validación comparativa

1. Repetir benchmarks 10/30/60 min.
2. Comparar contra baseline.
3. Documentar mejora en `PRPs/results-transcripcion-chunking.md`.

## 14. Risks & Assumptions

### Riesgos

- **Duplicados por overlap:** mitigar con tests de merge y normalización de texto.
- **Más I/O en disco:** usar chunks de 25 s evita archivos diminutos.
- **Múltiples contextos Whisper consumen mucha memoria:** empezar con 1 worker y subir solo post-stop.
- **Ollama lento con textos largos:** corregir por bloques y limitar tokens.
- **Windows requiere otro backend:** no mezclar con MVP Linux.

### Supuestos

- El usuario acepta una pequeña complejidad en disco a cambio de robustez.
- `small-q5` conserva calidad suficiente para español de reuniones.
- La RTX 4070 puede manejar Ollama 3B y Whisper sin superar límites si la concurrencia se controla.

## 15. Out of Scope

- Diarización en MVP.
- Transcripción cloud.
- Traducción automática.
- Soporte Windows completo en la primera iteración.
- Modelos Whisper `large` como default.

## 16. Open Questions

1. ¿El usuario prioriza español únicamente o también reuniones bilingües ES/EN?
2. ¿Se quiere transcripción visible durante la grabación o alcanza con acelerar el post-stop?
3. ¿El paquete final va a incluir modelos o solo detectar modelos descargados por el usuario?
4. ¿Se acepta guardar chunks con overlap duplicado en disco para simplificar MVP?

## 17. References

- `whisper-rs` docs vía Context7: `FullParams`, `set_n_threads`, `set_no_context`, `set_single_segment`, timestamps y segmentos.
- Brave Search: whisper.cpp streaming apps describen sliding window con `step_ms` y overlap para preservar contexto.
- Brave Search: ejemplo local de meeting transcription usa ventana de 5 s con 1 s overlap y procesamiento background.
- Brave Search: guías de Whisper recomiendan `tiny/base` para CPU/velocidad, `small` como balance, `medium` para más calidad.
- Brave Search/Ollama library: modelos pequeños recomendados para bajo recurso incluyen `qwen2.5` 3B/7B, `gemma2:2b`, `llama3.2:3b`, `phi` mini.
