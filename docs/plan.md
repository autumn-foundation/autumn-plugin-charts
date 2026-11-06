# Plan: charts v2 (autumn#3415)

Part of autumn#3393. The pack ships through the widget plugin seam
(autumn ADR 0030). Core `sparkline`, `bar_chart` and `line_chart` stay in
core. This is not a breaking change.

## Goal

Server-rendered SVG charts with no script. The pack adds axes and ticks,
legends, multi-series line and bar charts, stacked bars, area charts and
donut charts. Each chart has a data table fallback. Hover details use
`<title>` or a focusable mark.

## Brainstorm

- `LinearScale`: maps a value to a pixel. It never returns NaN.
- `nice_ticks`: "nice" steps (1, 2, 5 × 10ⁿ). The ticks cover the data.
- `Series`: a name and one value for each category. A non-finite value is
  a gap.
- `LineChart`, `BarChart`, `AreaChart`: one builder each, one
  `Cartesian` core for the axes, the grid, the legend and the table.
- `BarChart::stacked`, `AreaChart::stacked`: positive and negative values
  stack apart, from the zero line.
- `DonutChart`: annular sector paths. A total in the centre.
- Legend: an HTML list below the SVG. It wraps. It is text, so it scales.
- Data table: always in the markup. Visually hidden by default. The app can
  show it, or put it in a `<details>`.
- Hover: each mark has a `<title>`. `.focusable()` adds `tabindex="0"`, an
  `aria-label` and a tip that CSS shows on `:hover` and `:focus-visible`.
- Series style: a token color, a dash pattern and a marker shape. Color is
  not the only cue.
- A value formatter: `fn(f64) -> String`, for ticks, tips and the table.
- Root: `<figure>` with a `<figcaption>`, written with
  `widget_pack::element` for the `data-*` and `hx-*` passthrough.

## Reverse brainstorm (how can it fail?)

| Fault | Prevention |
|---|---|
| NaN or `inf` in an attribute. | Each coordinate goes through one formatter. It writes `0` for a non-finite value. A proptest renders random data and looks for `NaN` and `inf`. |
| `max - min` overflows (`-1e308..1e308`). | The scale works on halves. A proptest uses the full `f64` range. |
| Ticks repeat at `1e15 + 0.1`. | `nice_ticks` drops a tick that is not above the last one. If fewer than two stay, it uses the domain ends. |
| A tick loop does not stop (step 0, subnormal). | The tick count has a hard cap. A step that is not finite and positive falls back to the domain ends. |
| `0.30000000000000004` as a label. | Labels round to the step precision. |
| `1e300` as a 300-digit label. | Large and small values use exponent notation. |
| All values equal: span 0. | The domain grows by one unit on each side. |
| Bars do not start at zero. | The bar and area domain always holds 0. |
| Stacks of `1e308 + 1e308` overflow. | Stack sums saturate to `f64::MAX`. |
| A series is shorter or longer than the categories. | Short: gaps. Long: the extra values are dropped. |
| A donut slice is 100 %. | A full ring is two half arcs. |
| A negative or NaN donut value. | It is not drawn. The table shows it. |
| Only color separates series. | Dash pattern and marker shape per series. Legend text. Data table. |
| A label injects markup. | maud escapes all text. A test uses `"><script>`. |
| Focusable marks inside `role="img"`. | With `.focusable()`, the SVG is a `role="group"`. Each mark is a `role="img"` with a name. |
| 500 tab stops. | `.focusable()` is opt-in. |
| A class fails the pack class check. | Each class is `autumn-charts-*` or a core class. The conformance test runs on each story. |
| Text distorts when the chart scales. | A fixed `viewBox` with the default aspect ratio. Not `preserveAspectRatio="none"`. |
| Long labels collide. | The left margin grows with the tick label length. X labels thin out past a count. |
| Dark mode text fails contrast. | Text uses `--text` and `--text-muted`. axe runs in both schemes. |

## Six thinking hats

- **White (facts).** Core charts are one series, with `role="img"`,
  `<title>`, `<desc>` and an opt-in hidden table. `autumn-web` 0.10 has
  the widget seam, `widget_pack::element` and the conformance checks.
  The public tokens give six usable colors: `--primary`, `--info`,
  `--success`, `--warning`, `--danger`, `--text-muted`.
- **Red (feelings).** Users want charts that look good with no setup, and
  that do not need Chart.js.
- **Black (risks).** Float edge cases. SVG text and the a11y tree in
  different browsers. Too many tab stops. Snapshot churn on small changes.
- **Yellow (value).** No script, so a strict CSP works. The data table is
  always there. Output is deterministic, so snapshots are stable and
  caches work.
- **Green (ideas).** A `<details>` table toggle. Per-series marker shapes.
  A formatter for currency and percent. Later: horizontal bars, scatter,
  log scales.
- **Blue (process).** Spec, red, green, refactor. First the scale and tick
  properties, then one snapshot for each chart type, then the conformance
  gate and axe. Keep the pure math in `scale.rs`.

## Acceptance

| Criterion | Test |
|---|---|
| Snapshot of the SVG for each chart type. | `tests/snapshots.rs` |
| Property tests over scale and ticks: no NaN, no overflow, monotonic. | `tests/scale.rs` |
| axe passes. Stories through the plugin seam. | `tests/browser.rs`, `tests/conformance.rs` |
