# Accu-Chek Tactile Swiss Design System v2

> Canonical implementation direction: a Swiss laboratory instrument designed by someone who loves synthesizers, scientific field notebooks, industrial control panels, and Indian decorative craft.

## 1. Design proposition

Accu-Chek Local is a **personal glucose data instrument**. Its interface combines:

- Swiss editorial discipline: strict grids, clear hierarchy, compact labels, and precise alignment.
- Hardware-grade tactility: layered construction, inset displays, raised controls, and visible pressed states.
- Personal colour: a warm, lively palette used to identify functions and make the product feel owned rather than institutional.
- Analytical texture: fine rules, measurement grids, halftones, and restrained print-like patterns.

The interface should feel less like a dashboard styled as hardware and more like an actual personal analytical machine with a visual personality. It should be engineered enough for serious analysis and personal enough that someone would enjoy leaving it open on their desktop.

Its four visual traditions are:

- **Precision instrumentation:** controls have travel, displays sit in wells, state is visible before interaction, and every element appears mechanically placed.
- **Swiss information design:** alignment, hierarchy, typography, and density remain rigorous regardless of decoration.
- **Technical terminal culture:** monospaced metadata, diagnostic rails, lamps, grids, ticks, event logs, hashes, and machine-readable details create intelligence rather than a superficial tech aesthetic.
- **Crafted individuality:** selective ornament, print texture, jewel-like accents, and subtle handmade irregularity keep the machine from feeling sterile.

The concise implementation formula is **80% Swiss precision + 10% laboratory and synthesizer hardware + 5% technical printmaking + 5% crafted ornament**.

Large human-readable conclusions sit on top; instrumentation supports them underneath; raw evidence is always one action away.

## 1.1 Three-layer construction

Every screen deliberately combines three layers:

| Layer | Purpose | Includes | Answers |
|---|---|---|---|
| Mechanical | Physical chassis | Panels, rails, wells, slots, buttons, lamps, tracks, bezels | What can I operate? |
| Analytical | Information machine | Graphs, readouts, grids, thresholds, timestamps, metadata | What is happening? |
| Expressive | Personal character | Colour blocks, halftones, print texture, line ornament | Why does this object feel like mine? |

Expressive elements may decorate mechanical structure but must never compete with analytical content.

## 2. Governing principles

### Instrument first

Every surface must look intentionally constructed. Controls should appear operable; displays should appear embedded; related information should share a physical panel.

### Grid before decoration

Colour, texture, and depth sit on top of a rigorous grid. Alignment and hierarchy must remain obvious even when the interface is visually rich.

### Colour has a job

Colour identifies functional areas, selection, device state, and glucose context. It is never scattered randomly and never replaces written meaning.

### Dense but calm

The app may expose advanced analysis, but the first reading order is always: current state, important context, trend, then detail.

### Personal, not childish

Bright colour and playful hardware cues create warmth. Proportions, typography, and restraint keep the result credible and precise.

### Source-faithful data

Visual summaries never obscure the underlying measurements. Raw readings, provenance, annotations, and meter metadata remain inspectable and exportable.

## 3. Visual grammar

### Geometry

- Primary panels: square corners or `2 px` corner radius.
- Small controls: `2-3 px` maximum radius.
- Circular geometry is reserved for indicator lamps, rotary controls, chart points, and radio selectors.
- No pill-shaped buttons, pill navigation, floating rounded cards, glassmorphism, or soft blob containers.
- Modules meet on a shared grid and should resemble fitted parts of one chassis.

### Grid

- Base spacing unit: `4 px`.
- Main spacing steps: `4, 8, 12, 16, 24, 32, 48 px`.
- Default desktop content grid: 12 columns.
- Major panels align to common vertical and horizontal rails.
- Dense internal readouts use 4- or 8-column subgrids.
- Use fine rules and changes in surface level before adding more empty space.

### Depth model

Depth communicates construction and interaction, not luxury.

| Level | Role | Treatment |
|---|---|---|
| D0 Chassis | App background | Matte colour or subtle grain; no shadow |
| D1 Faceplate | Navigation and major sections | 1 px hard edge, short lower shadow |
| D2 Module | Metric blocks and tool groups | Raised 1-2 px with top highlight and lower edge |
| D-1 Well | Charts, tables, inputs, displays | Inset top shadow and darker inner edge |
| D3 Control | Primary physical action | Raised keycap; clear side wall and drop shadow |
| Pressed | Active control | Translate down 1-2 px; shadow contracts; surface darkens slightly |

Shadows are tight and directional. Avoid large blurred floating-card shadows. Highlights and lower edges should imply a consistent light source from the upper left.

### Physical construction vocabulary

Every interactive or analytical element must be assembled from the shared construction vocabulary rather than drawn as an arbitrary rectangle.

| Token | Meaning |
|---|---|
| `plate` | Mounted surface |
| `well` | Recessed display |
| `slot` | Control track or mode rail |
| `key` | Pressable action |
| `lamp` | State indicator |
| `bezel` | Edge framing a display |
| `track` | Progress or selectable continuum |
| `index` | Tick, graduation, number, or position marker |

A slider is a track with indices and a movable key. A segmented control is a slot containing persistent mechanical states. A chart is a calibrated well with a bezel. A sync control is a key with an adjacent lamp.

### Instrument detailing

Use tiny, functional-looking details to reward close inspection:

- Calibration ticks and scale graduations.
- Channel numbers such as `A1`, `A2`, and `R03`.
- Stamped section identifiers and field indices.
- Serial-style import and export batch IDs.
- Engraved arrows and datum marks.
- Coordinate labels such as `RANGE/07`, `SYNC/USB`, and `VIEW/03`.
- Mounting dots or screw-head motifs only at genuine panel boundaries.

These details should recede at normal viewing distance and become delightful when inspected.

### Controlled imperfection

Geometry, typography, numbers, hit areas, and charts stay exact. Surface treatment may feel subtly screen printed:

- Non-uniform paper grain.
- Slightly varied halftone dots.
- Minute irregularity in hatching.
- Less than 1 px print-registration variation on selected decorative rules.
- Extremely subtle tonal variation inside large colour fields.

The target is a screen-printed instrument enclosure, never a distressed vintage interface.

### Texture

- Use subtle paper grain on light chassis surfaces and fine powder-coat grain on dark surfaces.
- Halftone, hatch, dot-matrix, and measurement-grid patterns belong inside analytical displays or large quiet fields.
- Texture opacity stays low enough that values and chart marks remain dominant.
- Do not apply texture independently to every card.

## 4. Colour system

The default composition uses warm paper and carbon together. Each screen may feature two or three functional colours plus semantic glucose colours.

### Material foundations

| Token | Value | Use |
|---|---:|---|
| `paper` | `#FCFCFB` | Technical drafting paper; reports, ledgers, explanations |
| `cream` | `#F5F1E7` | Warm paper support surface |
| `carbon` | `#333333` | Powder-coated metal; chassis and diagnostic zones |
| `night` | `#111111` | Deep instrument wells and high-emphasis ink |
| `espresso` | `#473733` | Warm dark supporting material |
| `rule-light` | `#D8D4CA` | Fine rules on light surfaces |
| `rule-dark` | `#494949` | Fine rules on dark surfaces |

### Functional colours

| Family | Base | Deep accent | Primary assignment |
|---|---:|---:|---|
| Butter material | `#FFE791` | `#FFC45C` | Import and device connection |
| Tangerine enamel | `#FFA53A` | `#FF8844` | Physical actions, active indices, current position |
| Raspberry lacquer | `#FFCBF8` | `#FF98CF` | Notes, tags, and user-supplied context |
| Lavender plastic | `#B5B5FE` | `#56547F` | Pattern analysis and statistical comparison |
| Lime phosphor | `#CFEC6F` | `#6E762D` | Illuminated outputs, progress, and healthy state |
| Cyan glass | `#AEF3F3` | `#68B6E8` | Recent data, selected readings, timestamps, active channels |

### Glucose semantics

Glucose status colours remain stable across every screen and export preview:

- Very low: deep raspberry/red.
- Low: coral.
- In sampled range: fresh green/lime.
- High: orange.
- Very high: deep tangerine/red-orange.

Always pair status colour with a written label and value. Functional module colour must yield to glucose semantic colour when both would compete.

### Jewel moments

Most screens use carbon and paper with one concentrated, intensely coloured analytical object: a sampled-range ring, reading-bead field, selected calendar cell, multichannel marker, or import meter. These moments should feel like enamel, stained glass, or illuminated instrument lamps mounted into a restrained machine. Do not colour every panel.

## 5. Three-voice typography

Use local or system fonts only.

- **Human voice:** large system sans for the latest reading, concise conclusions, and period summaries.
- **Machine voice:** local monospace for timestamps, units, sample counts, device IDs, percentiles, hashes, status values, and calibration labels.
- **Editorial voice:** restrained local serif for report titles, selected major section titles, date dividers, and paper-like explanatory moments.

Do not use monospace for ordinary prose. Use local or system fonts only.

### Type scale

| Role | Size | Treatment |
|---|---:|---|
| Hero value | 48-64 px | Sans, regular, tight leading |
| Screen title | 28-34 px | Serif or sans, regular |
| Module value | 24-32 px | Sans, regular |
| Section title | 16-18 px | Sans, medium |
| Body | 14-15 px | Sans, regular |
| Instrument label | 10-12 px | Monospace, uppercase, tracked |
| Micro metadata | 10-11 px | Monospace, muted |

Units never compete with the numeric value. Labels sit above values or align to a dedicated metadata column; avoid colon-heavy label/value prose.

## 6. Core components

### Faceplate

A major section mounted on the chassis. It owns a title rail and one coherent function. Faceplates may contain modules and wells but should not be nested repeatedly.

### Instrument module

A rectangular, raised block for one metric or closely related group. It contains:

1. Monospaced label.
2. Primary value.
3. Unit or comparison.
4. Optional small status indicator.

### Display well

An inset region for charts, tables, timelines, and device readouts. It uses a darker inner edge, minimal radius, and may carry a fine grid or halftone texture.

### Keycap button

- Primary: strong functional colour, raised D3 treatment.
- Secondary: neutral raised surface with coloured edge or label.
- Destructive/high-risk: semantic colour and explicit text.
- Pressed: 1-2 px downward motion, shortened shadow, lower brightness.
- Loading: label remains stable while an adjacent lamp or progress strip animates.

Labels are compact and direct: `SYNC`, `EXPORT PDF`, `EXPORT XLSX`, `SAVE NOTE`.

### Segmented switch

Rectangular adjoining choices used for navigation, chart modes, units, and date ranges. The selected segment appears physically depressed or illuminated. Avoid independent rounded chips.

### Indicator lamp

A small circular light with an adjacent text label. Used for device connected, sync in progress, saved, exported, or error states. Lamps supplement rather than replace text.

### Data ledger

Readings appear as structured columns, not concatenated strings. The ledger supports a fixed header, row selection, scanning by date/value/status, and visible note or tag markers. Selected rows appear inset or mechanically latched.

### Status strip

Persistent system feedback sits in a narrow chassis rail. Errors remain until dismissed or resolved. Success messages may collapse into a timestamped status entry instead of disappearing abruptly.

### Reading beads

Reading beads are the signature Accu-Chek Local visualization: small physical cells where every bead represents an actual measurement. Use them for sampled-range composition, daily density, import counts, weekly activity, time-of-day distributions, and sample-count context. Beads must always encode real data.

### Evidence rail

Every human-readable analytical conclusion mounts a compact mono evidence strip directly beneath it. Examples:

`7 DAYS   N=23   MEDIAN 128   IQR 31   RANGE 82-211`

`CV 28.6%   PRIOR 21.2%   DELTA +7.4`

### Inspection probe

Replace generic floating tooltips with an instrument probe attached to the selected mark by a thin line. A probe may show timestamp, value, meal context, device, and source identity. It uses square geometry and remains visually connected to the measured point.

### Persistent mechanical state

Selected date ranges, modes, units, and ledger rows remain spatially and visually latched until replaced. State should look depressed, indexed, illuminated, or seated in a slot rather than merely recoloured.

### Progressive technical disclosure

The default surface shows understandable identity and status. A deliberate expansion reveals protocol, VID/PID, meter clock, device key, raw status words, schema version, database size, and integrity detail. Technical depth is available without dominating the primary task.

## 7. Charts and analytical displays

- Charts live inside display wells.
- Use thin grid lines, crisp plots, and compact monospaced axes.
- Prefer direct labels and small legends mounted to the chart frame.
- Halftone fills may differentiate distributions or sampled-range areas.
- Thresholds appear as stable horizontal markers using both line pattern and label.
- Avoid glossy gradients, oversized legends, and generic rainbow palettes.
- Titles answer a question: `HOW READINGS HAVE MOVED`, `WHEN HIGH VALUES APPEAR`, `READING DISTRIBUTION`.
- Use “sampled range” language rather than continuous-monitoring “time in range” language.

Each chart well has four zones:

1. Top rail: trace ID, period, and sample count.
2. Analytical field: mathematically precise plot.
3. Calibration bezel: threshold ticks, limit labels, and datum marks.
4. Lower evidence rail: unit, local meter-time context, and source limits.

Instrument traces use thin lines with occasional chunky physical markers. Thresholds appear calibrated into the bezel rather than drawn as generic floating lines. Hovering moves an inspection probe that snaps to real data.

### Permitted technical-art textures

Only these six texture families may be introduced:

1. Dot matrix for sample density.
2. Fine orthographic grid for charts.
3. Diagonal drafting hatch for inactive or raw-source states.
4. Halftone gradient for distributions and report imagery.
5. Registration dots for import and progress.
6. Engraved line ornament for casing edges.

Agents must not invent unrelated textures.

## 8. Application composition

### Global chassis

- Left: narrow vertical mode selector for Overview, Readings, Analysis, Reports, and Settings.
- Top: device identity and connection rail.
- Main: grid-aligned faceplates that change by mode.
- Bottom: persistent storage, last import, and operation status strip.

The bottom strip is the machine status bus, for example:

`USB  ● CONNECTED    DB  ● HEALTHY    LAST SYNC 22:14:08    +12 NEW / 438 TOTAL    STORE 3.8 MB`

Failures replace the relevant segment with an explicit durable state such as `DRIVER ! WINUSB REQUIRED`. Generic toast spam is prohibited.

### Overview

The visual anchor is a large recessed latest-reading display. Supporting modules show sampled-range composition, average, median, variability, and last import. A broad trend display follows beneath, then a compact recent-reading ledger.

The human summary is short and evidence-based, such as `RECENT READINGS LOOK STEADY`, followed immediately by the period and sample count. It must not provide diagnosis or treatment advice.

The Overview hierarchy is fixed: one oversized latest-reading or recent-pattern conclusion, its evidence rail, sampled-range bead field, four subordinate metrics, one major trace, then the recent ledger. Do not begin with equally weighted metric cards.

### Readings

Use a full-width ledger with a mounted search/filter rail. Selecting a row opens a fixed detail faceplate containing provenance, note editing, and meal-context controls. Notes use the raspberry family; provenance stays neutral and monospaced.

### Analysis

Use a rectangular segmented switch to move among trend, distribution, time-of-day, daily sampled range, time bins, and calendar. Each view contains one primary chart and a small evidence rail with sample count, period, and interpretation limits.

Analysis is a workbench: the mode controls remain fixed while the instrument beneath them changes. The selected mode is mechanically depressed. Technical context stays mounted to the well.

### Import

Treat device sync as operating a physical instrument. Show connection lamp, identified meter, read-only promise, import progress, and a factual summary of new, existing, and total readings. The `SYNC` key uses tangerine. Driver errors remain visible with clear recovery instructions.

Import behaves as a process console with discrete stages:

1. Device found.
2. PHDC session.
3. Reading metadata.
4. Transfer.
5. Verify.
6. Store.

The console reports detected, existing, new, and rejected counts. Ordinary users see plain labels; expanded details may expose protocol events and meter identity.

### Reports and data

PDF and XLSX are equal first-class outputs. The faceplate exposes period, unit, annotations, and included sections, followed by two distinct hardware-style export keys.

The XLSX workbook contains:

- `Raw Readings`: every stored source and provenance field without presentation rounding.
- `Annotations`: notes, tags, and meal context linked to reading identity.
- `Devices`: model, serial identity, stable device key, and import metadata.
- `Summary`: selected period, thresholds, unit, descriptive statistics, and sampled-range distribution.

Workbook presentation includes frozen headers, filters, typed dates and numbers, sensible widths, and Excel tables. Export never mutates or silently normalizes stored source data.

Reports behave like a print studio: a configuration plate for period, unit, included sections, and output keys sits beside a paper-like preview. PDF and XLSX exports receive stamped batch IDs and appear in local export history.

### Panel families

| Family | Character | Primary use |
|---|---|---|
| Measurement | Dark, numeric, precise, minimal | Current values and compact metrics |
| Analytical | Dark or cream field with calibrated grid | Charts and distributions |
| Editorial | Paper faceplate with selective serif | Reports and explanations |
| Operational | Carbon, mono labels, lamps, state rows | Import, storage, and system health |

## 9. Motion and feedback

- Hover response: 80-120 ms.
- Button press: immediate, with 1-2 px travel.
- Panel changes: 140-200 ms, using a short mechanical slide or opacity change.
- Indicator lamps may pulse slowly during active sync.
- Charts reveal without elastic or playful bounce.
- Avoid ambient motion. Nothing should move unless the system or user caused it.
- Segment switches move inward when selected.
- Nearby scale ticks illuminate around active sliders or tracks.
- Import progress advances in discrete reading or packet steps.
- Saving a note flashes a lamp once before settling to `SAVED`.
- Export uses a thin paper-feed line rather than a generic spinner.
- Range changes recalibrate charts with a short mechanical sweep.
- No bounce, elastic easing, confetti, or motion for spectacle.

## 10. Content rules

- Use `readings`, `samples`, and `sampled range`; do not imply continuous monitoring.
- State the selected period and sample count beside every summary.
- Use plain status language: `DEVICE FOUND`, `SYNCING`, `12 NEW READINGS`, `EXPORT SAVED`.
- Avoid medical diagnosis, prescriptions, or congratulatory claims unsupported by the data.
- Reserve uppercase for instrument labels and short operational states, not paragraphs.

## 11. Do and do not

| Do | Do not |
|---|---|
| Build one coherent chassis | Scatter floating cards across a canvas |
| Use boxy fitted modules | Default to large rounded rectangles |
| Create depth with edges and short shadows | Use soft floating shadows or glass blur |
| Assign colours consistently | Make every metric a different arbitrary colour |
| Pair large values with compact metadata | Build dense colon-separated text blocks |
| Make actions feel physically operable | Use flat text links for primary operations |
| Use print texture inside analytical fields | Place noisy texture behind essential text |
| Preserve raw values in exports | Export only formatted summaries |

Also prohibit:

- Generic SaaS card dashboards and floating white cards on grey.
- Neon cyberpunk effects, glowing gradients, and glass blur.
- Excessive monospace or terminal styling for ordinary content.
- Fake knobs where a simpler control communicates better.
- Decorative screws on every panel.
- Graphs or animation without analytical meaning.
- More than three functional accent families in one viewport.
- Rounded badge soup and generic AI-insight cards.
- Hidden source provenance.

## 11.1 Signature motifs

Six motifs make the product recognizably Accu-Chek Local:

1. **Orange index line:** a 2-3 px tangerine cursor marking current selection.
2. **Reading bead field:** real samples represented as small physical cells.
3. **Evidence rail:** mono metadata immediately supporting conclusions.
4. **Calibration bezel:** ticks and labels framing analytical wells.
5. **Status bus:** persistent operational state across the chassis.
6. **Raw/source hatch:** drafting texture reserved for provenance-level information.

## 11.2 Instrument glyphs

Use familiar platform icons where clarity demands them, supplemented by a small glyph family for USB, meter, reading, sampled range, note, export, database, raw source, threshold, clock, and distribution. Glyphs use 1.5-2 px strokes with squared terminals. Important operations combine glyph and mono label; avoid icon-only primary actions.

## 11.3 Ornament boundaries

Craft influence appears at casing edges and transitions: nested display borders, quiet engraved linework, inactive chassis fields, or one special dial. It never sits beneath dense data. The influence is rhythmic, earthy, and jewel-like rather than a literal cultural motif.

## 11.4 Hidden depth

Curious users may reveal protocol identity, raw source fields, hexadecimal status, meter-native representation, export batch history, schema version, database size, and integrity state. These remain one deliberate action deeper than the primary experience.

## 11.5 Two-second rule

Every screen must immediately answer:

1. What am I looking at?
2. What is the most important value or state?
3. What can I do next?

Complexity becomes discoverable only after these answers are clear.

## 11.6 Responsive mechanical configurations

| Configuration | Width | Composition |
|---|---:|---|
| Wide bench | 1440 px and above | Navigation, work area, optional inspection panel |
| Instrument | 1000-1439 px | Navigation rail and work area; details overlay or replace columns |
| Compact console | Below 1000 px | Top mode strip; one- or two-column modules |

Do not merely shrink the 12-column grid. The physical metaphor and boxy fitted construction survive every configuration.

## 12. Acceptance criteria

The design system is correctly applied when:

- Every screen reads as part of one physical desktop instrument.
- Geometry is predominantly rectangular with no generic pill or soft-card language.
- Depth has a consistent construction model and pressed controls have visible travel.
- The palette is colourful but functionally assigned.
- Swiss grid discipline remains obvious at every supported window size.
- Overview, Readings, Analysis, Import, Reports, and Settings use the documented components.
- PDF and XLSX export are equally discoverable.
- XLSX output preserves raw data and produces a properly structured workbook.
- Medical language stays factual and source-faithful.
- No network dependency, remote asset, or cloud behaviour is introduced.
- Every viewport has one obvious primary value or conclusion and one obvious next action.
- Mechanical controls use the documented physical construction vocabulary.
- Charts use calibrated wells, evidence rails, and inspection probes.
- The six signature motifs recur consistently without dominating the content.
- Progressive disclosure keeps technical depth available but out of the primary reading path.

## 12.1 Emotional acceptance

The intended reaction is not “this healthcare dashboard looks nice.” It is:

> This feels like a beautifully engineered little machine for understanding my data.

Continued use should reveal more depth than was visible at first glance. Immediate clarity plus hidden sophistication is the emotional target.

## 13. Implementation constraint

The visual influence is conceptual: precision hardware, playful colour, Swiss editorial order, and tactile interaction. Do not copy the layout, branding, iconography, or distinctive product details of any single reference or manufacturer.
