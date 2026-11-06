# Changelog

## [Unreleased]

### Added

- Charts v2 as a widget pack (autumn#3415). The server writes SVG. There
  is no script.
- `LineChart`, `BarChart` and `AreaChart`: many series, axes, nice ticks,
  axis titles and a legend. `BarChart::stacked` and `AreaChart::stacked`
  stack positive and negative values apart.
- `DonutChart`: slices, a total in the centre and shares in the legend.
- A data table for each chart. `TableMode` sets where it shows.
- Hover details: a `<title>` on each mark. `focusable` makes each mark a
  tab stop with a tip.
- `scale`: `nice_ticks`, `domain`, `LinearScale`, `format_number`,
  `format_tick`. All outputs are finite.
- `ChartsPlugin` and `pack()`: install the CSS and the stories.
