# Backpack Battles TUI Design System

## 0. Research Log

- Embedded refs: minimalist operational UI + Raycast dark developer-tool chrome; selected for dense, keyboard-first terminal use.
- Ratatui: official 0.30.2 layout, gauge, paragraph, and terminal lifecycle documentation.
- Skipped lazyweb and image generation: browser-product screenshots and raster concepts do not translate reliably to terminal cells.

## 1. Atmosphere & Identity

A compact battle instrument: dark, precise, and readable at a glance. The signature is persistent colored item shapes whose vacated cells become dim red `×` marks, making falls visible without mixing state into the journal.

## 2. Color

| Role | RGB | Usage |
|---|---|---|
| Canvas | `7, 8, 10` | Terminal background |
| Surface | `16, 17, 17` | Panels |
| Border | `47, 48, 49` | Quiet containment |
| Text | `249, 249, 249` | Primary content |
| Muted | `156, 156, 157` | Help and inactive state |
| Left / info | `85, 179, 255` | Left hero and informational events |
| Right / danger | `255, 99, 99` | Right hero, damage, and fallen cells |
| Success | `95, 201, 146` | Healing and healthy HP |
| Warning | `255, 188, 51` | Low HP, speed, prevention |
| Special | `195, 125, 255` | Item falls |

Color is semantic. Item colors identify ownership within a bag; status colors describe event meaning.

## 3. Typography

The terminal's monospace font is the only typeface. Bold marks titles and live values; dim text marks help, inactive state, and fallen inventory.

## 4. Spacing & Layout

- Base unit: one terminal cell.
- Wide terminals: stacked hero panels on the left, journal on the right.
- Narrow terminals: hero panels share the top row, journal fills the bottom.
- Minimum usable viewport: 72 columns by 20 rows; smaller terminals receive a resize prompt.

## 5. Components

### Hero panel
- Bordered surface with name, HP gauge, block, bag grid, and compact two-column legend.
- The side color owns the border; health color owns only the gauge.
- Fallen cells and legend entries remain visible as dim red `×` marks.

### Fight journal
- Independent bordered pane with automatic follow while the battle runs.
- Arrow and page keys disable follow; `End` restores it.
- Events use one semantic accent and keep tick/HP context in a muted header.

### Status bars
- Header shows seed, tick, speed, and run state.
- Footer lists keyboard controls without decoration.

## 6. Motion & Interaction

Battle ticks are the only animation. `Space` pauses, `+`/`-` changes speed, arrows and page keys scroll, `Home`/`End` jump, and `q` exits. No decorative motion.

## 7. Depth & Surface

Borders-only. Near-black canvas, slightly lighter panels, quiet gray borders, and brighter focused information create hierarchy without gradients or shadows.

## 8. Accessibility Constraints & Accepted Debt

- Every status has text or shape in addition to color.
- Full keyboard operation; controls remain visible.
- Layout adapts to terminal size and never relies on pointer input.
- Accepted debt: terminal color fidelity depends on the user's emulator; all content remains legible with reduced color support.
