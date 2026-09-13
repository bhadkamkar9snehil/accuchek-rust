# Accu-Chek Tactile Swiss Design System

## 1. Design proposition

Accu-Chek Local is a **personal glucose data instrument**. Its interface combines:

- Swiss editorial discipline: strict grids, clear hierarchy, compact labels, and precise alignment.
- Hardware-grade tactility: layered construction, inset displays, raised controls, and visible pressed states.
- Personal colour: a warm, lively palette used to identify functions and make the product feel owned rather than institutional.
- Analytical texture: fine rules, measurement grids, halftones, and restrained print-like patterns.

The interface should feel like a beautifully engineered tabletop instrument, not a hospital portal, generic dashboard, or mobile app enlarged for desktop.

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

### Texture

- Use subtle paper grain on light chassis surfaces and fine powder-coat grain on dark surfaces.
- Halftone, hatch, dot-matrix, and measurement-grid patterns belong inside analytical displays or large quiet fields.
- Texture opacity stays low enough that values and chart marks remain dominant.
- Do not apply texture independently to every card.

## 4. Colour system

The default composition uses warm paper and carbon together. Each screen may feature two or three functional colours plus semantic glucose colours.

### Foundations

| Token | Value | Use |
|---|---:|---|
| `paper` | `#FCFCFB` | Main light faceplate |
| `cream` | `#F5F1E7` | Secondary light surface |
| `carbon` | `#333333` | Dark chassis and primary ink |
| `night` | `#111111` | Deep wells and high-emphasis text |
| `espresso` | `#473733` | Warm dark supporting surface |
| `rule-light` | `#D8D4CA` | Fine rules on light surfaces |
| `rule-dark` | `#494949` | Fine rules on dark surfaces |

### Functional colours

| Family | Base | Deep accent | Primary assignment |
|---|---:|---:|---|
| Butter | `#FFE791` | `#FFC45C` | Import and device connection |
| Tangerine | `#FFA53A` | `#FF8844` | Primary actions and attention |
| Raspberry | `#FFCBF8` | `#FF98CF` | Notes and personal annotations |
| Lavender | `#B5B5FE` | `#56547F` | Analysis and patterns |
| Lime | `#CFEC6F` | `#6E762D` | Positive progress and in-range summaries |
| Mint | `#AEF3F3` | `#68B6E8` | Recent readings and data views |

### Glucose semantics

Glucose status colours remain stable across every screen and export preview:

- Very low: deep raspberry/red.
- Low: coral.
- In sampled range: fresh green/lime.
- High: orange.
- Very high: deep tangerine/red-orange.

Always pair status colour with a written label and value. Functional module colour must yield to glucose semantic colour when both would compete.

## 5. Typography

Use local or system fonts only.

- **Display numbers:** a clean system sans in large, light or regular weight. Used for latest glucose, averages, and percentages.
- **Editorial headings:** a restrained local serif where available, with a system serif fallback. Used sparingly for screen titles and report-like moments.
- **Interface text:** system sans for controls, descriptions, and navigation.
- **Instrument labels:** local monospace for timestamps, units, device identity, field labels, and calibration-like markings.

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

## 7. Charts and analytical displays

- Charts live inside display wells.
- Use thin grid lines, crisp plots, and compact monospaced axes.
- Prefer direct labels and small legends mounted to the chart frame.
- Halftone fills may differentiate distributions or sampled-range areas.
- Thresholds appear as stable horizontal markers using both line pattern and label.
- Avoid glossy gradients, oversized legends, and generic rainbow palettes.
- Titles answer a question: `HOW READINGS HAVE MOVED`, `WHEN HIGH VALUES APPEAR`, `READING DISTRIBUTION`.
- Use “sampled range” language rather than continuous-monitoring “time in range” language.

## 8. Application composition

### Global chassis

- Left: narrow vertical mode selector for Overview, Readings, Analysis, Reports, and Settings.
- Top: device identity and connection rail.
- Main: grid-aligned faceplates that change by mode.
- Bottom: persistent storage, last import, and operation status strip.

### Overview

The visual anchor is a large recessed latest-reading display. Supporting modules show sampled-range composition, average, median, variability, and last import. A broad trend display follows beneath, then a compact recent-reading ledger.

The human summary is short and evidence-based, such as `RECENT READINGS LOOK STEADY`, followed immediately by the period and sample count. It must not provide diagnosis or treatment advice.

### Readings

Use a full-width ledger with a mounted search/filter rail. Selecting a row opens a fixed detail faceplate containing provenance, note editing, and meal-context controls. Notes use the raspberry family; provenance stays neutral and monospaced.

### Analysis

Use a rectangular segmented switch to move among trend, distribution, time-of-day, daily sampled range, time bins, and calendar. Each view contains one primary chart and a small evidence rail with sample count, period, and interpretation limits.

### Import

Treat device sync as operating a physical instrument. Show connection lamp, identified meter, read-only promise, import progress, and a factual summary of new, existing, and total readings. The `SYNC` key uses tangerine. Driver errors remain visible with clear recovery instructions.

### Reports and data

PDF and XLSX are equal first-class outputs. The faceplate exposes period, unit, annotations, and included sections, followed by two distinct hardware-style export keys.

The XLSX workbook contains:

- `Raw Readings`: every stored source and provenance field without presentation rounding.
- `Annotations`: notes, tags, and meal context linked to reading identity.
- `Devices`: model, serial identity, stable device key, and import metadata.
- `Summary`: selected period, thresholds, unit, descriptive statistics, and sampled-range distribution.

Workbook presentation includes frozen headers, filters, typed dates and numbers, sensible widths, and Excel tables. Export never mutates or silently normalizes stored source data.

## 9. Motion and feedback

- Hover response: 80-120 ms.
- Button press: immediate, with 1-2 px travel.
- Panel changes: 140-200 ms, using a short mechanical slide or opacity change.
- Indicator lamps may pulse slowly during active sync.
- Charts reveal without elastic or playful bounce.
- Avoid ambient motion. Nothing should move unless the system or user caused it.

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

## 13. Implementation constraint

The visual influence is conceptual: precision hardware, playful colour, Swiss editorial order, and tactile interaction. Do not copy the layout, branding, iconography, or distinctive product details of any single reference or manufacturer.
