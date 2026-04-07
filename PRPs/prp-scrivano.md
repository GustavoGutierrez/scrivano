# PRP: Scrivano — Grabación Inteligente de Reuniones en PC

## 1. Project Overview

**Proyecto:** Scrivano  
**Tipo de patrón:** Pattern C — AI-Native Desktop System  
**Timeline estimado:** 8–12 semanas para MVP funcional; 12–16 semanas para estabilización multiplataforma y hardening técnico.  
**Usuarios objetivo:** profesionales que asisten a reuniones online, consultores, desarrolladores, investigadores, equipos remotos y power users que necesitan grabación local, transcripción y resúmenes accionables sin depender de la nube.  
**Contexto de negocio / producto:** Scrivano es una aplicación de escritorio nativa en Rust con `eframe/egui` para PC que graba audio del sistema y del micrófono durante reuniones, genera transcripciones en español e inglés y produce resúmenes accionables usando modelos locales. La decisión de arquitectura para v1 es usar **Whisper embebido en la propia aplicación** mediante integración directa en Rust sobre `whisper.cpp` vía bindings como `whisper-rs` para transcripción, y **Ollama externo como runtime oficial para resúmenes/LLM**, dejando Ollama como ruta alternativa opcional para STT en escenarios avanzados o de contingencia.

Scrivano debe priorizar Linux (especialmente Ubuntu y derivados) y macOS arm64, con soporte secundario para Windows 11, y almacenar toda su información en SQLite3 sin sincronización a la nube. La solución debe funcionar en hardware limitado, incluyendo equipos con 4–8 GB de VRAM o incluso CPU moderna cuando no haya GPU disponible, por lo que la arquitectura debe minimizar dependencias pesadas y mantener un pipeline de procesamiento tolerante a sesiones largas.

## 2. Problem Statement

Profesionales que asisten a reuniones online enfrentan una fricción constante entre participar activamente y documentar decisiones, tareas y puntos clave. Esto produce pérdida de información, dificultad para reconstruir contexto y baja trazabilidad posterior, especialmente cuando la reunión ocurre en herramientas como Zoom, Teams o Meet y la grabación depende de permisos del anfitrión o de servicios cloud con limitaciones.

El problema se agrava porque muchas soluciones de transcripción y resumen dependen de la nube, lo que introduce riesgos de privacidad, dependencia de Internet, costos recurrentes y restricciones sobre dónde se almacenan los datos. Incluso las herramientas locales existentes suelen estar poco integradas con el flujo real de reuniones: no priorizan captura de audio del sistema, no ofrecen hotkeys globales ni highlights con timestamps, y no están optimizadas para ejecutarse en equipos con recursos limitados ni para separar correctamente la respuesta final de modelos “thinking”.

Se sabe que este problema es real porque el PRD base define explícitamente la necesidad de una solución completamente local, en español e inglés, con Whisper en Rust, resúmenes vía Ollama, soporte para modelos thinking, streaming adaptativo, almacenamiento SQLite3 y experiencia centrada en reuniones reales en desktop.

## 3. Success Criteria

### North Star

- Reducir el tiempo necesario para revisar y convertir una reunión grabada en información accionable de 30–60 minutos manuales a menos de 5 minutos mediante transcripción navegable + resumen estructurado local.

### Métricas primarias

- 90% de las grabaciones válidas finalizan con archivo de audio persistido localmente sin corrupción en Linux y macOS arm64.
- 85% de las sesiones procesadas generan transcripción completa en el idioma configurado (es/en) sin intervención manual cuando Whisper está correctamente instalado y configurado.
- 80% de los resúmenes solicitados producen una salida utilizable en alguna de las plantillas soportadas: ejecutivo, tareas o decisiones.
- Tiempo de inicio de grabación menor a 1 segundo desde hotkey o click en hardware objetivo para no interrumpir el flujo de reunión.

### Métricas secundarias

- Tiempo de respuesta percibido para comenzar a renderizar un resumen en modo streaming menor a 2 segundos cuando el modelo y el host de Ollama lo permitan.
- 100% de los modelos thinking configurados muestran solo la respuesta final en UI, sin exponer razonamiento interno al usuario final.
- 95% de las exportaciones a TXT, Markdown, JSON, SRT y WebVTT terminan exitosamente sobre grabaciones válidas.
- Sesiones de 2 horas continuas completadas sin pérdida de audio ni crash en plataformas prioritarias.

### Umbrales mínimos de éxito del MVP

- Grabación local confiable de audio del sistema + micrófono.  
- Transcripción local ES/EN operativa con Whisper integrado en Rust.  
- Resumen ejecutivo vía Ollama.  
- Soporte básico de highlights.  
- Persistencia local con SQLite3.  
- Exportación funcional a TXT/Markdown/SRT.

## 4. User Stories (Jobs-to-be-Done)

1. Cuando estoy en una reunión importante, quiero iniciar la grabación con un atajo global, para no perder contexto ni salir de la app de videollamada.
2. Cuando termina la reunión, quiero obtener la transcripción local en español o inglés, para poder revisar lo dicho sin reproducir todo el audio.
3. Cuando detecto un momento importante durante la llamada, quiero marcar un highlight, para revisarlo rápidamente más tarde.
4. Cuando necesito compartir resultados con mi equipo, quiero exportar transcripción y resumen en formatos abiertos, para integrarlos con mis herramientas existentes.
5. Cuando uso modelos de resumen reasoning/thinking, quiero que la app me muestre solo la respuesta útil final, para evitar ruido y no exponer cadenas de pensamiento del modelo.
6. Cuando el modelo soporta streaming, quiero ver el resumen aparecer progresivamente si eso mejora la latencia percibida, para empezar a revisar antes de que termine toda la generación.
7. Cuando no tengo Internet, quiero que la app siga grabando y transcribiendo localmente, para mantener privacidad y confiabilidad operacional.
8. Cuando soy un usuario técnico, quiero poder configurar host y puerto de Ollama — local o remoto — para adaptar la app a mi infraestructura.
9. Cuando Whisper local falla o no está disponible, quiero tener una ruta alternativa configurable, para no bloquear completamente el procesamiento de audio.

## 5. Functional Requirements

### P0 — Core / MVP

**FR-001. Captura de audio**  
El sistema debe capturar audio del sistema (loopback) y opcionalmente audio del micrófono, generando al menos un flujo mezclado y, si es viable por plataforma, pistas separadas.

**FR-002. Inicio/parada de grabación**  
La aplicación debe permitir iniciar y detener la grabación mediante botón en UI y atajo global configurable.

**FR-003. Estado visible de grabación**  
La UI debe mostrar en tiempo real estado de grabación, tiempo transcurrido y nivel de audio, en una vista compacta utilizable mientras ocurre la reunión.

**FR-004. Highlights**  
El usuario debe poder insertar marcadores durante la grabación por hotkey o botón, registrando timestamp y etiqueta opcional.

**FR-005. Transcripción primaria local en Rust**  
Tras finalizar la grabación, el audio debe procesarse con Whisper integrado directamente en Rust mediante bindings sobre `whisper.cpp`, como `whisper-rs` o equivalente, produciendo texto estructurado en español o inglés.

**FR-006. Ruta alternativa STT vía Ollama**  
El sistema debe poder, de forma opcional, usar Whisper u otros modelos STT disponibles a través de Ollama como vía alternativa, manteniendo la integración directa en Rust como ruta por defecto.

**FR-007. Segmentación temporal**  
La transcripción debe almacenarse por segmentos con `start/end`, y opcionalmente con etiquetas de hablante si se implementa diarización local.

**FR-008. Resúmenes estructurados**  
El usuario debe poder solicitar plantillas de resumen al menos en tres formatos: resumen ejecutivo, tareas y decisiones. La generación se realizará vía Ollama usando un modelo local ligero de redacción.

**FR-009. Modelos thinking**  
La integración de resúmenes debe soportar modelos estándar y modelos thinking/reasoning, detectando si la respuesta contiene campos `thinking`/`reasoning` o tags `<think>...</think>`, y entregando al dominio solo el contenido final útil.

**FR-010. Streaming adaptativo**  
La integración con Ollama debe soportar streaming y non-streaming, con política configurable (`auto`, `stream`, `non_stream`) y selección automática cuando eso mejore latencia o UX.

**FR-011. Historial de grabaciones**  
La aplicación debe ofrecer una vista de historial con filtros por fecha, duración, título y etiquetas.

**FR-012. Exportación**  
El sistema debe exportar audio, transcripción y resúmenes en formatos abiertos, incluyendo TXT, Markdown, JSON, SRT y WebVTT; el soporte a audio puede incluir WAV, FLAC y MP3 según disponibilidad del pipeline local.

**FR-013. Persistencia local**  
Todos los metadatos, transcripciones, resúmenes, highlights y configuración deben almacenarse localmente en SQLite3 y sistema de archivos local.

**FR-014. Configuración de motores y entorno**  
La app debe permitir configurar idioma por defecto, hotkeys, dispositivos de audio, ruta/modelo de Whisper, uso de GPU/CPU, host/puerto de Ollama con auto-detección de `localhost:11434` y opción de host remoto.

**FR-015. Funcionar offline**  
La aplicación debe seguir siendo operativa sin Internet una vez instalados modelos y binarios locales; si Ollama no está disponible, la grabación y la transcripción primaria deben seguir funcionando siempre que Whisper local esté disponible.

### P1 — Important / Post-MVP cercano

**FR-016. Recuperación de sesiones incompletas**  
Al reiniciar después de un cierre inesperado, la app debe detectar grabaciones incompletas y ofrecer recuperación o descarte seguro.

**FR-017. Detección robusta de thinking**  
El sistema debe identificar modelos thinking por nombre conocido (p. ej. `deepseek-r1`, `qwen3`) y por estructura de payload, configurable desde ajustes avanzados.

**FR-018. Política de almacenamiento del razonamiento**  
La app debe soportar al menos `hide_thinking` y `store_but_hide`; en builds internas podrá habilitarse `show_for_debug`.

**FR-019. Abstracción futura para API**  
La capa de dominio debe exponer operaciones internas claramente separadas para futura exposición HTTP/local API sin acoplarlas a la UI de `egui`.

### P2 — Nice to Have / Future

**FR-020. Detección inferida de app fuente**  
Inferir metadatos de la aplicación de reunión activa cuando sea posible (Zoom, Teams, Meet), sin depender de integraciones oficiales.

**FR-021. Diarización local mejorada**  
Incorporar pipeline opcional de identificación de hablantes si la carga computacional lo permite.

## 6. Non-Functional Requirements

### Performance

- Inicio de grabación en menos de 1 segundo desde la acción del usuario.
- Captura sin cortes perceptibles durante sesiones de al menos 2 horas en Linux y macOS arm64.
- Uso de CPU y RAM razonable para coexistir con apps de videoconferencia y con inferencia local.
- El pipeline debe segmentar audio o resumir por lotes cuando la duración o el consumo de memoria lo requiera.

### Reliability

- Manejo tolerante a fallos de Ollama: si no está disponible, la app conserva grabación y puede marcar el resumen como pendiente.
- Manejo tolerante a fallos de Whisper: errores de carga de modelo deben producir mensajes claros y permitir reintento o ruta alternativa.
- Protección frente a grabaciones vacías, cambios de dispositivo y falta de espacio en disco.

### Security & Privacy

- Todo el procesamiento debe ser local por defecto; no se debe enviar información a servicios cloud en v1.
- No habrá cifrado en reposo en esta versión; esto debe documentarse explícitamente para el usuario.
- La aplicación debe dejar claro en onboarding y documentación que la seguridad depende del equipo local del usuario.

### Usability

- La app debe poder usarse con aprendizaje mínimo: iniciar, detener, marcar highlights y revisar resultados en un flujo claro.
- La UI debe ocultar siempre razonamiento thinking en escenarios de uso normal.
- La experiencia debe ser suficientemente compacta para coexistir con una videollamada en curso.

### Portability

- Linux Ubuntu y derivados: prioridad alta.  
- macOS arm64: prioridad alta.  
- Windows 11: soporte secundario en v1.

## 7. Technical Constraints

### Stack y arquitectura

- Lenguaje principal: Rust.  
- UI desktop: `eframe/egui`.  
- Persistencia: SQLite3 para metadatos; sistema de archivos local para audio y exportaciones.  
- STT principal: integración directa de Whisper en Rust usando `whisper-rs`/`whisper.cpp` o wrapper equivalente.  
- LLM para resúmenes: Ollama local o remoto configurable, con soporte para modelos ligeros y modelos thinking.

### Restricciones de infraestructura

- Debe funcionar en equipos con 4–8 GB de VRAM o solo CPU moderna, evitando dependencias innecesarias.
- Ollama debe autodetectarse en entorno local (`localhost:11434`) y permitir override manual a host remoto.
- No se implementará API pública en v1, pero la arquitectura debe mantener separación por capas: captura, orquestación, IA, persistencia, presentación.

### Restricciones de producto

- Solo español e inglés en v1.
- Sin integración con calendario.
- Sin exportación DOCX.
- Sin cifrado en reposo en v1.

## 8. Data Requirements

### Fuentes de datos

- Audio del sistema capturado por loopback.  
- Audio de micrófono opcional.  
- Configuración de usuario.  
- Resultados de Whisper.  
- Respuestas de Ollama para resúmenes.

### Modelo de datos base

**Recording**  
`id`, `created_at`, `updated_at`, `title`, `meeting_id?`, `source_app`, `duration_seconds`, `audio_path`, `sample_rate`, `channels`, `language`, `has_transcript`, `has_summaries`.

**TranscriptSegment**  
`id`, `recording_id`, `start_sec`, `end_sec`, `speaker_label?`, `text`.

**Highlight**  
`id`, `recording_id`, `timestamp_sec`, `label`.

**Summary**  
`id`, `recording_id`, `template`, `content`, `model_name`, `is_thinking_model`, `raw_thinking?`.

**UserSettings**  
`id`, `language_default`, `hotkey_start_stop`, `hotkey_highlight`, `audio_output_device`, `audio_input_device`, `whisper_model_path`, `whisper_use_gpu`, `ollama_host`, `ollama_port`, `use_ollama_for_stt`, `summary_model`, `summary_stream_mode`, `summary_thinking_policy`.

### Reglas de gestión de datos

- Todo se almacena localmente.  
- No se envía telemetría sensible a terceros.  
- `raw_thinking` es opcional y no debe mostrarse en UI final.
- Los archivos de audio deben conservarse aunque falle el procesamiento posterior.

## 9. UI/UX Requirements

### Flujo principal

1. Usuario instala y abre Scrivano.  
2. Onboarding explica permisos de audio, hotkeys, motor Whisper local y resúmenes con Ollama.  
3. El usuario configura dispositivos, idioma y conexión a Ollama si aplica.  
4. Durante reunión, un panel compacto muestra tiempo, VU meter, estado y botón/hotkey de highlight.  
5. Al detener la sesión, la app pasa a “Procesando” para transcripción y resumen.  
6. En revisión, se muestra reproductor, transcripción alineada por tiempo, highlights y resúmenes.

### Requisitos UX explícitos

- La UI nunca debe mostrar thinking/reasoning al usuario final en flujo normal.
- Debe existir feedback claro para estados: grabando, procesando, pendiente, error, listo.
- Debe haber mensajería clara para errores de Whisper, Ollama, disco insuficiente y permisos de audio.
- La UI debe ser compacta y utilizable en escritorio mientras otra ventana ocupa el foco principal.

### Requisitos de accesibilidad / interacción

- Hotkeys configurables.  
- Navegación clara de historial.  
- Acciones de exportación simples.  
- Distinción visible entre audio disponible, transcripción disponible y resumen disponible.

## 10. Risks & Assumptions

### Riesgos

1. **Captura de audio del sistema en Linux/macOS puede variar por backend o permisos.**  
Mitigación: abstraer capa de captura y validar tempranamente PipeWire/PulseAudio en Linux y estrategia en macOS.

2. **Whisper local puede degradar rendimiento en hardware limitado.**  
Mitigación: permitir selección de modelo, CPU/GPU, procesamiento por lotes y ruta alternativa vía Ollama.

3. **Modelos thinking generan payloads heterogéneos.**  
Mitigación: parser robusto por nombre de modelo + estructura de respuesta + tests con streaming y non-streaming.

4. **UI en `egui` puede acoplarse demasiado a lógica de negocio si no se diseña bien la arquitectura.**  
Mitigación: separar casos de uso, repositorios y adaptadores antes de construir pantallas.

5. **Sesiones largas pueden provocar archivos grandes, consumo elevado de memoria o errores de recuperación.**  
Mitigación: escritura incremental, checkpoints, colas de procesamiento y validación de espacio en disco.

### Suposiciones

- Los usuarios aceptarán instalar modelos locales y configurar Whisper/Ollama si esto mejora privacidad.
- La mayoría del valor del MVP se obtiene con ES/EN, highlights y resumen ejecutivo.
- Un modelo local ligero en Ollama será suficiente para generar resúmenes útiles en hardware objetivo.
- La mayoría de usuarios avanzados preferirá un flujo local antes que depender de integraciones cloud.

### Dependencias

- Integración funcional de Whisper en Rust (`whisper-rs` / `whisper.cpp`).
- Integración estable con API de Ollama para streaming y thinking.
- Backend de captura de audio estable por plataforma.

## 11. Out of Scope

- Soporte para más idiomas aparte de español e inglés en v1.
- Integración con calendarios o proveedores de reuniones.
- Exposición de API pública HTTP en v1.
- Exportación a DOCX u otros formatos ofimáticos cerrados.
- Sincronización cloud o almacenamiento remoto administrado por la app.
- Cifrado local en reposo en esta versión.
- Mobile app o companion app en esta fase.

## Decision: LLM Runtime Strategy

### Decisión

Para Scrivano v1 se adopta una estrategia híbrida: **Whisper embebido + Ollama externo**. Esto significa que la transcripción de audio se ejecuta principalmente dentro de la propia aplicación mediante una integración nativa en Rust sobre `whisper.cpp`/`whisper-rs`, mientras que la generación de resúmenes y transformaciones de texto de alto nivel se delega a Ollama como runtime externo oficial.

### Motivo de la decisión

Esta decisión ofrece el mejor equilibrio entre experiencia nativa, control técnico, mantenibilidad del producto y experiencia de usuario final. Whisper embebido encaja muy bien como motor especializado de transcripción porque la aplicación controla directamente el pipeline de audio, el progreso, la persistencia local y la recuperación de errores, sin depender de un proceso externo para una capacidad crítica.

Para la capa LLM, Ollama resuelve problemas de producto que serían costosos de reconstruir dentro de Scrivano: distribución y gestión de modelos, API local, soporte de streaming, y compatibilidad operativa con modelos thinking/reasoning que separan `thinking` y `content`. Esto reduce complejidad de implementación, acelera el MVP y mejora la experiencia de usuarios técnicos y no técnicos frente a un enfoque de LLM completamente embebido basado en `llama.cpp` desde el día uno.

### Alternativas consideradas

**1. LLM embebido dentro de Scrivano con `llama.cpp` o runtime equivalente**  
Ventajas: mayor sensación de aplicación autocontenida, control fino sobre modelos GGUF, menos dependencia de procesos externos.  
Desventajas: obliga a construir gestión de descarga de modelos, compatibilidad por plataforma, lifecycle del runtime, selección de quantización, administración de memoria, y una UX de setup que Ollama ya resuelve razonablemente bien.

**2. Ollama para todo, incluyendo STT y resúmenes**  
Ventajas: una sola integración de inferencia, operación uniforme por HTTP, menor lógica nativa en la app.  
Desventajas: introduce dependencia externa incluso para la transcripción, que es una capacidad core del producto, y empeora la sensación de aplicación nativa/offline-first si Ollama no está levantado o bien configurado.

### Consecuencia arquitectónica

Scrivano debe modelar explícitamente dos abstracciones independientes:

- `TranscriptionEngine`, con implementación principal `EmbeddedWhisperEngine` y opcional `OllamaSttEngine`.
- `SummaryEngine`, con implementación principal `OllamaSummaryEngine`.

La UI y los casos de uso nunca deben depender directamente de Ollama ni de `whisper.cpp`; deben depender de estas interfaces para preservar testabilidad y permitir futuros backends alternativos.

### Política de producto derivada

- **Ruta oficial v1 de transcripción:** Whisper embebido.
- **Ruta oficial v1 de resumen:** Ollama externo.
- **Ruta avanzada/contingencia para STT:** Ollama opcional.
- **Ruta futura potencial:** backend embebido de LLM tipo `llama.cpp` si se justifica una edición portable/autocontenida.

### Impacto en onboarding y UX

El onboarding debe presentar esta decisión de forma simple:

- Scrivano ya puede transcribir localmente con Whisper integrado.
- Para generar resúmenes, el usuario debe tener Ollama disponible en local o remoto.
- La app debe autodetectar `localhost:11434`, permitir edición manual de host/puerto y validar conectividad/modelo configurado antes de ejecutar un resumen.

### Impacto en implementación

- El instalador o primera ejecución debe verificar disponibilidad de modelos Whisper locales y guiar su descarga/configuración.
- Debe existir un asistente de detección de Ollama con validación de host, puerto y modelo de resumen.
- Las pruebas de integración deben cubrir combinaciones: `Whisper embebido + Ollama ok`, `Whisper embebido + Ollama caído`, `Ollama STT opcional habilitado`, y `modelo thinking con streaming`.

## 12. Open Questions

1. **¿Qué backend de captura de audio será estándar en Linux?**  
Owner: Engineering Lead  
Deadline: Semana 1  
Impacto: Define arquitectura base de captura y test matrix.

2. **¿Qué estrategia exacta se usará en macOS arm64 para capturar system audio de forma confiable?**  
Owner: Platform Engineer  
Deadline: Semana 2  
Impacto: Afecta viabilidad del soporte prioritario en macOS.

3. **¿Se almacenará `raw_thinking` en builds de producción o solo en debug/internal?**  
Owner: Product + Engineering  
Deadline: Semana 2  
Impacto: Afecta esquema final, almacenamiento y postura de privacidad.

4. **¿Cuál es el modelo local por defecto recomendado para resumen en hardware de 4–8 GB VRAM?**  
Owner: AI Engineer  
Deadline: Semana 2  
Impacto: Afecta UX inicial y performance.

5. **¿Qué formato de audio interno será canonical para el pipeline completo?**  
Owner: Audio Engineer  
Deadline: Semana 1  
Impacto: Afecta compatibilidad, tamaño y costo de transcripción.

6. **¿La diarización queda fuera del MVP o entra como feature flag experimental?**  
Owner: Product Manager  
Deadline: Semana 3  
Impacto: Cambia complejidad del pipeline de transcripción.

7. **¿Cómo se priorizará Windows 11 si Linux/macOS consumen la mayor parte del timeline?**  
Owner: Product + Engineering  
Deadline: Semana 4  
Impacto: Afecta expectativa comercial y roadmap.

## Implementation Notes for AI-Assisted Development

### Arquitectura sugerida

- `app/` UI `egui` + estados visuales.  
- `domain/` entidades, casos de uso y reglas.  
- `audio/` captura de sistema, micrófono, mezcla, archivos temporales.  
- `transcription/` adaptador Whisper Rust + adaptador alternativo Ollama STT.  
- `summarization/` cliente Ollama, parser thinking, streaming assembler.  
- `storage/` repositorios SQLite3 + filesystem.  
- `export/` TXT, MD, JSON, SRT, VTT.  
- `platform/` abstracciones Linux/macOS/Windows.

### Recomendaciones de implementación

- Definir interfaces primero (`RecordingService`, `TranscriptionEngine`, `SummaryEngine`, `StorageRepo`).
- Mantener pipelines cancelables y con progreso observable para UI.  
- Tratar thinking como dato interno del adaptador, no del dominio de presentación.  
- Implementar política `stream_mode=auto` con heurística por longitud de prompt, capacidad del modelo y tamaño esperado de salida.
- Crear fixtures de respuesta Ollama para: estándar, streaming, thinking streaming, thinking non-streaming, payload inválido.

### Validación mínima del PRP

Este PRP será “bueno” cuando permita a un equipo o a un agente de IA implementar Scrivano sin necesitar aclaraciones constantes sobre:  
- objetivo del producto,  
- límites del MVP,  
- comportamiento de Whisper y Ollama,  
- manejo de modelos thinking,  
- persistencia local,  
- prioridades de plataforma,  
- riesgos técnicos y supuestos pendientes.
