# autumn-charts-plugin

Charts v2 for [autumn-web](https://github.com/autumn-foundation/autumn).
It is a first-party widget pack (autumn#3415). The core `sparkline`,
`bar_chart` and `line_chart` stay in core.

The server writes each chart as SVG. There is no script. Charts work with
JavaScript off and under a strict CSP.

| Chart | Builder |
|---|---|
| Line, many series | `LineChart` |
| Bar, grouped | `BarChart` |
| Bar, stacked | `BarChart::stacked` |
| Area | `AreaChart` |
| Area, stacked | `AreaChart::stacked` |
| Donut | `DonutChart` |

Each chart has:

- **Axes and ticks.** A value axis with "nice" ticks (1, 2 or 5 × 10ⁿ),
  and a category axis. Axis titles with `x_label` and `y_label`.
- **A legend.** It shows for two or more series, and on each donut. Set it
  with `legend`.
- **A data table.** It is always in the markup. See [Data table](#data-table).
- **Hover details.** Each mark has a `<title>`. With `focusable`, each mark
  is a tab stop with an `aria-label` and a tip. See [Keyboard](#keyboard).
- **Series styles.** Color is not the only cue. A line or an area has a
  token color, a dash pattern and a marker shape. A bar or a slice has a
  token color and the marker shape cut out of it. The legend shows the
  same cues. Six styles, then they repeat.

## Install

```toml
[dependencies]
autumn-charts-plugin = { git = "https://github.com/autumn-foundation/autumn-plugin-charts" }
```

```rust,ignore
use autumn_charts_plugin::ChartsPlugin;

autumn_web::app().plugin(ChartsPlugin)
```

The plugin serves the pack CSS and adds the stories to `/_stories`. In the
layout `<head>`, after the core widget CSS, add the pack link:

```rust,ignore
head {
    link rel="stylesheet" href=(autumn_web::ui::WIDGETS_CSS_PATH);
    (autumn_charts_plugin::pack().head())
}
```

## Use

```rust
use autumn_charts_plugin::{BarChart, DonutChart, LineChart, Series, TableMode};

const MONTHS: [&str; 3] = ["Jan", "Feb", "Mar"];
const WEB: [f64; 3] = [120.0, 132.0, 101.0];
const MOBILE: [f64; 3] = [220.0, 182.0, f64::NAN];

let series = [Series::new("Web", &WEB), Series::new("Mobile", &MOBILE)];

let line = LineChart::new("Visits", &MONTHS, &series)
    .x_label("Month")
    .y_label("Visits")
    .render();

let bars = BarChart::new("Visits", &MONTHS, &series)
    .stacked()
    .focusable()
    .table(TableMode::Details)
    .render();

let donut = DonutChart::new("Plans", &[("Free", 120.0), ("Team", 45.0)])
    .center_label("Accounts")
    .render();
```

Each builder also implements `maud::Render`, so `(chart)` works in
`html!`.

### Data

- A `Series` has a name and one value for each category.
- A non-finite value (`NaN`, `inf`) is missing. A line breaks there. The
  chart draws no bar for it. The table shows "No value".
- A short series is missing its last values. The chart ignores values past
  the last category.
- In a stack, positive and negative values stack apart, from zero. In a
  stacked area, a missing value adds 0. Give each series of a stacked area
  one sign: a series that changes sign draws a crossed shape.
- A donut draws each positive value. The table lists all values. A share
  that is not 0 or all shows as `<0.1%` or `>99.9%`, never as `0%` or
  `100%`.
- Past 40 categories, markers show only on hover and focus.

### Options

| Method | Charts | Effect |
|---|---|---|
| `desc` | all | The SVG name for screen readers. The default gives the chart type, the counts and the value range. |
| `format` | all | A `&dyn Fn(f64) -> String` for ticks, tips, the summary and the table, and the donut legend and total. It can capture a locale. |
| `words` | all | The fixed words in the table: `Words::new().category(…).no_value(…)`. |
| `legend` | all | Show or hide the legend. |
| `focusable` | all | Make each mark a tab stop with a tip. |
| `table`, `table_label` | all | Where the data table shows. |
| `hide_title` | all | Hide the `<figcaption>` visually. |
| `class`, `options` | all | A root class, and the `data-*`/`hx-*` passthrough. |
| `x_label`, `y_label` | line, bar, area | Axis titles. |
| `min`, `max`, `ticks` | line, bar, area | The y domain and about how many ticks (1 to 18). A non-finite `min` or `max` does nothing. With a `min` above 0, bars and areas draw from the axis. |
| `zero_baseline` | line | Put 0 in the y domain. Bars and areas always have it. |
| `stacked` | bar, area | Stack the series. |
| `center_label` | donut | A label under the total. |

## Data table

Each chart writes a `<table>` with a `<caption>`, a header for each series
and a row header for each category. It is the text alternative for the
chart. The caption is the title, then the `y_label`.

The `<figcaption>` names the figure. The summary (`desc`) names the SVG,
so a screen reader does not read the title two times.

| `TableMode` | Effect |
|---|---|
| `Hidden` (default) | Visually hidden. Screen readers read it. |
| `Details` | In a `<details>` below the chart. The summary is `table_label`. |
| `Visible` | Shown below the chart. |

## Keyboard

Marks do not take focus by default. Many marks make many tab stops.

With `focusable`:

| Key | Effect |
|---|---|
| Tab | Moves to the next mark. The tip shows its series, category and value. The order is series by series: each value of the first series, then each value of the next. |
| Shift+Tab | Moves to the previous mark. |

Each mark is one tab stop. Use `focusable` only on a chart with few marks.
The focused mark has an outline. The tip stays while the pointer is on the
mark or on its tip.

In a donut, the tip shows in the centre, in place of the total. The SVG is
`role="group"`, and each mark is `role="img"` with an `aria-label`.

## Styling

All colors in the pack CSS are public tokens. Override these properties on
`.autumn-charts-chart`:

| Property | Default |
|---|---|
| `--autumn-charts-grid` | `var(--border)` |
| `--autumn-charts-axis` | `var(--text-muted)` |
| `--autumn-charts-area-opacity` | `0.18` |

Each series class `.autumn-charts-s0` … `.autumn-charts-s5` sets
`--autumn-charts-color` (`--primary`, `--info`, `--warning`, `--success`,
`--danger`, `--text-muted`) and `--autumn-charts-dash`. To change one, use
a selector that is more specific, for example
`.my-report .autumn-charts-s0 { --autumn-charts-color: var(--danger); }`.

In forced-colors mode, marks use `CanvasText`, gaps and cut-outs use
`Canvas`, and focus uses `Highlight`.

## Limits

- The x axis shows categories. It has no number or time scale.
- SVG text scales with the chart. In a narrow chart, labels are small.
- The default summary is English. Set `desc` and `words` for another
  language.

## Tests

```bash
cargo test
```

| Suite | What |
|---|---|
| `tests/scale.rs` | Proptests: ticks are finite, ascending, bounded and cover the domain. The scale is finite and monotonic. |
| `tests/render.rs` | Behaviour of each chart and option. A proptest checks that no chart writes `NaN` or `inf`. |
| `tests/snapshots.rs` | An SVG snapshot for each chart type ([insta](https://insta.rs)). |
| `tests/conformance.rs` | The static widget conformance gate. |
| `tests/browser.rs` | axe in the light and the dark scheme, console and keyboard, and the focus tips. |

The browser tests need Chromium:

```bash
AUTUMN_CHROMIUM=/path/to/chrome cargo test --features system-tests --test browser -- --include-ignored
```

To update a snapshot, run `INSTA_UPDATE=always cargo test --test snapshots`
and read the diff.

`tests/fixtures/axe-core/` is a copy of axe-core 4.x (MPL-2.0). It is not in
the published crate.

See [docs/plan.md](docs/plan.md) and
[ADR 0001](docs/adr/0001-script-free-svg-charts.md).

## Licence

MIT or Apache-2.0.
