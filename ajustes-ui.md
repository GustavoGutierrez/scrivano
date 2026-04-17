Actúa como un ingeniero senior especializado en Rust + egui/eframe con fuerte criterio de UX.

Tu tarea es REFACTORIZAR y REDISEÑAR la pantalla de Configuración de la aplicación Scrivano para que coincida con un nuevo diseño tipo dashboard oscuro modular, basado en tarjetas y sidebar, manteniendo compatibilidad total con egui 0.27.

## CONTEXTO

Scrivano es una aplicación de escritorio en Rust (egui/eframe) para:
- grabación de audio (mic + sistema)
- transcripción con Whisper
- mejora con Ollama
- generación de resúmenes
- prompts personalizados
- configuración de idioma, atajos y paths

Actualmente la pantalla de configuración es un formulario largo vertical.

Debes transformarla en una UI modular, escaneable, con jerarquía clara.

## OBJETIVO

Convertir la pantalla actual en una interfaz con:

- Sidebar izquierda para navegación
- Área principal con cards (bloques)
- Separación por módulos funcionales
- Jerarquía visual clara
- Menos scroll
- Mejor escaneo visual
- Estados visibles (activo/inactivo/disponible)

## IMPORTANTE (RESTRICCIONES)

- NO uses HTML, CSS ni conceptos web
- NO propongas nada fuera de egui
- NO uses efectos visuales complejos (blur, sombras avanzadas)
- TODO debe ser implementable con:
  - egui::SidePanel
  - egui::CentralPanel
  - egui::Frame
  - egui::CollapsingHeader
  - egui::Grid / horizontal / vertical layouts
  - egui::ComboBox
  - egui::TextEdit
  - egui::Checkbox / Toggle
  - egui::Button
- Usa egui-phosphor para iconos
- Mantén dark theme

## CAMBIOS QUE DEBES IMPLEMENTAR

### 1. Layout general

Reestructura la pantalla en:

- SidePanel izquierdo (ancho fijo)
- CentralPanel con contenido

Sidebar debe contener:
- Audio
- Transcripción
- IA (Ollama)
- Resúmenes
- Prompts
- Sistema

Debe haber estado activo seleccionado.

---

### 2. Sistema de tarjetas (cards)

Cada módulo debe renderizarse dentro de un:

```rust
Frame::group(ui.style())
````

Con:

* padding interno consistente
* separación clara entre cards
* título arriba
* contenido organizado

---

### 3. Sección AUDIO (card)

Debe incluir:

* ComboBox → micrófono
* Botón → actualizar dispositivos
* ComboBox → salida de audio
* Label de estado:

  * "Capturando correctamente" (verde)
  * o warning si falla

Layout:

* labels arriba
* inputs debajo
* spacing consistente

---

### 4. Sección TRANSCRIPCIÓN

Card con:

* título + badge "Activo"
* ComboBox modelo Whisper
* texto informativo:

  * Calidad
  * Latencia

Evitar sobrecarga visual.

---

### 5. Sección IA (OLLAMA) → IMPORTANTE

Debe ser visualmente destacada:

* Toggle activado/desactivado
* Estado: disponible / no disponible
* ComboBox modelo
* Botón actualizar modelos

Lista de features:
✔ Corrección ortográfica
✔ Mejora semántica
✔ Limpieza de ruido

* texto pequeño:
  "Uso estimado: Bajo"

---

### 6. Sección RESÚMENES

Card compacta:

* modelo
* modo streaming
* thinking policy

Usar layout vertical limpio.

---

### 7. Sección PROMPTS

NO mostrar los 3 prompts abiertos.

Usar:

```rust
CollapsingHeader
```

Para:

* Ejecutivo
* Tareas
* Decisiones

Cada uno contiene:

* TextEdit multiline

Por defecto: colapsados.

---

### 8. Sección SISTEMA

Card con:

* idioma (ComboBox)
* atajos (labels o inputs)
* carpeta (path visible)

---

### 9. Botón global

Al final:

* botón centrado:
  "Guardar configuración"

---

## REGLAS VISUALES

* Fondo general oscuro
* Cards ligeramente más claras
* Separación vertical clara entre bloques
* Títulos con mayor tamaño
* Labels en color gris tenue
* Estados:

  * verde = activo
  * amarillo = warning
  * gris = inactivo
* Uso mínimo de iconos (Phosphor)

---

## UX QUE DEBES MEJORAR

* Reducir scroll
* Agrupar por intención
* Mejorar escaneo visual
* Evitar saturación
* Hacer evidente qué está activo
* Mostrar estados del sistema claramente

---

## ESTRUCTURA DE CÓDIGO ESPERADA

Debes organizar el código en funciones reutilizables:

Ejemplo:

* fn render_sidebar(ui: &mut Ui)
* fn render_audio_section(ui: &mut Ui)
* fn render_transcription_section(ui: &mut Ui)
* fn render_ollama_section(ui: &mut Ui)
* fn render_summary_section(ui: &mut Ui)
* fn render_prompts_section(ui: &mut Ui)
* fn render_system_section(ui: &mut Ui)

Cada sección debe ser independiente.

---

## ENTREGA

Quiero que me devuelvas:

1. Código Rust usando egui/eframe
2. Separado por funciones
3. Con layout completo funcional
4. Sin pseudo-código
5. Sin placeholders innecesarios
6. Listo para integrar en el proyecto

---

## BONUS

Si puedes:

* crea un helper tipo "section_card(ui, title, content)"
* crea un pequeño sistema de spacing consistente

---

## PROHIBIDO

* No inventes APIs inexistentes
* No uses librerías externas
* No conviertas esto en web
* No hagas diseño teórico

---

El resultado debe sentirse como una app profesional tipo herramienta de productividad/IA hecha en Rust, no como un formulario.

````

---

# 🔥 Cómo usar este prompt (importante)

Te recomiendo usarlo así:

### 1. Primero (esto que te di)
👉 genera estructura UI

### 2. Luego refinas con esto:

```text
Ahora mejora el código anterior:

- Reduce duplicación
- Mejora spacing
- Haz consistentes los labels
- Asegura que no haya overflow horizontal
- Mejora jerarquía visual con tamaños de texto

No cambies la estructura, solo mejora calidad de implementación.
