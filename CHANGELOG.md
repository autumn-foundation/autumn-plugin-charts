# Changelog

## [Unreleased]

### Added

- Charts v2 as a widget pack (autumn#3415). The server writes SVG. There
  is no script.
- `LineChart`, `BarChart` and `AreaChart`: many series, axes, nice ticks,
  axis titles and a legend. `BarChart::stacked` and `AreaChart::stacked`
  stack positive and negative values apart.
- `DonutChart`: slices, a total in the centre and shares in the legend.
- A data table for each chart. `TableMode` sets where it shows. `Words`
  sets its fixed words.
- Hover details: a `<title>` on each mark. `focusable` makes each mark a
  tab stop with an `aria-label` and a tip.
- `format` takes a `&dyn Fn(f64) -> String`, so it can capture a locale.
- Series styles: a color, a dash and a marker shape. Bars and slices have
  the shape cut out of them.
- `scale`: `nice_ticks`, `domain`, `LinearScale`, `format_number`,
  `format_tick`, `tick_labels`. All outputs are finite. Adjacent tick
  labels differ.
- `ChartsPlugin` and `pack()`: install the CSS and the stories.
