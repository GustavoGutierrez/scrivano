# PRD: Scrivano — Grabación Inteligente de Reuniones

## 1. Overview

**Scrivano** es una aplicación de escritorio nativa (Rust + egui/eframe) para PC que graba el audio del sistema y del micrófono durante reuniones en línea, genera transcripciones en español e inglés y produce resúmenes accionables usando modelos locales: Whisper integrado directamente en Rust para transcripción y un modelo de redacción (por ejemplo, un modelo ligero tipo gemma4:e2b) ejecutado vía Ollama. Todo funciona en local, almacena sus datos en SQLite3 y prioriza Linux (Ubuntu y derivados) y macOS arm64, con soporte secundario para Windows 11.

## 2. Problem

Los profesionales que asisten a reuniones online (Zoom, Teams, Meet, etc.) suelen depender de tomar notas manuales o grabaciones dispersas, lo que provoca pérdida de información clave, dificultad para revisar acuerdos y baja trazabilidad de tareas. Las herramientas de transcripción basadas en la nube implican riesgos de privacidad y dependencia de conexión, mientras que muchas soluciones locales existentes no están optimizadas para flujos reales de reuniones (atajos globales, highlights, resúmenes estructurados) ni para funcionar en equipos con recursos limitados.

## 3. Goals

- Permitir grabar audio del sistema y micrófono de cualquier aplicación de reunión en PC con experiencia de “un clic” (atajo global o botón persistente).
- Producir transcripciones automáticas de alta calidad en español e inglés usando Whisper integrado en Rust como motor principal, con opción alternativa de usar Whisper u otros modelos STT vía Ollama si el usuario así lo configura.
- Generar resúmenes automáticos configurables mediante plantillas básicas (p. ej. resumen ejecutivo, tareas, decisiones) usando un modelo local ligero vía Ollama (por ejemplo gemma4:e2b).
- Soportar modelos de resumen “thinking” (razonadores) vía Ollama, de forma que Scrivano pueda aprovechar su mejor calidad de respuesta pero muestre al usuario únicamente la respuesta final, ocultando el razonamiento interno.
- Detectar y aprovechar el modo streaming de Ollama, eligiendo automáticamente entre streaming o respuesta completa en función de las capacidades del modelo y del tamaño esperado del resumen para optimizar latencia y experiencia de usuario.
- Permitir marcar momentos clave durante la reunión (“highlights” mediante hotkey) y reflejarlos en la línea de tiempo y en la transcripción.
- Ofrecer exportación de grabaciones, transcripciones y resúmenes en formatos de texto simples (TXT, Markdown, JSON, SRT/WebVTT) sin dependencias de formatos ofimáticos cerrados.
- Garantizar que todo el procesamiento y almacenamiento se realicen en local, sin enviar datos a la nube.
- Usar SQLite3 como base de datos embebida para persistencia de metadatos, transcripciones, resúmenes y configuración.
- Priorizar compatibilidad y usabilidad en Linux (especialmente Ubuntu y derivados) y macOS arm64; Windows 11 se considera plataforma soportada en segundo plano.

## 4. Non-Goals

- No se implementará en esta versión soporte para más idiomas distintos de español e inglés.
- No se implementará integración con calendario (Google Calendar, Outlook, etc.) en esta versión.
- No se implementará un API HTTP público en v1; solo se tendrán en cuenta lineamientos para facilitar su desarrollo futuro.
- No se proporcionará exportación a formatos ofimáticos tipo DOCX.
- No se utilizarán servicios en la nube para transcripción o resumen; todo se procesará de forma local con Whisper integrado en Rust y, opcionalmente, Ollama.
- No se implementará cifrado en reposo de los audios ni de los datos en esta versión; el enfoque de seguridad se basa en almacenamiento local y ausencia de sincronización remota.

## 5. User Stories

- Como profesional que asiste a reuniones online, quiero iniciar y detener la grabación del audio del sistema con un atajo de teclado global para no interrumpir mi flujo de trabajo.
- Como usuario, quiero que Scrivano detecte y use el dispositivo de salida de audio adecuado (altavoces, auriculares, etc.) para grabar lo que escucho en la reunión.
- Como usuario, quiero poder añadir marcadores durante la reunión para señalar momentos importantes que luego pueda revisar rápidamente.
- Como usuario, quiero obtener una transcripción automática de la reunión en español o inglés con identificación básica de segmentos para entender el contexto temporal.
- Como usuario, quiero recibir resúmenes automáticos de la reunión (decisiones, tareas, puntos clave) sin tener que leer toda la transcripción.
- Como usuario, quiero que todas las grabaciones y datos se guarden únicamente en mi equipo y poder trabajar totalmente offline.
- Como usuario desarrollador o power user, quiero que la arquitectura interna facilite la futura exposición de un API para integrarlo con otras herramientas.

## 6. Functional Requirements

**FR-1:** La aplicación debe capturar audio del sistema (loopback) y, opcionalmente, el micrófono del usuario, generando al menos un flujo mezclado y, si es posible, pistas separadas.

**FR-2:** Debe existir un mecanismo de inicio/parada de grabación mediante:
- Botón en la UI principal.
- Atajo de teclado global configurable.

**FR-3:** La aplicación debe mostrar en tiempo real el estado de grabación (grabando, pausado, inactivo) en una vista compacta (por ejemplo, panel flotante o barra en la ventana principal).

**FR-4:** El usuario podrá insertar “highlights” (marcadores) durante la grabación mediante hotkey o botón, que queden registrados con timestamp y etiqueta opcional.

**FR-5:** Tras finalizar la grabación, el sistema enviará el audio al motor de transcripción local principal, implementado en Rust usando Whisper (por ejemplo, vía bindings a whisper.cpp), para obtener texto estructurado en español o inglés.

**FR-6:** El sistema podrá, de forma opcional y configurable, utilizar Whisper u otros modelos STT expuestos por Ollama como ruta alternativa de transcripción (por ejemplo, para probar modelos o configuraciones distintas), manteniendo siempre la opción de usar la integración directa en Rust como ruta por defecto.

**FR-7:** El sistema generará una transcripción segmentada por intervalos de tiempo (start/end) y asociada a la grabación; opcionalmente podrá incluir etiquetas de segmentos (p. ej. S1, S2) si se implementa diarización local.

**FR-8:** El usuario podrá solicitar uno o varios tipos de resumen para una grabación:

- Resumen ejecutivo.
- Lista de tareas.
- Lista de decisiones.

Estos resúmenes se generarán usando un modelo local vía Ollama (p. ej. gemma4:e2b o modelos “thinking”), a través de una capa de integración que:

- Soporte modelos estándar (solo `content`) y modelos “thinking” que separan razonamiento y respuesta (por ejemplo, modelos que exponen campos `thinking`/`reasoning` o tags `<think>...</think>`).
- Extraiga y entregue a la capa de dominio solo el texto de respuesta final (resumen), descartando u ocultando el razonamiento interno.

**FR-9:** Se ofrecerá una vista de historial de grabaciones con filtros por fecha, duración, título y etiquetas.

**FR-10:** El sistema soportará exportación de:

- Audio en formatos estándar (p. ej. WAV/FLAC/MP3).
- Transcripción (TXT, Markdown, JSON).
- Resúmenes (TXT, Markdown, JSON).
- Subtítulos (SRT, WebVTT), alineados con los timestamps.

**FR-11:** Todas las grabaciones, transcripciones, resúmenes y metadatos se almacenarán en disco local; la base de datos de metadatos será SQLite3.

**FR-12:** Debe existir un panel de configuración donde el usuario pueda:

- Seleccionar dispositivos de entrada/salida preferidos.
- Elegir idioma por defecto (es/en).
- Configurar atajos de teclado.
- Configurar parámetros de Whisper integrado en Rust (por ejemplo, ruta de modelos, tamaño de modelo, uso de GPU/CPU).
- Configurar parámetros de Ollama:
  - Detección automática de host y puerto por defecto (p. ej. `http://localhost:11434`).
  - Posibilidad de cambiar a un host remoto (por ejemplo, `http://mi-servidor-remoto:11434`).
  - Selección de modelo LLM para resúmenes (p. ej. `gemma4:e2b` u otro modelo, incluyendo modelos “thinking”).

**FR-13:** La solución debe diseñarse con una separación clara de capas (captura de audio, orquestación, persistencia, IA) para facilitar la implementación de un API en versiones futuras, sin exponer todavía endpoints públicos.

**FR-14:** La aplicación debe funcionar sin conexión a Internet una vez instalados los modelos locales (Whisper integrado y, en su caso, Ollama y modelos LLM); si Ollama o los modelos no están disponibles, deberá mostrarse un error claro y conservar las grabaciones para procesarlas más tarde.

**FR-15:** La capa de integración con Ollama debe ser capaz de tratar de forma diferenciada los modelos reasoning/thinking, mediante:

- Detección por nombre de modelo (por ejemplo, sufijos o nombres conocidos: `deepseek-r1`, `qwen3`, etc.), configurable desde Scrivano.
- Detección por estructura de respuesta:
  - En streaming: lectura de los campos `thinking` / `reasoning` separados del `content` en los chunks, según la especificación de Ollama.
  - En respuestas no–streaming: detección de patrones como tags `<think>...</think>` y separación de razonamiento y respuesta.

En todos los casos, el dominio solo debe recibir la porción “respuesta final” (resumen) ya limpia.

**FR-16:** La integración con Ollama para resúmenes debe:

- Soportar tanto llamadas en modo streaming como no–streaming a los modelos de Ollama.
- Exponer una política configurable de uso de streaming:
  - Opción “automático”:
    - Si el modelo soporta streaming, usarlo por defecto para resúmenes esperados largos.
    - Para modelos o prompts pequeños, se puede optar por respuesta no–streaming para simplificar el flujo.
  - Opción “forzar streaming” y “forzar non–streaming” como overrides manuales.
- Cuando se use streaming con modelos thinking:
  - Acumular por separado los fragmentos de `thinking` y de `content`.
  - Empezar a mostrar en la UI solo el contenido de respuesta (`content`), nunca el razonamiento (`thinking`), aunque se reciba.

## 7. UX / UI Behavior

### Flujo de primera ejecución

1. El usuario instala y abre Scrivano (binario nativo compilado con eframe/egui).
2. Se muestra un onboarding corto explicando:
   - Qué graba Scrivano (audio del sistema y opcionalmente micrófono).
   - Cómo se usan los atajos y marcadores.
   - Cómo se usan las funciones de transcripción con Whisper integrado y resúmenes con modelos locales vía Ollama.
3. El usuario concede permisos de audio necesarios (loopback/micrófono) y define dispositivos por defecto.
4. El usuario elige idioma principal (es/en) y configura:
   - Host/puerto de Ollama (detectado automáticamente y editable).
   - Modelos por defecto (Whisper integrado y LLM de resumen, incluyendo modelos thinking si se desea).

### Flujo de grabación de reunión

1. El usuario inicia una reunión en cualquier app (Zoom, Teams, Meet, etc.).
2. Desde la bandeja del sistema o mediante atajo, inicia la grabación en Scrivano.
3. Scrivano muestra un panel compacto con:
   - Indicador de tiempo.
   - Nivel de audio (VU meter).
   - Botón para marcar highlight.
   - Estado de grabación (grabando/pausado).
4. El usuario puede insertar marcadores con o sin etiquetas.
5. Al finalizar la reunión, el usuario detiene la grabación.
6. Scrivano pasa a estado “Procesando”:
   - Transcribe el audio con Whisper integrado en Rust (o, opcionalmente, vía Ollama si así está configurado).
   - Genera resúmenes según plantillas seleccionadas usando el modelo LLM configurado en Ollama, manejando internamente reasoning y streaming.

### Flujo de revisión

1. El usuario abre la grabación desde el historial.
2. La vista de detalle muestra:
   - Reproductor de audio con waveform y marcadores.
   - Transcripción alineada por timestamps (y etiquetas de segmento si existen).
   - Panel de resúmenes (ejecutivo, tareas, decisiones) generados por modelos locales; si se usan modelos “thinking”, la UI solo muestra el resumen final, nunca el razonamiento interno.
3. El usuario puede editar títulos, etiquetas, idioma de la transcripción (solo es/en) y dividir/combinar marcadores.
4. El usuario exporta en el formato deseado o copia contenido al portapapeles.

## 8. API Contract (futuro)

En v1 no se implementará un API HTTP pública, pero la arquitectura debe facilitar su incorporación posterior. Esta sección define lineamientos de diseño, no entregables de implementación.

### Lineamientos de diseño

- Mantener una capa interna de “servicio de grabaciones/transcripción” con métodos claramente definidos que puedan exponerse vía HTTP en el futuro.
- Para cada operación principal (iniciar/terminar grabación, añadir highlight, obtener transcripción, obtener resúmenes, exportar) debe existir una función de dominio con parámetros y tipos bien definidos.
- Evitar acoplar la lógica de negocio directamente a la UI (egui/eframe); la UI debe orquestar llamadas a una capa interna que pueda ser “envolvida” por un servidor local en futuras versiones.

Ejemplos de contratos futuros (no implementados en v1):

- `startRecording(source: "system", includeMic: bool, meetingId?: string) -> recordingId`
- `stopRecording(recordingId: string) -> status`
- `addHighlight(recordingId: string, label?: string) -> highlight`
- `getTranscript(recordingId: string) -> Transcript`
- `getSummary(recordingId: string, template: string) -> Summary`

## 9. Data Model

La base de datos principal será SQLite3.

### Tabla/Entidad: Recording

- `id` (UUID)
- `created_at` (datetime)
- `updated_at` (datetime)
- `title` (string)
- `meeting_id` (string opcional, solo para referencia manual)
- `source_app` (string, p. ej. "zoom", "teams", "unknown")
- `duration_seconds` (int)
- `audio_path` (string)
- `sample_rate` (int)
- `channels` (int)
- `language` (string, valores esperados: "es", "en")
- `has_transcript` (bool)
- `has_summaries` (bool)

### Tabla/Entidad: TranscriptSegment

- `id` (UUID)
- `recording_id` (FK Recording)
- `start_sec` (float)
- `end_sec` (float)
- `speaker_label` (string opcional, p. ej. "S1", "S2")
- `text` (text)

### Tabla/Entidad: Highlight

- `id` (UUID)
- `recording_id` (FK Recording)
- `timestamp_sec` (float)
- `label` (string)

### Tabla/Entidad: Summary

- `id` (UUID)
- `recording_id` (FK Recording)
- `template` (string, p. ej. "executive", "tasks", "decisions")
- `content` (text) — resumen limpio mostrado al usuario
- `model_name` (string) — modelo usado en Ollama
- `is_thinking_model` (bool) — indica si el modelo es de tipo “thinking”
- `raw_thinking` (text opcional) — razonamiento completo si se decide almacenarlo para depuración (no se muestra en UI)

### Tabla/Entidad: UserSettings

- `id` (UUID)
- `language_default` (string, "es" o "en")
- `hotkey_start_stop` (string)
- `hotkey_highlight` (string)
- `audio_output_device` (string)
- `audio_input_device` (string)
- `whisper_model_path` (string, ruta o identificador del modelo Whisper usado por la integración en Rust)
- `whisper_use_gpu` (bool)
- `ollama_host` (string, p. ej. "http://localhost")
- `ollama_port` (int, p. ej. 11434)
- `use_ollama_for_stt` (bool, indica si se quiere usar Ollama como alternativa para STT)
- `summary_model` (string, nombre del modelo LLM en Ollama, p. ej. "gemma4:e2b")
- `summary_stream_mode` (string, valores: "auto", "stream", "non_stream")
- `summary_thinking_policy` (string, valores: "hide_thinking", "store_but_hide", "show_for_debug"; por defecto "hide_thinking")

## 10. Edge Cases

- Reunión sin audio (silencio prolongado): la grabación debe completarse sin errores; la transcripción puede estar vacía.
- Cambios de dispositivo de audio durante la reunión: el sistema debería intentar reconectarse al nuevo dispositivo sin detener la grabación o notificar claramente al usuario.
- Reuniones muy largas (p. ej. > 8 horas): asegurar que no haya desbordes de tamaño de archivo ni pérdida de datos; la UI debe indicar claramente el tiempo y tamaño estimado.
- Falta de espacio en disco: Scrivano debe alertar antes o durante la grabación y detenerla de forma segura si se alcanza un límite crítico.
- Servicio de Ollama no disponible (no iniciado, puerto incorrecto, error en modelos):
  - La grabación debe conservarse y marcarse como “pendiente de procesar” si se configura `use_ollama_for_stt` y falla la transcripción.
  - Para resúmenes, se debe informar claramente que no pueden generarse hasta que Ollama esté disponible.
- Whisper integrado no disponible o mal configurado (ruta de modelo incorrecta, fallo al cargar): informar error claro y permitir que el usuario pruebe la vía alternativa de Ollama si está activada.
- Uso simultáneo de varias instancias de Scrivano: bloquear o advertir para evitar conflictos en el dispositivo de audio.
- Cierre inesperado de Scrivano o del sistema operativo durante la grabación: al reiniciar, Scrivano debe ser capaz de detectar archivos de audio incompletos y ofrecer opciones de recuperación o limpieza.

## 11. Constraints

### Tecnología

- **Framework de UI: egui/eframe (Rust nativo, sin WebView).**
  - Se elige `eframe` en lugar de Tauri por las siguientes razones:
    - egui/eframe **ya está implementado y funcionando** en Linux con la arquitectura actual del proyecto.
    - Compila de forma nativa en Linux, macOS y Windows **sin depender de un WebView ni de Node.js**, produciendo un único binario estático fácil de distribuir.
    - El modelo de renderizado **immediate mode** es ideal para las necesidades de Scrivano: waveform en tiempo real, VU meters, barras de progreso y actualizaciones de estado a 30 fps sin complejidad adicional.
    - La barrera real para la distribución multiplataforma **no es la UI** sino la capa de captura de audio del sistema (que requiere APIs distintas por plataforma). Tauri no resuelve esa barrera y añadiría una capa innecesaria de HTML/CSS/JS + bridge Rust/JS.
    - Para la distribución se generan binarios nativos por plataforma (`.deb` en Linux, `.dmg`/bundle en macOS, installer en Windows), sin necesitar un runtime de navegador embebido.
- Plataformas soportadas:
  - Linux (especialmente distribuciones tipo Ubuntu/Debian) — prioridad alta.
  - macOS arm64 — prioridad alta.
  - Windows 11 — prioridad secundaria.
- Captura de audio del sistema (adaptadores específicos por plataforma — es el verdadero trabajo de portabilidad):
  - Linux: PulseAudio / PipeWire loopback (ya implementado vía `libpulse-binding`).
  - macOS arm64: CoreAudio o ScreenCaptureKit para captura del audio del sistema (loopback). No requiere dispositivos virtuales adicionales en macOS 13+.
  - Windows 11: WASAPI loopback (para versiones futuras, sin priorizar en v1).
- IA local:
  - Whisper integrado en Rust como motor principal de transcripción (bindings a librerías tipo whisper.cpp u otras implementaciones eficientes en CPU/GPU).
  - Ollama opcional:
    - Como alternativa para STT (ejecutar Whisper u otros modelos de audio soportados).
    - Como vía principal para ejecutar el modelo LLM de resúmenes (por ejemplo, gemma4:e2b) y modelos thinking, optimizados para entornos de recursos limitados.
  - La integración con Ollama debe usar la API oficial soportando:
    - El campo de razonamiento/thinking separado del contenido final cuando se use la funcionalidad de thinking.
    - Streaming de respuestas, gestionando correctamente chunks que contengan solo reasoning, solo content o ambos.

### Performance

- La grabación debe funcionar en tiempo real sin cortes perceptibles en equipos estándar de usuario (4–8 GB VRAM, CPU de escritorio o portátil moderna).
- El uso de CPU y memoria debe mantenerse dentro de límites razonables para poder coexistir con apps de videoconferencia.
- El pipeline de transcripción/resumen debe ser tolerante a sesiones largas (varias horas), segmentando el audio si es necesario para evitar consumo excesivo de memoria.

### Security

- No habrá cifrado en reposo en esta versión; todo se almacena en archivos locales y SQLite3.
- No se realizará ningún envío de datos a servicios remotos ni a la nube.
- La aplicación debe dejar claro en la documentación y en el onboarding que todo se procesa en local y que la responsabilidad de protección del equipo recae en el usuario.

## 12. Acceptance Criteria

- Given que el usuario tiene una reunión online en ejecución y Scrivano está instalado en Linux o macOS, When presiona el atajo de teclado de grabación, Then Scrivano comienza a grabar el audio del sistema y muestra el estado de grabación en la UI.
- Given que una grabación está en curso, When el usuario presiona el atajo de highlight, Then se crea un marcador en el timestamp actual y aparece en la UI.
- Given que la grabación ha finalizado y Whisper integrado está correctamente configurado, When Scrivano procesa el audio, Then se genera una transcripción completa y navegable por timestamps en español o inglés según la configuración.
- Given que la transcripción existe, When el usuario solicita un resumen ejecutivo, Then Scrivano devuelve un resumen con los puntos clave de la reunión utilizando el modelo LLM configurado en Ollama.
- Given que el usuario intenta grabar sin espacio suficiente en disco, When Scrivano detecta la condición, Then muestra una advertencia y evita iniciar la grabación o la detiene de forma segura.
- Given que Ollama no está disponible pero Whisper integrado sí, When el usuario finaliza la grabación, Then Scrivano transcribe usando Whisper integrado y solo las funciones de resumen fallan con un mensaje claro.
- Given que el usuario exporta la transcripción, When elige un formato soportado (TXT/Markdown/JSON/SRT/VTT), Then Scrivano genera un archivo legible en ese formato.
- Given que el usuario ha configurado un modelo de resumen “thinking” en Ollama, When solicita un resumen de una grabación, Then Scrivano genera el resumen usando ese modelo y en la UI solo se muestra el texto de respuesta final, sin trazas del razonamiento interno.
- Given que el modelo configurado soporta streaming en Ollama y el modo de streaming está en “automático”, When el usuario solicita un resumen de una grabación larga, Then Scrivano recibe la respuesta en streaming, la renderiza progresivamente y el usuario nunca ve el contenido de reasoning, solo el resumen final.
- Given que el usuario cambia el modo de streaming a “non_stream”, When solicita un resumen, Then Scrivano espera a recibir la respuesta completa antes de mostrarla y el resultado sigue sin mostrar el razonamiento interno del modelo.

## 13. Definition of Done

- Todas las historias de usuario asociadas están implementadas y pasan pruebas funcionales en Linux y macOS arm64.
- Grabación de audio del sistema validada en Linux (Ubuntu o similar) y macOS arm64 durante sesiones de al menos 2 horas sin pérdida de datos.
- Transcripción automática funcionando en español e inglés con Whisper integrado en Rust, con pruebas básicas de calidad.
- Ruta alternativa de transcripción vía Ollama validada (si se habilita en configuración).
- Resúmenes generados para al menos un conjunto de plantillas (ejecutivo, tareas, decisiones) usando un modelo LLM local (p. ej. gemma4:e2b) vía Ollama.
- Integración con modelos thinking validada: el razonamiento se procesa internamente y nunca se muestra en la UI, las respuestas se almacenan correctamente en la tabla Summary.
- Streaming de resúmenes validado en modo “auto” y “stream”, incluyendo manejo correcto de chunks con reasoning y content.
- Marcadores asociados correctamente a timestamps y visibles en la UI y en exportaciones (incluyendo SRT/VTT).
- Exportaciones verificadas para todos los formatos soportados en v1 (audio, TXT, Markdown, JSON, SRT/VTT).
- Esquema de base de datos SQLite3 implementado, migraciones iniciales probadas y documentación técnica actualizada.
- Arquitectura de servicios internos preparada para futura exposición de API (interfaces de dominio definidas y desacopladas de la UI).
- No existen bugs críticos abiertos en el backlog para esta feature.
