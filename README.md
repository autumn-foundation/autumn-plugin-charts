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

- **Axes and ticks.** "Nice" ticks (1, 2 or 5 × 10ⁿ). Axis titles with
  `x_label` and `y_label`.
- **A legend.** It shows for two or more series. Set it with `legend`.
- **A data table.** It is always in the markup. See [Data table](#data-table).
- **Hover details.** Each mark has a `<title>`. With `focusable`, each mark
  is a tab stop and shows a tip. See [Keyboard](#keyboard).
- **Series styles.** A token color, a dash pattern and a marker shape for
  each series. Six styles, then they repeat. Color is not the only cue.

## Install

```toml
[dependencies]
autumn-charts-plugin = { git = "https://github.com/autumn-foundation/autumn-plugin-charts" }
```

```rust
use autumn_charts_plugin::ChartsPlugin;

autumn_web::app().plugin(ChartsPlugin)
```

The plugin serves the pack CSS and adds the stories to `/_stories`. In the
layout `<head>`, after the core widget CSS, add the pack link:

```rust
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
- A non-finite value (`NaN`, `inf`) is missing. A line breaks there. A bar
  is not drawn. The table shows "No value".
- A short series is missing its last values. Values past the last category
  are not used.
- In a stacked area, a missing value adds 0. In a stack, positive and
  negative values stack apart, from zero.
- A donut draws each positive value. The table lists all values.

### Options

| Method | Charts | Effect |
|---|---|---|
| `desc` | all | The SVG `<desc>`. The default sums up the data. |
| `format` | all | A `fn(f64) -> String` for ticks, tips, the legend and the table. |
| `legend` | all | Show or hide the legend. |
| `focusable` | all | Make each mark a tab stop with a tip. |
| `table`, `table_label` | all | Where the data table shows. |
| `hide_title` | all | Hide the `<figcaption>` visually. |
| `class`, `options` | all | A root class, and the `data-*`/`hx-*` passthrough. |
| `x_label`, `y_label` | line, bar, area | Axis titles. |
| `min`, `max`, `ticks` | line, bar, area | The y domain and about how many ticks. |
| `zero_baseline` | line | Put 0 in the y domain. Bars and areas always have it. |
| `stacked` | bar, area | Stack the series. |
| `center_label` | donut | A label under the total. |

## Data table

Each chart writes a `<table>` with a `<caption>`, a header for each series
and a row header for each category. It is the text alternative for the
chart.

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
| Tab | Moves to the next mark. The tip shows its series, category and value. |
| Shift+Tab | Moves to the previous mark. |

In a donut, the tip shows in the centre, in place of the total. The SVG is
`role="group"`, and each mark is `role="img"` with an `aria-label`.

## Styling

The pack CSS uses only the public tokens. Override these properties on
`.autumn-charts-chart`:

| Property | Default |
|---|---|
| `--autumn-charts-grid` | `var(--border)` |
| `--autumn-charts-axis` | `var(--text-muted)` |
| `--autumn-charts-area-opacity` | `0.18` |

Series colors are `--primary`, `--info`, `--warning`, `--success`,
`--danger` and `--text-muted`. Override `--autumn-charts-color` on
`.autumn-charts-s0` … `.autumn-charts-s5`.

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
