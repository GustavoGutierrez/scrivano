---
name: ui-redesign-scrivano
description: "Trigger: rediseñar UI, implementar diseño, componentes visuales, tema nebula, ui theme scrivano, aplicar diseño, crear componente, nueva funcionalidad UI, modificar interfaz, arreglar UI. Implements and enforces the Nebula Dark design system with reusable UI components and custom egui painting patterns for ongoing UI development."
version: 1.1.0
category: design
triggers:
  - 'rediseñar UI'
  - 'implementar diseño'
  - 'componentes visuales'
  - 'tema nebula'
  - 'ui theme scrivano'
  - 'aplicar diseño'
  - 'crear componente'
  - 'nueva funcionalidad UI'
  - 'modificar interfaz'
  - 'arreglar UI'
  - 'agregar botón'
  - 'nueva sección'
dependencies:
  required_mcps: []
  required_tools: []
  required_integrations: []
---

# UI Development Skill — Nebula Dark Design System

## Activation Contract

Use this skill for **ANY** UI work in Scrivano:

- Adding new features that need UI
- Modifying existing screens or components
- Fixing visual bugs or layout issues
- Adding new buttons, cards, sections, or controls
- Initial implementation of the Nebula Dark redesign
- Creating new settings sections or tabs

Do NOT use this skill for:
- Logic or business rule changes (use `rust-engineer`)
- Database or data model changes
- Audio capture or transcription pipeline changes

## Hard Rules

These rules are **NON-NEGOTIABLE**. They are also defined in `.agents/rules/design-system.md`.

1. **Consult `.agents/rules/design-system.md` FIRST** before any UI change
2. **NEVER hardcode colors** — use constants from `src/ui/theme.rs`
3. **NEVER hardcode font sizes** — use `FONT_H1` through `FONT_CAPTION`
4. **NEVER go below 12px** font size — accessibility violation
5. **NEVER hardcode spacing** — use `SPACING_SECTION` through `SPACING_MICRO`
6. **ALWAYS use component helpers** from `src/ui/components.rs` when available
7. **ALWAYS verify contrast** for new color combinations (≥ 4.5:1 for body text)
8. **NEVER change business logic** — UI code only renders, does not compute
9. **Keep 60fps** during recording — spectrum animation must stay smooth
10. **NEVER introduce regressions** — existing functionality must work identically

## Decision Gates

### When adding/changing UI:

| Scenario | Action |
|----------|--------|
| Need a button with color | `accent_button(ui, label, ACCENT_*, ACCENT_*_HOVER, size)` |
| Need a card section | `card_frame(ui).show(ui, \|ui\| { ... })` |
| Need a section title | `section_header(ui, icon, title, Some(description))` |
| Need status feedback | `status_badge(ui, text, ACCENT_*)` |
| Need progress indication | `progress_bar(ui, pct, ACCENT_*, "label")` |
| Need to separate sections | `divider(ui)` |
| Need a field label | `field_label(ui, "Label", Some("description"))` |
| Painting the spectrum | `paint_spectrum_bars()` from `src/ui/spectrum.rs` |
| Choosing text style | `body(text)`, `caption(text)`, `h1(text)`, etc. from theme |
| Adding a new tab | Match existing tab pattern, use `FONT_H1` for title |
| Need a new color | Add to BOTH `.agents/rules/design-system.md` AND `src/ui/theme.rs` |

### When a helper doesn't exist:

1. Check if you can compose existing helpers
2. If truly new, create the helper in `src/ui/components.rs`
3. Document it in `.agents/rules/design-system.md` Rule 5
4. Use ONLY theme constants, never raw values

## Mandatory Pre-Work Checklist

Before writing ANY UI code:

```
□ Read .agents/rules/design-system.md
□ Identify which existing components/helpers to reuse
□ Verify colors needed are in the palette
□ Verify font sizes needed are in the scale
□ Plan spacing using SPACING_* tokens
```

## Mandatory Post-Work Checklist

After writing UI code:

```
□ No hardcoded Color32::from_rgb() values
□ No font sizes below 12px
□ No hardcoded ui.add_space() values
□ Used component helpers where available
□ Contrast verified for any new color pairs
□ All UI states handled (normal, hover, active, disabled)
□ cargo fmt --check passes
□ cargo clippy --all-targets passes
□ Ran the app and visually verified
```

## Implementation Order for Initial Redesign

When implementing the full Nebula Dark redesign, follow this sequence:

1. **Theme migration** — Copy `assets/theme.rs` → `src/ui/theme.rs`, replace old palette
2. **Component extraction** — Copy `assets/components.rs` → `src/ui/components.rs`
3. **Spectrum extraction** — Copy `assets/spectrum.rs` → `src/ui/spectrum.rs`
4. **Module split** — Break `src/ui.rs` into `src/ui/{app,recording,settings,about}.rs`
5. **Typography** — Update all `RichText::new(...).size(N)` to theme constants
6. **Spacing** — Replace all `ui.add_space(N)` with constants
7. **Tab bar** — Redesign with theme colors
8. **Recording tab** — Action button, status bar, spectrum, transcript, history
9. **Settings tab** — Sidebar badges, section cards, tooltips
10. **About tab** — Header, features, tech stack, author
11. **Apply theme** — Call `apply_nebula_theme(ctx)` in `App::update()`
12. **Verify** — Full visual test of all states and tabs

## Component Reference

- `assets/theme.rs` — Color palette, typography, spacing, gradients (→ `src/ui/theme.rs`)
- `assets/components.rs` — Reusable UI widget helpers (→ `src/ui/components.rs`)
- `assets/spectrum.rs` — Audio visualizer (→ `src/ui/spectrum.rs`)

## References

- `.agents/rules/design-system.md` — **MANDATORY READ** — All design rules
- `.agents/skills/ui-redesign-scrivano/references/design-spec.md` — Visual design specification
- `PRPs/prp-ui-redesign.md` — Original redesign requirements
