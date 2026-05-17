# Scrivano Design System — MANDATORY Rules

> **ENFORCEMENT LEVEL: ABSOLUTE**
> These rules apply to ALL UI changes in any file under `src/ui/`.
> No agent, subagent, or human may override these without explicit approval
> documented in a PRP. Violations are bugs, not style preferences.

---

## Rule 1: Color Palette is IMMUTABLE

### Background Hierarchy (deepest → surface)

```
BG_VOID           #090A0F   Root window background
BG_NEBULA         #131620   Panels, sidebar
BG_STARDUST       #1A1F2E   Cards, content surfaces
BG_STARDUST_HOVER #202637   Card/row hover state
BG_ECLIPSE        #2A3045   Borders, separators, strokes
```

### Accent Colors (semantic roles)

```
ACCENT_CYAN          #00D4FF   Primary: actions, selection, focus
ACCENT_CYAN_HOVER    #33DEFF   Primary hover
ACCENT_CYAN_DIM      #00A0C8   Primary pressed/disabled
ACCENT_PURPLE        #A855F7   Secondary: AI features, magic, highlights
ACCENT_PURPLE_HOVER  #BC73FA   Secondary hover
ACCENT_PURPLE_DIM    #823CD2   Secondary pressed
ACCENT_EMERALD       #10B981   Success: recording active, confirmed
ACCENT_EMERALD_HOVER #34D399   Success hover
ACCENT_AMBER         #F59E0B   Warning: pending, attention
ACCENT_CRIMSON       #EF4444   Danger: stop, delete, error
ACCENT_CRIMSON_HOVER #F86464   Danger hover
```

### Text Colors (verified WCAG AA)

```
TEXT_STARLIGHT  #F1F5F9   Body text, headings (contrast ≥ 14:1)
TEXT_MOON       #94A3B8   Secondary labels (contrast ≥ 5.5:1)
TEXT_DUST       #64748B   Hints, placeholders (contrast ≥ 4.6:1)
TEXT_WHITE      #FFFFFF   Text on accent button fills only
```

### FORBIDDEN

- Hardcoding `Color32::from_rgb(...)` values directly in UI code
- Using colors outside this palette without documented justification
- Changing semantic meaning of any color (e.g., cyan for danger, crimson for success)
- Using different shades for the same semantic purpose across components

If a new semantic state needs a color, add it to the palette WITH justification.

---

## Rule 2: Typography Scale is IMMUTABLE

| Level | Size | Weight | Purpose |
|-------|------|--------|---------|
| H1    | 28px | Bold   | Tab titles, section headers |
| H2    | 20px | Bold   | Card titles, subsection headers |
| H3    | 16px | Medium | Field labels, group headers |
| Body  | 14px | Regular| General content, descriptions |
| Caption| 12px | Regular| Auxiliary, timestamps, hints |

### FORBIDDEN

- Text below 12px — **ABSOLUTELY NEVER**. This is an accessibility violation.
- More than 3 font sizes visible on screen simultaneously
- Using `Strong` on Body or Caption to simulate a heading level
- Skipping hierarchy levels (e.g., Body text directly under H1 without H2/H3)

### REQUIRED

- Every visible text element must use EXACTLY ONE of the 5 levels above
- Font sizes are defined as constants `FONT_H1`, `FONT_H2`, etc. in `src/ui/theme.rs`
- Use `RichText::new(text).size(FONT_*)` — never raw floats

---

## Rule 3: Spacing System is IMMUTABLE

| Token | Value | Usage |
|-------|-------|-------|
| `SPACING_SECTION` | 24px | Between major content sections |
| `SPACING_CARD_PAD` | 20px | Internal card padding (horizontal) |
| `SPACING_ELEMENT` | 12px | Between related UI elements |
| `SPACING_TIGHT` | 8px | Between compact/inline elements |
| `SPACING_MICRO` | 4px | Between icon+text, tiny gaps |

### REQUIRED

- Use spacing constants. Never hardcode `ui.add_space(7.0)` or similar.
- Gaps between unrelated sections: `SPACING_SECTION`
- Padding inside cards: `SPACING_CARD_PAD` (horizontal), 4px less for vertical
- Gaps between siblings: `SPACING_ELEMENT`
- Gaps between inline items: `SPACING_TIGHT`

---

## Rule 4: Border Radii

```
ROUNDING_CARD    12px   Content cards, frames
ROUNDING_BUTTON  10px   Action buttons
ROUNDING_SMALL    6px   Badges, inputs, small items
ROUNDING_PILL    20px   Status pills, tags
```

### REQUIRED

- All cards and grouped frames use `ROUNDING_CARD`
- Primary action buttons use `ROUNDING_BUTTON`
- Never use `0.0` rounding on interactive elements

---

## Rule 5: Component Patterns are MANDATORY

### When creating or modifying UI, use the helper functions from `src/ui/components.rs`:

| Need | Use |
|------|-----|
| Card container | `card_frame(ui)` |
| Section title | `section_header(ui, icon, title, subtitle)` |
| Accent button | `accent_button(ui, label, color, hover, size)` |
| Status pill | `status_badge(ui, text, color)` |
| Progress bar | `progress_bar(ui, pct, color, label)` |
| Divider | `divider(ui)` |
| Tech label | `tech_badge(ui, label, color)` |
| Apply theme | `apply_nebula_theme(ctx)` in `update()` |

### FORBIDDEN

- Copy-pasting card layout code. Use `card_frame()`.
- Manual progress bar painting. Use `progress_bar()`.
- Custom color mixing for status. Use `status_badge()`.

---

## Rule 6: Contrast — WCAG AA MANDATORY

### Verified pairs (all others must be verified before use)

| Foreground | Background | Ratio | Status |
|------------|-----------|-------|--------|
| TEXT_STARLIGHT | BG_VOID | 18.5:1 | AAA ✓ |
| TEXT_STARLIGHT | BG_STARDUST | 14.2:1 | AAA ✓ |
| TEXT_MOON | BG_VOID | 7.2:1 | AA ✓ |
| TEXT_MOON | BG_STARDUST | 5.5:1 | AA ✓ |
| TEXT_DUST | BG_VOID | 4.6:1 | AA ✓ |
| TEXT_WHITE | ACCENT_CYAN | 1.5:1 | ⚠ Large bold only |
| TEXT_WHITE | ACCENT_CRIMSON | 3.8:1 | ⚠ Large bold only |

### RULES

- Body text (14px) must have contrast ≥ 4.5:1
- Large text (≥ 18px bold) must have contrast ≥ 3:1
- TEXT_WHITE on accent backgrounds: only allowed on buttons with H3 (16px bold) or larger
- TEXT_DUST on BG_STARDUST: ≥ 3.7:1 — only for large text, NOT for body text

---

## Rule 7: Spectrum Visualizer — Do Not Degrade

The audio spectrum is a core identity element. Changes must preserve:

- 48 vertical bars, mirrored above/below centerline
- 5-stop gradient: cyan → blue → purple → magenta → rose
- Smooth animation: lerp factor 0.35 at 60fps, peak hold decay 0.92
- Max bar height: 35px per side
- Center reference line (subtle, BG_ECLIPSE)
- Rounded bar tops using `ROUNDING_SMALL`

---

## Rule 8: Layout Constraints

| Constraint | Value |
|------------|-------|
| Settings content max width | 760px |
| Settings field max width | 420px |
| Settings sidebar width | 172px |
| Main action button size | 320×56px |
| Transcript max height (collapsed) | 150px |
| Spectrum height | 90px |

---

## Rule 9: Structure Rules

### Module organization (REQUIRED)

```
src/ui/
├── mod.rs          # Re-exports, module declarations
├── theme.rs        # Color palette, typography, spacing constants
├── components.rs   # Reusable widget helpers
├── spectrum.rs     # Audio visualizer
├── recording.rs    # Recording tab
├── settings.rs     # Settings tab
├── about.rs        # About tab
└── app.rs          # App struct, eframe::App impl, tab routing
```

### FORBIDDEN

- Single monolithic `ui.rs` file. UI code MUST be split into modules.
- Mixing business logic with UI rendering. UI functions only consume data, never compute it.
- Hardcoding layout values. Use constants from `theme.rs`.

---

## Rule 10: Change Protocol

Before ANY UI change:

1. **Read** `.agents/rules/design-system.md` (this file)
2. **Verify** the change doesn't violate any rule above
3. **Use** existing component helpers — do not write raw painting unless no helper exists
4. **Check** contrast if adding new color combinations
5. **Run** `cargo fmt --check && cargo clippy --all-targets`
6. **Test** visually: run the app and verify all states (normal, hover, active, disabled)

If you need to ADD a color, spacing, or component pattern not covered by these rules:
1. Propose it in the change description
2. Add it to this rules file
3. Add the constant to `src/ui/theme.rs`
4. Document the rationale

---

## Related Files

- `src/ui/theme.rs` — Implementation of Rule 1-4 constants
- `src/ui/components.rs` — Implementation of Rule 5 helpers
- `src/ui/spectrum.rs` — Implementation of Rule 7 visualizer
- `.agents/skills/ui-redesign-scrivano/SKILL.md` — Skill for UI development workflows
- `PRPs/prp-ui-redesign.md` — Original redesign specification
- `.agents/skills/ui-redesign-scrivano/references/design-spec.md` — Visual design reference
