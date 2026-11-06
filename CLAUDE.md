# CLAUDE.md - autumn-charts-plugin

The charts v2 widget pack for autumn-web (autumn#3415). It follows the
widget contract in autumn `docs/guide/widget-packs.md` and ADR 0030.

## Rules

- **No script.** The pack has no JS asset. Hover and focus details use
  `<title>` and CSS.
- **The contract.** Classes are `autumn-charts-*` or core classes. CSS uses
  only public tokens and `--autumn-charts-*`. No `style` attribute: geometry
  goes in SVG attributes.
- **Finite output.** Each number in the markup goes through `svg::num`.
  `scale.rs` takes any `f64` and returns finite values. Keep the proptests
  in `tests/scale.rs` and `tests/render.rs` green.
- **The data table stays.** Each chart writes one. `TableMode` changes
  only where it shows.
- **Color is not the only cue.** Each series style has a color, a dash and
  a marker shape. Bars and slices have the shape cut out of them.
- **No `id`.** Two charts on one page must not collide.
- **Snapshots.** A markup change updates `tests/snapshots/`. Read the diff.
- **Comments stand alone.** No review rounds or findings labels. Write in
  ASD-STE100: short sentences, active voice.

## Commands

- Test: `cargo test`
- Lint: `cargo fmt --all && cargo clippy --all-targets -- -D warnings`
- Browser: `AUTUMN_CHROMIUM=/path/to/chrome cargo test --features system-tests --test browser -- --include-ignored`
- Snapshots: `INSTA_UPDATE=always cargo test --test snapshots`
- Coverage: `cargo llvm-cov --fail-under-lines 85`

## Layout

| Path | What |
|---|---|
| `src/scale.rs` | `nice_ticks`, `domain`, `LinearScale`, number labels. |
| `src/svg.rs` | Coordinates, paths, marker shapes. |
| `src/chart.rs` | Shared options and parts: root, SVG, mark, legend, table. |
| `src/cartesian.rs` | `LineChart`, `BarChart`, `AreaChart`. |
| `src/donut.rs` | `DonutChart`. |
| `src/stories.rs` | One story for each chart type and feature. |
| `src/charts.css` | The pack CSS. |
| `tests/` | `scale`, `render`, `snapshots`, `conformance`, `browser`. |
