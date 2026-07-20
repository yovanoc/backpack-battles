# Backpack Battles Live Viewer Design System

## 0. Research Log

- Static reference: Kenney Tiny Dungeon `Preview.png` and 12×11 tile sheet; extracted hard 16px pixel geometry, warm timber, cool stone, cream highlights, and compact RPG-object silhouettes.
- PixiJS: verified current v8 `Application.init`, `Assets.load`, `Graphics`, and shared-context guidance from official documentation.
- Image generation skipped: the user supplied the licensed Kenney reference and asked to build with those assets; a generated replacement would weaken reference fidelity.
- Lazyweb skipped: this is a tool surface, not a clone; the supplied game-art reference defines the visual language.

## 1. Atmosphere & Identity

A compact dungeon workbench where battle simulation feels like operating a physical pixel-board. The signature is the **split arena table**: two stone-framed bags face across a narrow combat lane, with warm wood controls and five unmistakable archetype inks. It must feel tactical and handmade, never glossy, glassy, or visually similar to Backpack Battles.

## 2. Color

| Role | Token | Value | Usage |
|---|---|---:|---|
| Canvas | `--surface-canvas` | `#15131b` | Page background |
| Recessed | `--surface-recessed` | `#201a24` | Wells and stage surround |
| Panel | `--surface-panel` | `#30242b` | Primary panels |
| Wood | `--surface-wood` | `#754231` | Controls and headers |
| Wood lit | `--surface-wood-lit` | `#985034` | Hovered controls |
| Stone | `--surface-stone` | `#4c5263` | Bag frames |
| Stone lit | `--surface-stone-lit` | `#697186` | Raised stone edges |
| Ink | `--border-ink` | `#0d0c11` | Hard outlines |
| Text | `--text-primary` | `#fff0c2` | Primary copy |
| Muted | `--text-muted` | `#e0d2aa` | Secondary copy |
| Aggression | `--archetype-aggression` | `#ef604f` | Damage items |
| Defense | `--archetype-defense` | `#68a9d4` | Defensive items |
| Scaling | `--archetype-scaling` | `#55c985` | Scaling items |
| Control | `--archetype-control` | `#aa78d4` | Control items |
| Support | `--archetype-support` | `#e5b84e` | Support items |
| Health | `--status-health` | `#d9544d` | HP bars |
| Block | `--status-block` | `#5c9fd1` | Block bars |
| Poison | `--status-poison` | `#63c75d` | Poison feedback |
| Focus | `--status-focus` | `#fff1a8` | Keyboard focus |

No colors outside this table. Archetype color communicates category; it is not decorative.

## 3. Typography

| Level | Size | Weight | Line height | Usage |
|---|---:|---:|---:|---|
| Display | `clamp(1.5rem, 3vw, 2.25rem)` | 800 | 1.1 | Product title |
| H2 | `1.125rem` | 800 | 1.25 | Panel title |
| H3 | `1rem` | 700 | 1.3 | Item title |
| Body | `0.9375rem` | 500 | 1.5 | Default copy |
| Small | `0.8125rem` | 600 | 1.4 | Controls and stats |
| Caption | `0.75rem` | 700 | 1.3 | Labels |

- Primary and mono: `ui-monospace, "SFMono-Regular", Consolas, "Liberation Mono", monospace`.
- Text uses normal browser rasterization; pixel identity comes from geometry and assets, not illegibly tiny bitmap fonts.

## 4. Spacing & Layout

Base unit: 4px. Tokens: `--space-1: 4px`, `--space-2: 8px`, `--space-3: 12px`, `--space-4: 16px`, `--space-5: 20px`, `--space-6: 24px`, `--space-8: 32px`.

- Shell maximum: 1800px; gutters use `clamp(12px, 2vw, 28px)`.
- The main page is battle-only: setup, arena, telemetry, summary, and playback fill the first viewport. Catalog, stats, and both editable Bags never compete in normal page flow.
- Desktop: arena spans the full shell width; canvas and the large live combat rail divide it evenly. The canvas is vertically centered beside the 520–580px rail; Effects and log each receive half its height.
- Under 1300px: canvas takes full width first; effects and log become two side-by-side panes at least 380px tall below it.
- Under 700px: one column; setup controls collapse to two rows; the Pixi arena stacks fighters vertically and telemetry panes stack below it.
- Pixi canvas owns battle visualization only. Editing, controls, stats, and status remain semantic DOM.

## 5. Components

### Pixel Panel
- Structure: semantic section with heading and body.
- Variants: panel, recessed, stone.
- States: default and focused-within.
- Accessibility: labelled by heading.
- Motion: none.

### Pixel Button
- Structure: native `button` with optional icon span and label.
- Variants: wood primary, stone secondary, danger.
- States: default, hover, active, focus-visible, disabled.
- Accessibility: native keyboard behavior and visible 3px focus outline.
- Motion: 120ms transform/opacity only; active translates 2px.

### Field
- Structure: `label` plus native input/select.
- States: default, hover, focus, invalid, disabled.
- Accessibility: explicit labels; numeric bounds retained.

### Item Palette
- Structure: search, archetype filters, list/grid view toggle, scrollable item buttons.
- States: selected item, selected archetype, list, grid, empty search.
- Accessibility: selection announced with `aria-pressed`; weapon and footprint included in accessible label.

### Bag Editor
- Structure: labelled 5×4 CSS grid of native cell buttons.
- States: empty, valid target, blocked target, occupied.
- Constraint: a Bag holds at most two copies of any Item kind.
- Accessibility: row/column announced per cell; placed items removable with Enter/Space; instructions are persistent.
- Motion: placement pop 180ms; removal fade 120ms; disabled under reduced motion.

### Loadout Workshop Dialog
- Structure: native modal `dialog` with catalog, two-Bag editor, and selected-item inspector in one self-contained workspace.
- States: closed, open, filtered, selected item, invalid placement, ready to fight.
- Accessibility: `showModal()` provides top-layer modality and Escape handling; the visible Close control receives initial focus; closing restores focus to the opener.
- Responsive: three columns on wide screens, Bag editor first with catalog/inspector alongside; one scrollable column on mobile.
- Motion: backdrop and shell enter over 240ms; no decorative movement inside the editor.

### Battle Stage
- Structure: large Pixi canvas with a DOM combat rail containing hero effects, item charge/speed, and fight log.
- States: ready, playing, paused, complete.
- Accessibility: canvas is presentational; live DOM summary reports tick, health, block, and latest event.
- Motion: activation pulse, source-to-target tracer, target shock plate, heal glow, backed damage/heal labels, cause-specific fall exit, and poison tint; all encode engine events.

### Combat Rail
- Structure: two hero status sections followed by a scrollable ordered fight log.
- States: ready, active effects, no events, complete.
- Accessibility: charge uses native progress plus text; log is an ordered list; current tick summary remains live.
- Motion: none; the rail is the stable textual counterpart to the animated canvas.

### Quick Add
- Structure: two inspector buttons, Add left and Add right.
- States: available, no-space error.
- Accessibility: result is announced through the existing status region.
- Motion: newly placed occupied cells use the Bag Editor entry motion.

### Timeline
- Structure: play/pause button, range input, tick text, speed select.
- States: start, playing, paused, complete.
- Accessibility: range has tick label and keyboard control.

### Stat Inspector
- Structure: selected item title, effect, footprint, stat definition list.
- States: item selected and empty prompt.
- Accessibility: weapon status is text, never color alone.

## 6. Motion & Interaction

| Token | Duration | Easing | Usage |
|---|---:|---|---|
| `--motion-micro` | 120ms | `ease-out` | Press/focus feedback |
| `--motion-standard` | 240ms | `ease-in-out` | Selection and placement |
| `--motion-emphasis` | 420ms | `cubic-bezier(.16,1,.3,1)` | Battle effects |
| `--motion-cinematic` | 720ms | `cubic-bezier(.16,1,.3,1)` | Source-to-target consequences and exits |

- Pixi animations change `position`, `scale`, `alpha`, or tint only.
- Activation pulse identifies the source item; floating values identify consequence and target.
- `prefers-reduced-motion` removes pulses/floats and applies final states immediately.
- Playback defaults paused so the user chooses when motion begins.

## 7. Depth & Surface

Mixed pixel depth: every raised surface has a 3px dark outline plus a hard 4px offset shadow; recessed wells use an inset dark edge. No blur, glass, soft shadow, gradient glow, or rounded-pill styling. Corner radius is 0 or 4px only.

## 8. Accessibility Constraints & Accepted Debt

### Constraints

- WCAG 2.2 AA: 4.5:1 body contrast, 3:1 large text and UI boundaries.
- Every action is reachable by keyboard with persistent focus-visible treatment.
- Canvas never owns required text or editing behavior.
- Status uses text plus color; reduced-motion is respected.
- Primary content reflows at 375px without horizontal overflow.

### Accepted Debt

| Item | Location | Why accepted | Owner / Exit |
|---|---|---|---|
| Kenney sprites are decorative approximations for some authored item names | Battle canvas and palette | Tiny Dungeon has fewer unique object sprites than the 52-item catalog | Replace only when a dedicated sprite sheet exists |
