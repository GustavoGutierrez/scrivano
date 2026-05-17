# PRP: Rediseño Integral de UI para Scrivano

## 1. Project Overview

**Proyecto:** Rediseño UI/UX de Scrivano — Aplicación de grabación y transcripción de audio
**Pattern:** B (Feature Enhancement — System Redesign)
**Timeline:** 3-4 semanas (diseño e implementación)
**Target:** Usuarios técnicos y profesionales que graban reuniones, calls, y audio del sistema
**Contexto:** La UI actual es funcional pero carece de atractivo visual, jerarquía tipográfica clara, y una experiencia moderna que compita con herramientas como Otter.ai o Fireflies.

---

## 2. Problem Statement

**Los usuarios de Scrivano** enfrentan una interfaz que, aunque funcional, **se siente anticuada y densa** al interactuar con grabaciones y configuraciones.
Esto causa **fatiga visual** (textos pequeños), **curva de aprendizaje innecesaria** (settings poco intuitivos), y **percepción de herramienta amateur** en un producto que técnicamente es sólido.

**Evidencia:**
- Tamaños de fuente 11-14px en todo el sistema (por debajo de estándares de accesibilidad)
- Paleta monocromática sin identidad visual distintiva
- Settings con solo dropdowns genéricos, sin feedback visual del estado actual
- Layout lineal sin jerarquía visual clara entre secciones
- Sin transiciones o micro-interacciones que guíen al usuario

---

## 3. Success Criteria

**Primarias:**
- 100% de textos con jerarquía tipográfica clara (5 niveles: 28/20/16/14/12px)
- Relación de contraste >= 4.5:1 en todos los textos (WCAG AA)
- Settings configurable en <= 2 minutos por un usuario nuevo
- Feedback visual inmediato en cada interacción (hover, click, progreso)

**Secundarias:**
- Paleta de color con identidad propia y diferenciable de otras apps oscuras
- Espectro de audio mantiene fluidez 60fps con animación suave
- Reducción de densidad visual percibida en 40%
- 0 regresiones en funcionalidad existente

**Mínimo aceptable:**
- Textos >= 14px en cuerpo, jerarquía de al menos 3 niveles
- Contraste texto/fondo >= 3:1 mínimo

---

## 4. User Stories (Jobs-to-be-Done)

**Usuario grabando:**
1. Cuando estoy en una reunión, quiero ver claramente el estado de grabación y el audio entrando, para saber que todo funciona sin distraerme.
2. Cuando termino de grabar, quiero ver el resultado de la transcripción inmediatamente, para decidir si necesito regrabar.
3. Cuando reviso grabaciones pasadas, quiero identificar rápidamente cuál es cada una por título y badges visuales, para no perder tiempo buscando.

**Usuario configurando:**
1. Cuando configuro por primera vez, quiero entender cada opción sin leer documentación, para empezar a usar la app en minutos.
2. Cuando cambio de dispositivo de audio, quiero ver confirmación visual de que el cambio se aplicó, para confiar en que grabará correctamente.

---

## 5. Functional Requirements

### P0 (Core — MVP del rediseño)

- **FR-001:** Nueva paleta de colores "Nebula Dark" aplicada globalmente
- **FR-002:** Jerarquía tipográfica de 5 niveles con tamaños definidos
- **FR-003:** Espaciado consistente (márgenes 24px, gaps 16px entre secciones)
- **FR-004:** Estados hover/active con efecto glow sutil en elementos interactivos
- **FR-005:** El espectro de audio mantiene 48 barras, 60fps, animación suave con nuevo gradiente
- **FR-006:** Badges de estado con colores semánticos (grabando/procesando/listo/error)
- **FR-007:** Tarjetas (cards) con sombra sutil y bordes redondeados 12px

### P1 (Importante — Post-MVP)

- **FR-008:** Settings con previews visuales del estado actual (íconos + descripciones + badges)
- **FR-009:** Secciones de settings colapsables con indicador de configuración completada
- **FR-010:** Tooltips contextuales en campos de configuración
- **FR-011:** Variante circular del espectro de audio (opción visual)
- **FR-012:** Animaciones de transición entre tabs (fade suave)

### P2 (Nice-to-have — Futuro)

- **FR-013:** Modo claro/oscuro conmutables
- **FR-014:** Personalización de color de acento por usuario
- **FR-015:** Mini-espectro en la bandeja del sistema

---

## 6. Non-Functional Requirements

**Performance:**
- NFR-001: Renderizado a 60fps durante grabación (sin regresión)
- NFR-002: Repaint solo cuando necesario (no idle rendering)
- NFR-003: Carga de UI < 500ms en inicio

**Accesibilidad:**
- NFR-004: Contraste texto/fondo >= 4.5:1 (WCAG AA)
- NFR-005: Tamaños de fuente escalables con configuración del sistema
- NFR-006: Estados focus visibles en todos los controles

**Consistencia:**
- NFR-007: Mismos espaciados en todas las secciones
- NFR-008: Misma paleta en todos los componentes
- NFR-009: Comportamiento predecible de interacciones

**Mantenibilidad:**
- NFR-010: Constantes de diseño centralizadas (no hardcodeadas)
- NFR-011: Componentes visuales reutilizables (funciones helper)

---

## 7. Technical Constraints

**Framework:**
- egui 0.27+ / eframe (immediate mode GUI) — no se puede cambiar
- Custom painting con `egui::Painter` para espectro y elementos decorativos
- Phosphor Icons (`egui-phosphor`) para iconografía

**Limitaciones de egui:**
- Sin CSS — todo es código Rust imperativo
- Sin animaciones nativas — deben ser manuales vía interpolación
- Sin temas dinámicos fáciles — requiere reconstruir Visuals
- Layout inmediato: anidamiento de `ui.horizontal()` / `ui.vertical()`

**Restricciones de diseño:**
- NO se puede usar HTML/CSS (es app nativa, no web)
- NO se pueden usar fuentes externas fácilmente (requiere cargar archivos .ttf)
- El Painter de egui soporta formas básicas (rect, circle, line, path, mesh, text)

---

## 8. Data Requirements

**Estados visuales a representar:**
- Grabando (rojo pulsante) / Deteniendo (ámbar cuenta regresiva) / Inactivo (verde listo)
- Transcribiendo (barra azul con %) / Mejorando con IA (barra púrpura con %)
- Reproduciendo (verde) / Pausado (ámbar) / Detenido (gris)
- Ollama disponible (verde) / No disponible (ámbar) / Error (rojo)

**Datos del espectro:**
- 48 bandas de frecuencia RMS desde buffer de audio
- Smoothing factor 0.35, peak decay 0.92
- Valores normalizados 0.0-1.0 para altura de barras

---

## 9. UI/UX Requirements

### 9.1 Sistema de Diseño

#### Paleta "Nebula Dark"

```
// Fondos (jerarquía de profundidad)
DEEP_VOID       #090A0F  — Fondo raíz
DARK_NEBULA     #131620  — Paneles
STARDUST        #1A1F2E  — Cards/superficies
ECLIPSE         #2A3045  — Bordes

// Acentos
CYAN_NEON       #00D4FF  — Primario (acciones, selección)
PURPLE_NOVA     #A855F7  — Secundario (IA, magia)
EMERALD         #10B981  — Éxito, grabación
AMBER           #F59E0B  — Advertencia
CRIMSON         #EF4444  — Peligro, detener

// Texto
STARLIGHT       #F1F5F9  — Texto principal
MOON_GRAY       #94A3B8  — Texto secundario
DUST_GRAY       #64748B  — Texto muted
```

#### Jerarquía Tipográfica

```
H1 (Display)    28px Bold   — Título de sección
H2 (Heading)    20px Bold   — Subtítulo
H3 (Subhead)    16px Medium — Label de grupo
Body            14px Regular— Texto general
Caption         12px Regular— Texto auxiliar
```

#### Espaciado

```
SECTION_GAP     24px — Entre secciones principales
CARD_PADDING    20px — Padding interno de cards
ELEMENT_GAP     12px — Entre elementos relacionados
TIGHT_GAP        8px — Entre elementos compactos
```

### 9.2 Estados de Componentes

**Botones:**
- Normal: Fill acento + texto blanco
- Hover: Fill acento más claro + glow sutil (borde 2px más brillante)
- Active/Pressed: Fill acento más oscuro
- Disabled: Fill gris + texto muted

**Cards:**
- Normal: Fill superficie + borde eclipse
- Hover: Fill ligeramente más claro + borde cyan sutil
- Selected/Active: Borde cyan + glow shadow

**Inputs/Combos:**
- Normal: Fill panel + borde eclipse
- Focus: Borde cyan
- Error: Borde crimson

### 9.3 Layout General

```
┌──────────────────────────────────────────────┐
│  [Grabación] [Configuración] [Acerca de]     │ ← Tab bar con indicador activo animado
├──────────────────────────────────────────────┤
│                                              │
│   ┌──────────────────────────────┐           │
│   │      ● INICIAR GRABACIÓN     │           │ ← Botón principal prominente
│   └──────────────────────────────┘           │
│                                              │
│   ┌──────────────────────────────┐           │
│   │ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ │           │ ← Espectro audio 48 barras
│   │ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ │           │   con gradiente cyan→magenta
│   └──────────────────────────────┘           │
│                                              │
│   Transcripción                              │
│   ┌──────────────────────────────┐           │
│   │ texto de transcripción...    │           │
│   └──────────────────────────────┘           │
│                                              │
│   ▼ Grabaciones recientes (3)               │ ← Lista expandible
│   ┌──────────────────────────────┐           │
│   │ 📄 Título reunión   ⏱ 23:45 │           │ ← Card colapsada
│   └──────────────────────────────┘           │
│   ┌──────────────────────────────┐           │
│   │ ▶ Título expandido          │           │ ← Card expandida con detalles
│   │   Acciones: [Exportar] [▶]  │           │
│   │   Resúmenes...              │           │
│   └──────────────────────────────┘           │
└──────────────────────────────────────────────┘
```

### 9.4 Settings Rediseñado

```
┌──── Sidebar ────┬──── Contenido ──────────────────┐
│                  │                                 │
│ ● Audio          │  🎤 Audio        [Capturando ✓] │
│   Transcripción  │                                 │
│   IA (Ollama)    │  Dispositivo actual:            │
│   Resúmenes      │  [Built-in Microphone ▼]        │
│   Prompts        │                                 │
│   Sistema        │  📋 Descripción amigable de     │
│                  │  para qué sirve esta opción...  │
│                  │                                 │
│                  │  ┌─────────────────────────┐    │
│                  │  │   Guardar configuración  │    │
│                  │  └─────────────────────────┘    │
└──────────────────┴─────────────────────────────────┘
```

- Cada sección del sidebar muestra badge de estado (✓ configurado / ⚠ pendiente)
- Las secciones de contenido tienen ícono grande + título + descripción
- Los combos tienen ancho fijo y texto truncado con tooltip

### 9.5 Espectro de Audio

- **Mantener:** 48 barras, 60fps, suavizado, peak hold
- **Mejorar:** Gradiente más vibrante (5 colores), opción de vista circular, glow en picos
- **Nuevo gradiente:** `#00FFFF → #3B82F6 → #8B5CF6 → #EC4899 → #F43F5E`
- Cada barra con esquinas redondeadas superiores (2px radius)
- Línea central sutil de referencia

---

## 10. Riesgos y Supuestos

**Riesgos:**
1. Cambiar tamaños de fuente rompe layouts existentes
   - Mitigación: Testing incremental, sección por sección
2. La paleta nueva reduce contraste en algún estado
   - Mitigación: Verificar con WCAG contrast checker cada par texto/fondo
3. El nuevo gradiente del espectro es menos visible
   - Mitigación: Mantener el gradiente actual como fallback configurable
4. El código se vuelve más complejo con helpers visuales
   - Mitigación: Extraer a módulo `ui/theme.rs` todas las constantes

**Supuestos:**
1. egui 0.27+ soporta todas las operaciones de Painter necesarias
2. Las fuentes Phosphor tienen todos los íconos requeridos
3. El usuario tiene monitor con soporte de color adecuado (sRGB)
4. La app corre en Linux con PulseAudio/PipeWire

**Dependencias:**
- `egui` 0.27+ (ya en uso)
- `egui-phosphor` (ya en uso)
- Ninguna dependencia nueva requerida

---

## 11. Out of Scope

**Explícitamente NO incluido:**
- Cambiar de egui a otro framework GUI (iced, tauri, etc.)
- Modo claro (fase futura)
- Personalización de temas por usuario
- Internacionalización (i18n) del texto de UI
- Soporte para lectores de pantalla avanzados
- Versión web/WASM del rediseño
- Refactorización de lógica de negocio (solo UI)
- Cambios en la estructura de datos o base de datos

---

## 12. Open Questions

**Q1:** El gradiente del espectro — configurable por el usuario o fijo?
- Impacto: Añade complejidad de settings pero da personalización

**Q2:** Layout de la tab de grabación — mantener columna única o explorar 2 columnas?
- Impacto: Layout 2-columnas da más información visible pero requiere rediseño más agresivo

**Q3:** Animaciones de transición entre tabs? (egui no las soporta nativamente)
- Impacto: Requiere implementar interpolación manual de opacidad
