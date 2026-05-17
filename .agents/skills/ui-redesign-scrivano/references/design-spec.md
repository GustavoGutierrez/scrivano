# Nebula Dark — Visual Design Specification

> Reference document for the Scrivano UI redesign.
> All values verified for WCAG AA compliance on dark backgrounds.

---

## Color Palette

### Backgrounds (Depth Hierarchy)

| Token | Hex | RGB | Usage |
|-------|-----|-----|-------|
| `BG_VOID` | `#090A0F` | 9, 10, 15 | Root window background |
| `BG_NEBULA` | `#131620` | 19, 22, 32 | Panel backgrounds, sidebar |
| `BG_STARDUST` | `#1A1F2E` | 26, 31, 46 | Card surfaces |
| `BG_STARDUST_HOVER` | `#202637` | 32, 38, 55 | Card hover state |
| `BG_ECLIPSE` | `#2A3045` | 42, 48, 69 | Borders, separators |

### Accents

| Token | Hex | RGB | Usage |
|-------|-----|-----|-------|
| `ACCENT_CYAN` | `#00D4FF` | 0, 212, 255 | Primary actions, selection |
| `ACCENT_CYAN_HOVER` | `#33DEFF` | 51, 222, 255 | Button hover |
| `ACCENT_PURPLE` | `#A855F7` | 168, 85, 247 | AI features, magic |
| `ACCENT_PURPLE_HOVER` | `#BC73FA` | 188, 115, 250 | AI button hover |
| `ACCENT_EMERALD` | `#10B981` | 16, 185, 129 | Success, recording |
| `ACCENT_EMERALD_HOVER` | `#34D399` | 52, 211, 153 | Success hover |
| `ACCENT_AMBER` | `#F59E0B` | 245, 158, 11 | Warnings, pending |
| `ACCENT_CRIMSON` | `#EF4444` | 239, 68, 68 | Danger, stop, delete |

### Text

| Token | Hex | RGB | Contrast on VOID | Usage |
|-------|-----|-----|------------------|-------|
| `TEXT_STARLIGHT` | `#F1F5F9` | 241, 245, 249 | 18.5:1 | Body text, headings |
| `TEXT_MOON` | `#94A3B8` | 148, 163, 184 | 7.2:1 | Secondary labels |
| `TEXT_DUST` | `#64748B` | 100, 116, 139 | 4.6:1 | Hints, placeholders |

All text/background pairs meet WCAG AA (>= 4.5:1 for body text, >= 3:1 for large text).

---

## Typography Scale

| Level | Size | Weight | Line Height | Usage |
|-------|------|--------|-------------|-------|
| H1 (Display) | 28px | Bold | 1.3 | Tab titles, major section headers |
| H2 (Heading) | 20px | Bold | 1.3 | Card titles, subsection headers |
| H3 (Subhead) | 16px | Medium | 1.4 | Field labels, group headers |
| Body | 14px | Regular | 1.5 | General content, descriptions |
| Caption | 12px | Regular | 1.5 | Auxiliary text, timestamps |

**Rules:**
- Minimum font size is 12px — never go below
- Maximum 3 font sizes visible simultaneously on screen
- Headings use `Strong` weight, body uses `Regular`

---

## Spacing Tokens

| Token | Value | Usage |
|-------|-------|-------|
| `SPACING_SECTION` | 24px | Between major content sections |
| `SPACING_CARD_PAD` | 20px | Internal card padding (horizontal) |
| `SPACING_ELEMENT` | 12px | Between related UI elements |
| `SPACING_TIGHT` | 8px | Between compact/inline elements |
| `SPACING_MICRO` | 4px | Between icon+text, tiny gaps |

**Layout widths:**
- Settings content max width: 760px
- Settings field max width: 420px
- Sidebar width: 172px
- Recording action button: 320px x 56px

---

## Border Radii

| Token | Value | Usage |
|-------|-------|-------|
| `ROUNDING_CARD` | 12px | Content cards, frames |
| `ROUNDING_BUTTON` | 10px | Action buttons |
| `ROUNDING_SMALL` | 6px | Small badges, inputs |
| `ROUNDING_PILL` | 20px | Status pills, tags |

---

## Spectrum Visualizer Spec

- **Bars:** 48 vertical bars, mirrored above/below centerline
- **Width:** auto-calculated from available space, 25% gap between bars
- **Max height:** 35px per side (70px total amplitude)
- **Gradient:** 5-stop: cyan → blue → purple → magenta → rose
- **Smoothing:** lerp factor 0.35 per frame (at 60fps)
- **Peak hold:** decay 0.92 per frame, shown as brighter dot above bar
- **Idle state:** gentle sine wave animation when no audio
- **Bar tops:** rounded (2px radius) using `ROUNDING_SMALL`

---

## Component States

### Buttons

```
Normal:  fill=ACCENT, text=WHITE, border=ACCENT_HOVER(2px)
Hover:   fill=ACCENT_HOVER, text=WHITE, border=ACCENT_HOVER(2px)
Pressed: fill=ACCENT_DIM, text=WHITE, border=ACCENT_HOVER(2px)
Disabled: fill=BG_NEBULA, text=TEXT_DUST, border=BG_ECLIPSE(1px)
```

### Cards (Recordings list)

```
Normal:   fill=BG_STARDUST, border=BG_ECLIPSE(1px)
Hover:    fill=BG_STARDUST_HOVER, border=ACCENT_CYAN*0.4(1px)
Expanded: fill=BG_STARDUST, border=ACCENT_CYAN(1px) + content visible
```

### Inputs (Combos, TextEdits)

```
Normal: fill=BG_NEBULA, border=BG_ECLIPSE(1px), text=TEXT_STARLIGHT
Focus:  fill=BG_NEBULA, border=ACCENT_CYAN(2px)
Error:  fill=BG_NEBULA, border=ACCENT_CRIMSON(2px)
```

---

## Layout Diagrams

### Recording Tab

```
┌─────────────────────────────────────────────────────┐
│ [Tab1] [Tab2] [Tab3]                                │ ← 12px padding
├─────────────────────────────────────────────────────┤
│                                                     │ ← 24px gap
│   ┌─────────────────────────────────────┐          │
│   │  ● INICIAR GRABACIÓN  (320x56)     │          │ ← Card: BG_STARDUST
│   │  [progreso si aplica]               │          │   ROUNDING_CARD
│   │  ● GRABANDO  23:45                  │          │
│   └─────────────────────────────────────┘          │
│                                                     │ ← 24px gap
│   ┌─────────────────────────────────────┐          │
│   │ ♫ Audio en tiempo real              │          │ ← Card: spectrum
│   │ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│          │   48 bars, 90px height
│   └─────────────────────────────────────┘          │
│                                                     │ ← 12px gap
│   Transcripción (H3)                                │
│   ┌─────────────────────────────────────┐          │
│   │ [transcript text edit, 150px max]   │          │
│   └─────────────────────────────────────┘          │
│                                                     │ ← 24px gap
│   ▼ Grabaciones recientes (3)   (H3, clickable)     │
│   ┌─────────────────────────────────────┐          │
│   │ ▶ Título...  ⏱ 23:45  📄  ✨      │          │ ← Card colapsada
│   └─────────────────────────────────────┘          │
│   ┌─────────────────────────────────────┐          │
│   │ ▼ Título expandido                  │          │ ← Card expandida
│   │   [Exportar texto ▼] [Audio ▼]      │          │
│   │   ✨ Resumir: [Ejecutivo] [Completo] │          │
│   │   [Tareas] [Jira] [Decisiones]      │          │
│   │   ───────────────────────            │          │
│   │   📋 Ejecutivo (via llama3)          │          │
│   │   [summary content...]              │          │
│   └─────────────────────────────────────┘          │
└─────────────────────────────────────────────────────┘
```

### Settings Tab

```
┌─── Sidebar (172px) ───┬─── Content (760px max) ───────┐
│                       │                                │
│ Configuración (H1)    │  🎤 Audio    [✔ Activo]        │
│                       │                                │
│ ● Audio        [✓]    │  Descripción amigable de       │
│   Transcripción [✓]   │  qué hace esta sección...      │
│   IA (Ollama)  [✓]    │                                │
│   Resúmenes    [⚠]    │  Micrófono                     │
│   Prompts      [ ]    │  [Built-in Audio ▼]            │
│   Sistema      [✓]    │                                │
│                       │  Salida de audio               │
│                       │  [Speakers ▼]                  │
│                       │                                │
│                       │  ┌──────────────────────┐      │
│                       │  │ Guardar configuración │      │
│                       │  └──────────────────────┘      │
│                       │  ✓ Configuración guardada      │
└───────────────────────┴────────────────────────────────┘
```

---

## Contrast Verification

All pairs verified at https://webaim.org/resources/contrastchecker/:

| Foreground | Background | Ratio | WCAG AA |
|------------|-----------|-------|---------|
| TEXT_STARLIGHT (#F1F5F9) | BG_VOID (#090A0F) | 18.5:1 | ✓ AAA |
| TEXT_STARLIGHT (#F1F5F9) | BG_STARDUST (#1A1F2E) | 14.2:1 | ✓ AAA |
| TEXT_MOON (#94A3B8) | BG_VOID (#090A0F) | 7.2:1 | ✓ AA |
| TEXT_MOON (#94A3B8) | BG_STARDUST (#1A1F2E) | 5.5:1 | ✓ AA |
| TEXT_DUST (#64748B) | BG_VOID (#090A0F) | 4.6:1 | ✓ AA |
| TEXT_DUST (#64748B) | BG_STARDUST (#1A1F2E) | 3.7:1 | ⚠ Large text only |
| TEXT_WHITE (#FFFFFF) | ACCENT_CYAN (#00D4FF) | 1.5:1 | ✗ (use for large bold only) |
| TEXT_WHITE (#FFFFFF) | ACCENT_CRIMSON (#EF4444) | 3.8:1 | ⚠ Large text only |
| TEXT_WHITE (#FFFFFF) | ACCENT_EMERALD (#10B981) | 3.4:1 | ⚠ Large text only |

**Action required:** Button text on accent backgrounds should use FONT_H3 (16px bold) or larger to meet WCAG large-text threshold.
