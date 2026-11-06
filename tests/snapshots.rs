//! A snapshot of the SVG for each chart type.
//!
//! Update with `INSTA_UPDATE=always cargo test --test snapshots`, then
//! read the diff.

use autumn_charts_plugin::{AreaChart, BarChart, DonutChart, LineChart, Series, TableMode};

const QUARTERS: [&str; 4] = ["Q1", "Q2", "Q3", "Q4"];
const NORTH: [f64; 4] = [12.0, 19.0, 7.0, 15.0];
const SOUTH: [f64; 4] = [8.0, 11.0, 14.0, 9.5];
const NET: [f64; 4] = [-6.0, 4.0, f64::NAN, 10.0];

const fn regions() -> [Series<'static>; 2] {
    [Series::new("North", &NORTH), Series::new("South", &SOUTH)]
}

/// One tag on each line, so a diff shows the changed element.
fn pretty(html: &str) -> String {
    html.replace("><", ">\n<")
}

#[test]
fn line_chart() {
    let series = regions();
    let html = LineChart::new("Sales", &QUARTERS, &series)
        .x_label("Quarter")
        .y_label("Units")
        .render()
        .into_string();
    insta::assert_snapshot!(pretty(&html));
}

#[test]
fn line_chart_with_a_gap() {
    let series = [Series::new("Net", &NET)];
    let html = LineChart::new("Net change", &QUARTERS, &series)
        .table(TableMode::Details)
        .render()
        .into_string();
    insta::assert_snapshot!(pretty(&html));
}

#[test]
fn bar_chart() {
    let series = regions();
    let html = BarChart::new("Sales", &QUARTERS, &series)
        .render()
        .into_string();
    insta::assert_snapshot!(pretty(&html));
}

#[test]
fn bar_chart_with_negative_values() {
    let series = [Series::new("Net", &NET)];
    let html = BarChart::new("Net change", &QUARTERS, &series)
        .render()
        .into_string();
    insta::assert_snapshot!(pretty(&html));
}

#[test]
fn stacked_bar_chart() {
    let series = regions();
    let html = BarChart::new("Sales", &QUARTERS, &series)
        .stacked()
        .render()
        .into_string();
    insta::assert_snapshot!(pretty(&html));
}

#[test]
fn area_chart() {
    let series = regions();
    let html = AreaChart::new("Sales", &QUARTERS, &series)
        .render()
        .into_string();
    insta::assert_snapshot!(pretty(&html));
}

#[test]
fn stacked_area_chart() {
    let series = regions();
    let html = AreaChart::new("Sales", &QUARTERS, &series)
        .stacked()
        .table(TableMode::Visible)
        .render()
        .into_string();
    insta::assert_snapshot!(pretty(&html));
}

#[test]
fn donut_chart() {
    let html = DonutChart::new(
        "Market share",
        &[("Rust", 45.0), ("Go", 30.0), ("Zig", 25.0)],
    )
    .center_label("Teams")
    .render()
    .into_string();
    insta::assert_snapshot!(pretty(&html));
}

#[test]
fn focusable_bar_chart() {
    let series = [Series::new("North", &NORTH)];
    let html = BarChart::new("Sales", &QUARTERS, &series)
        .focusable()
        .render()
        .into_string();
    insta::assert_snapshot!(pretty(&html));
}
