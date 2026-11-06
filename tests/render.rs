//! Render behaviour of each chart type.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test fails on a panic"
)]

use autumn_charts_plugin::{AreaChart, BarChart, DonutChart, LineChart, Series, TableMode};
use autumn_web::form::FieldOptions;
use proptest::prelude::*;

const MONTHS: [&str; 4] = ["Jan", "Feb", "Mar", "Apr"];
const VISITS: [f64; 4] = [0.0, 20.0, 60.0, 100.0];
const SIGNUPS: [f64; 4] = [10.0, 30.0, 25.0, 40.0];

const fn one() -> [Series<'static>; 1] {
    [Series::new("Visits", &VISITS)]
}

const fn two() -> [Series<'static>; 2] {
    [
        Series::new("Visits", &VISITS),
        Series::new("Signups", &SIGNUPS),
    ]
}

/// The attribute `name` of each element whose class list holds `class`.
fn attrs(html: &str, class: &str, name: &str) -> Vec<String> {
    let needle = format!("class=\"{class}");
    html.match_indices(&needle)
        .filter_map(|(at, _)| {
            let start = html[..at].rfind('<')?;
            let end = at + html[at..].find('>')?;
            let tag = &html[start..end];
            let key = format!(" {name}=\"");
            let from = tag.find(&key)? + key.len();
            let to = from + tag[from..].find('"')?;
            Some(tag[from..to].to_owned())
        })
        .collect()
}

fn numbers(html: &str, class: &str, name: &str) -> Vec<f64> {
    attrs(html, class, name)
        .iter()
        .map(|v| v.parse::<f64>().unwrap())
        .collect()
}

/// The y of each point in a `M x,y L x,y …` path.
fn path_ys(d: &str) -> Vec<f64> {
    d.split(['M', 'L'])
        .filter(|p| !p.trim().is_empty())
        .map(|p| p.trim().split(',').nth(1).unwrap().parse::<f64>().unwrap())
        .collect()
}

fn assert_csp_safe(html: &str) {
    assert!(!html.contains("<script"), "{html}");
    assert!(!html.contains("style="), "{html}");
    assert!(!html.contains(" on"), "an on* attribute: {html}");
    assert!(!html.contains("NaN"), "{html}");
    assert!(!html.contains("inf"), "{html}");
}

// ── Root and accessible name ────────────────────────────────────────────────

#[test]
fn the_root_is_a_figure_with_a_caption_and_an_svg_image() {
    let series = one();
    let html = LineChart::new("Visits", &MONTHS, &series)
        .render()
        .into_string();
    assert!(
        html.starts_with(r#"<figure class="autumn-charts-chart autumn-charts-chart--line""#),
        "{html}"
    );
    assert!(
        html.contains(r#"<figcaption class="autumn-charts-title">Visits</figcaption>"#),
        "{html}"
    );
    assert!(html.contains(r#"role="img""#), "{html}");
    assert!(html.contains(r#"aria-label="Visits""#), "{html}");
    assert!(html.contains("<title>Visits</title>"), "{html}");
    assert!(html.contains("<desc>"), "{html}");
    assert_csp_safe(&html);
}

#[test]
fn the_default_desc_sums_up_the_data() {
    let series = two();
    let html = LineChart::new("Visits", &MONTHS, &series)
        .render()
        .into_string();
    assert!(
        html.contains(
            "<desc>Line chart with 2 series and 4 categories. Values range from 0 to 100.</desc>"
        ),
        "{html}"
    );
}

#[test]
fn an_app_desc_replaces_the_default() {
    let series = one();
    let html = LineChart::new("Visits", &MONTHS, &series)
        .desc("Visits grow each month.")
        .render()
        .into_string();
    assert!(
        html.contains("<desc>Visits grow each month.</desc>"),
        "{html}"
    );
}

#[test]
fn hide_title_keeps_the_caption_for_screen_readers() {
    let series = one();
    let html = LineChart::new("Visits", &MONTHS, &series)
        .hide_title()
        .render()
        .into_string();
    assert!(
        html.contains(r#"<figcaption class="autumn-charts-title autumn-visually-hidden">"#),
        "{html}"
    );
}

#[test]
fn the_app_passthrough_and_class_go_on_the_root() {
    let series = one();
    let options = FieldOptions::new()
        .data("report", "q3")
        .hx("get", "/charts/visits");
    let html = LineChart::new("Visits", &MONTHS, &series)
        .class("wide")
        .options(&options)
        .render()
        .into_string();
    let root = &html[..html.find('>').unwrap()];
    assert!(root.contains("autumn-charts-chart--line wide"), "{root}");
    assert!(root.contains(r#"data-report="q3""#), "{root}");
    assert!(root.contains(r#"hx-get="/charts/visits""#), "{root}");
}

// ── Axes and ticks ──────────────────────────────────────────────────────────

#[test]
fn the_y_axis_has_a_label_for_each_nice_tick() {
    let series = one();
    let html = LineChart::new("Visits", &MONTHS, &series)
        .render()
        .into_string();
    let ticks: Vec<String> = html
        .match_indices(r#"class="autumn-charts-tick autumn-charts-tick--y""#)
        .map(|(at, _)| {
            let rest = &html[at..];
            let open = rest.find('>').unwrap() + 1;
            let close = rest.find("</text>").unwrap();
            rest[open..close].to_owned()
        })
        .collect();
    assert_eq!(ticks, ["0", "20", "40", "60", "80", "100"], "{html}");
    assert_eq!(
        attrs(&html, "autumn-charts-grid", "y1").len(),
        6,
        "one grid line for each tick: {html}"
    );
}

#[test]
fn the_x_axis_has_a_label_for_each_category() {
    let series = one();
    let html = BarChart::new("Visits", &MONTHS, &series)
        .render()
        .into_string();
    for month in MONTHS {
        assert!(
            html.contains(&format!(">{month}</text>")),
            "{month}: {html}"
        );
    }
}

#[test]
fn x_labels_thin_out_when_they_do_not_fit() {
    let labels: Vec<String> = (0..120).map(|i| format!("Day {i}")).collect();
    let cats: Vec<&str> = labels.iter().map(String::as_str).collect();
    let values: Vec<f64> = (0..120).map(f64::from).collect();
    let series = [Series::new("Load", &values)];
    let html = LineChart::new("Load", &cats, &series)
        .render()
        .into_string();
    let shown = html
        .matches(r#"class="autumn-charts-tick autumn-charts-tick--x""#)
        .count();
    assert!(shown > 1 && shown < 30, "{shown} x labels");
    assert!(html.contains(">Day 0</text>"), "the first label shows");
}

#[test]
fn axis_titles_show_when_set() {
    let series = one();
    let html = LineChart::new("Visits", &MONTHS, &series)
        .x_label("Month")
        .y_label("Visits per day")
        .render()
        .into_string();
    assert!(html.contains(">Month</text>"), "{html}");
    assert!(html.contains(">Visits per day</text>"), "{html}");
    assert_eq!(
        html.matches(r#"class="autumn-charts-axis-label"#).count(),
        2,
        "{html}"
    );
}

#[test]
fn min_and_max_override_the_domain() {
    let series = one();
    let html = LineChart::new("Visits", &MONTHS, &series)
        .min(0.0)
        .max(200.0)
        .render()
        .into_string();
    assert!(html.contains(">200</text>"), "{html}");
}

#[test]
fn a_line_domain_does_not_hold_zero_unless_asked() {
    let values = [50.0, 55.0, 60.0];
    let series = [Series::new("Temp", &values)];
    let cats = ["a", "b", "c"];
    let html = LineChart::new("Temp", &cats, &series)
        .render()
        .into_string();
    assert!(!html.contains(">0</text>"), "{html}");
    let html = LineChart::new("Temp", &cats, &series)
        .zero_baseline()
        .render()
        .into_string();
    assert!(html.contains(">0</text>"), "{html}");
}

#[test]
fn the_formatter_writes_ticks_tips_and_table_values() {
    let series = one();
    let html = BarChart::new("Visits", &MONTHS, &series)
        .format(|v| format!("{v} pts"))
        .render()
        .into_string();
    assert!(html.contains(">100 pts</text>"), "tick: {html}");
    assert!(html.contains("<td>60 pts</td>"), "table: {html}");
    assert!(html.contains("Visits, Mar: 60 pts"), "mark: {html}");
}

// ── Series and marks ────────────────────────────────────────────────────────

#[test]
fn each_series_has_its_own_style_class() {
    let series = two();
    let html = LineChart::new("Traffic", &MONTHS, &series)
        .render()
        .into_string();
    assert_eq!(
        attrs(&html, "autumn-charts-line autumn-charts-s0", "d").len(),
        1
    );
    assert_eq!(
        attrs(&html, "autumn-charts-line autumn-charts-s1", "d").len(),
        1
    );
    assert_eq!(
        html.matches(r#"class="autumn-charts-mark""#).count(),
        8,
        "one mark for each point: {html}"
    );
}

#[test]
fn styles_repeat_after_six_series() {
    let values = [1.0];
    let series: Vec<Series<'_>> = (0..7).map(|_| Series::new("S", &values)).collect();
    let html = BarChart::new("Seven", &["a"], &series)
        .render()
        .into_string();
    assert_eq!(
        attrs(&html, "autumn-charts-bar autumn-charts-s0", "x").len(),
        2
    );
    assert!(!html.contains("autumn-charts-s6"), "{html}");
}

#[test]
fn each_mark_has_a_title_with_its_value() {
    let series = two();
    let html = LineChart::new("Traffic", &MONTHS, &series)
        .render()
        .into_string();
    assert!(html.contains("<title>Visits, Feb: 20</title>"), "{html}");
    assert!(html.contains("<title>Signups, Apr: 40</title>"), "{html}");
}

#[test]
fn a_missing_value_breaks_the_line() {
    let values = [1.0, f64::NAN, 3.0, 4.0];
    let series = [Series::new("Gaps", &values)];
    let html = LineChart::new("Gaps", &MONTHS, &series)
        .render()
        .into_string();
    let d = attrs(&html, "autumn-charts-line autumn-charts-s0", "d");
    assert_eq!(d.len(), 1, "{html}");
    assert_eq!(d[0].matches('M').count(), 2, "two segments: {}", d[0]);
    assert_eq!(html.matches(r#"class="autumn-charts-mark""#).count(), 3);
    assert!(html.contains("<td>No value</td>"), "{html}");
}

#[test]
fn a_short_series_has_gaps_and_a_long_series_is_cut() {
    let short = [1.0, 2.0];
    let long = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let series = [Series::new("Short", &short), Series::new("Long", &long)];
    let html = BarChart::new("Lengths", &MONTHS, &series)
        .render()
        .into_string();
    assert_eq!(html.matches("<rect").count(), 6, "2 + 4 bars: {html}");
    assert_eq!(html.matches("<td>No value</td>").count(), 2, "{html}");
    assert!(!html.contains("<td>6</td>"), "{html}");
}

#[test]
fn many_points_mark_the_chart_dense() {
    let values: Vec<f64> = (0..60).map(f64::from).collect();
    let labels: Vec<String> = (0..60).map(|i| i.to_string()).collect();
    let cats: Vec<&str> = labels.iter().map(String::as_str).collect();
    let series = [Series::new("Load", &values)];
    let html = LineChart::new("Load", &cats, &series)
        .render()
        .into_string();
    assert!(html.contains("autumn-charts-chart--dense"), "{html}");
    let html = LineChart::new("Visits", &MONTHS, &one())
        .render()
        .into_string();
    assert!(!html.contains("autumn-charts-chart--dense"), "{html}");
}

// ── Legend ──────────────────────────────────────────────────────────────────

#[test]
fn the_legend_shows_for_two_or_more_series() {
    let series = two();
    let html = LineChart::new("Traffic", &MONTHS, &series)
        .render()
        .into_string();
    assert!(
        html.contains(r#"<ul class="autumn-charts-legend""#),
        "{html}"
    );
    assert_eq!(
        html.matches(r#"<li class="autumn-charts-legend-item">"#)
            .count(),
        2
    );
    assert!(html.contains("Signups</li>"), "{html}");
}

#[test]
fn the_legend_hides_for_one_series_unless_asked() {
    let series = one();
    let html = LineChart::new("Visits", &MONTHS, &series)
        .render()
        .into_string();
    assert!(!html.contains("autumn-charts-legend"), "{html}");
    let html = LineChart::new("Visits", &MONTHS, &series)
        .legend(true)
        .render()
        .into_string();
    assert!(html.contains("autumn-charts-legend"), "{html}");
    let html = LineChart::new("Traffic", &MONTHS, &two())
        .legend(false)
        .render()
        .into_string();
    assert!(!html.contains("autumn-charts-legend"), "{html}");
}

#[test]
fn the_legend_swatch_is_hidden_from_screen_readers() {
    let series = two();
    let html = BarChart::new("Traffic", &MONTHS, &series)
        .render()
        .into_string();
    assert!(
        html.contains(r#"<svg class="autumn-charts-swatch autumn-charts-s1" viewBox="0 0 24 12" aria-hidden="true""#),
        "{html}"
    );
}

// ── Data table ──────────────────────────────────────────────────────────────

#[test]
fn the_data_table_is_always_there_and_hidden_by_default() {
    let series = two();
    let html = LineChart::new("Traffic", &MONTHS, &series)
        .x_label("Month")
        .render()
        .into_string();
    assert!(
        html.contains(r#"<table class="autumn-charts-table autumn-visually-hidden">"#),
        "{html}"
    );
    assert!(html.contains("<caption>Traffic</caption>"), "{html}");
    assert!(html.contains(r#"<th scope="col">Month</th>"#), "{html}");
    assert!(html.contains(r#"<th scope="col">Visits</th>"#), "{html}");
    assert!(html.contains(r#"<th scope="col">Signups</th>"#), "{html}");
    assert!(
        html.contains(r#"<th scope="row">Mar</th><td>60</td><td>25</td>"#),
        "{html}"
    );
}

#[test]
fn the_table_header_defaults_to_category() {
    let html = BarChart::new("Visits", &MONTHS, &one())
        .render()
        .into_string();
    assert!(html.contains(r#"<th scope="col">Category</th>"#), "{html}");
}

#[test]
fn the_table_can_show_in_a_details_element() {
    let series = one();
    let html = LineChart::new("Visits", &MONTHS, &series)
        .table(TableMode::Details)
        .render()
        .into_string();
    assert!(
        html.contains(r#"<details class="autumn-charts-data"><summary>Show the data</summary><table class="autumn-charts-table">"#),
        "{html}"
    );
    let html = LineChart::new("Visits", &MONTHS, &series)
        .table(TableMode::Details)
        .table_label("Data")
        .render()
        .into_string();
    assert!(html.contains("<summary>Data</summary>"), "{html}");
}

#[test]
fn the_table_can_show_below_the_chart() {
    let series = one();
    let html = LineChart::new("Visits", &MONTHS, &series)
        .table(TableMode::Visible)
        .render()
        .into_string();
    assert!(
        html.contains(r#"<table class="autumn-charts-table">"#),
        "{html}"
    );
    assert!(!html.contains("<details"), "{html}");
}

// ── Hover details ───────────────────────────────────────────────────────────

#[test]
fn focusable_marks_take_focus_and_show_a_tip() {
    let series = one();
    let html = LineChart::new("Visits", &MONTHS, &series)
        .focusable()
        .render()
        .into_string();
    assert!(html.contains(r#"role="group""#), "{html}");
    assert!(!html.contains(r#"<svg class="autumn-charts-svg" viewBox="0 0 640 320" role="img""#));
    assert_eq!(html.matches(r#"tabindex="0""#).count(), 4, "{html}");
    assert!(
        html.contains(
            r#"<g class="autumn-charts-mark" tabindex="0" role="img" aria-label="Visits, Mar: 60">"#
        ),
        "{html}"
    );
    assert_eq!(
        html.matches(r#"class="autumn-charts-tip""#).count(),
        4,
        "{html}"
    );
    assert!(!html.contains("<title>Visits, Mar: 60</title>"), "{html}");
}

#[test]
fn marks_do_not_take_focus_by_default() {
    let html = LineChart::new("Visits", &MONTHS, &one())
        .render()
        .into_string();
    assert!(!html.contains("tabindex"), "{html}");
    assert!(!html.contains("autumn-charts-tip"), "{html}");
}

// ── Bars ────────────────────────────────────────────────────────────────────

#[test]
fn grouped_bars_sit_side_by_side() {
    let series = two();
    let html = BarChart::new("Traffic", &MONTHS, &series)
        .render()
        .into_string();
    assert!(html.contains("autumn-charts-chart--bar"), "{html}");
    let a = numbers(&html, "autumn-charts-bar autumn-charts-s0", "x");
    let b = numbers(&html, "autumn-charts-bar autumn-charts-s1", "x");
    assert_eq!((a.len(), b.len()), (4, 4));
    for (x0, x1) in a.iter().zip(&b) {
        assert!(x1 > x0, "series 1 is right of series 0");
    }
}

#[test]
fn bars_grow_from_zero_in_both_directions() {
    let values = [-10.0, 5.0];
    let series = [Series::new("Net", &values)];
    let html = BarChart::new("Net", &["a", "b"], &series)
        .render()
        .into_string();
    let zero = numbers(&html, "autumn-charts-zero", "y1");
    assert_eq!(zero.len(), 1, "a zero line: {html}");
    let ys = numbers(&html, "autumn-charts-bar autumn-charts-s0", "y");
    let hs = numbers(&html, "autumn-charts-bar autumn-charts-s0", "height");
    assert!(
        (ys[0] - zero[0]).abs() < 0.01,
        "the negative bar starts at zero"
    );
    assert!(
        (ys[1] + hs[1] - zero[0]).abs() < 0.01,
        "the positive bar ends at zero"
    );
    assert!(hs[0] > hs[1], "|-10| is longer than |5|");
}

#[test]
fn stacked_bars_sit_on_each_other() {
    let series = two();
    let html = BarChart::new("Traffic", &MONTHS, &series)
        .stacked()
        .render()
        .into_string();
    let x0 = numbers(&html, "autumn-charts-bar autumn-charts-s0", "x");
    let x1 = numbers(&html, "autumn-charts-bar autumn-charts-s1", "x");
    assert_eq!(x0, x1, "one column for each category");
    let y0 = numbers(&html, "autumn-charts-bar autumn-charts-s0", "y");
    let y1 = numbers(&html, "autumn-charts-bar autumn-charts-s1", "y");
    let h1 = numbers(&html, "autumn-charts-bar autumn-charts-s1", "height");
    for i in 1..4 {
        assert!((y1[i] + h1[i] - y0[i]).abs() < 0.01, "segment {i}: {html}");
    }
    assert!(
        html.contains(">140</text>") || html.contains(">150</text>"),
        "the domain holds the sum: {html}"
    );
}

#[test]
fn stacked_bars_stack_negatives_down() {
    let a = [5.0, -5.0];
    let b = [5.0, -5.0];
    let series = [Series::new("A", &a), Series::new("B", &b)];
    let html = BarChart::new("Net", &["x", "y"], &series)
        .stacked()
        .render()
        .into_string();
    let ya = numbers(&html, "autumn-charts-bar autumn-charts-s0", "y");
    let ha = numbers(&html, "autumn-charts-bar autumn-charts-s0", "height");
    let yb = numbers(&html, "autumn-charts-bar autumn-charts-s1", "y");
    assert!(
        (ya[1] + ha[1] - yb[1]).abs() < 0.01,
        "B hangs below A: {html}"
    );
}

// ── Areas ───────────────────────────────────────────────────────────────────

#[test]
fn an_area_fills_down_to_zero() {
    let series = one();
    let html = AreaChart::new("Visits", &MONTHS, &series)
        .render()
        .into_string();
    assert!(html.contains("autumn-charts-chart--area"), "{html}");
    let d = attrs(&html, "autumn-charts-area autumn-charts-s0", "d");
    assert_eq!(d.len(), 1, "{html}");
    assert!(d[0].ends_with('Z'), "a closed shape: {}", d[0]);
    assert_eq!(
        attrs(&html, "autumn-charts-line autumn-charts-s0", "d").len(),
        1
    );
}

#[test]
fn stacked_areas_sit_on_each_other() {
    let series = two();
    let html = AreaChart::new("Traffic", &MONTHS, &series)
        .stacked()
        .render()
        .into_string();
    let cy0 = path_ys(&attrs(&html, "autumn-charts-line autumn-charts-s0", "d")[0]);
    let cy1 = path_ys(&attrs(&html, "autumn-charts-line autumn-charts-s1", "d")[0]);
    assert_eq!((cy0.len(), cy1.len()), (4, 4), "{html}");
    for i in 0..4 {
        assert!(cy1[i] < cy0[i], "series 1 is above series 0 at {i}");
    }
}

// ── Donut ───────────────────────────────────────────────────────────────────

const SHARE: [(&str, f64); 3] = [("Rust", 50.0), ("Go", 30.0), ("Zig", 20.0)];

#[test]
fn a_donut_has_one_slice_for_each_value() {
    let html = DonutChart::new("Languages", &SHARE).render().into_string();
    assert!(html.contains("autumn-charts-chart--donut"), "{html}");
    for k in 0..3 {
        assert_eq!(
            attrs(
                &html,
                &format!("autumn-charts-slice autumn-charts-s{k}"),
                "d"
            )
            .len(),
            1,
            "slice {k}: {html}"
        );
    }
    assert!(html.contains("<title>Rust: 50 (50%)</title>"), "{html}");
    assert_csp_safe(&html);
}

#[test]
fn a_donut_shows_the_total_and_a_legend_with_shares() {
    let html = DonutChart::new("Languages", &SHARE)
        .center_label("Total")
        .render()
        .into_string();
    assert!(html.contains(">100</text>"), "{html}");
    assert!(html.contains(">Total</text>"), "{html}");
    assert!(html.contains("Go: 30 (30%)</li>"), "{html}");
}

#[test]
fn a_donut_table_has_value_and_share() {
    let html = DonutChart::new("Languages", &SHARE).render().into_string();
    assert!(html.contains(r#"<th scope="col">Share</th>"#), "{html}");
    assert!(
        html.contains(r#"<th scope="row">Zig</th><td>20</td><td>20%</td>"#),
        "{html}"
    );
}

#[test]
fn a_full_donut_is_a_ring() {
    let html = DonutChart::new("All", &[("Only", 7.0)])
        .render()
        .into_string();
    assert!(html.contains(r#"fill-rule="evenodd""#), "{html}");
    assert!(!html.contains("NaN"), "{html}");
}

#[test]
fn a_donut_skips_negative_and_missing_values_but_lists_them() {
    let slices = [("A", 3.0), ("B", -2.0), ("C", f64::NAN), ("D", 1.0)];
    let html = DonutChart::new("Mixed", &slices).render().into_string();
    assert_eq!(html.matches("autumn-charts-slice ").count(), 2, "{html}");
    assert!(
        html.contains(r#"<th scope="row">B</th><td>-2</td><td>No value</td>"#),
        "{html}"
    );
    assert!(
        html.contains(r#"<th scope="row">C</th><td>No value</td>"#),
        "{html}"
    );
    assert!(
        html.contains(">4</text>"),
        "the total counts A and D: {html}"
    );
}

#[test]
fn focusable_donut_slices_take_focus() {
    let html = DonutChart::new("Languages", &SHARE)
        .focusable()
        .render()
        .into_string();
    assert_eq!(html.matches(r#"tabindex="0""#).count(), 3, "{html}");
    assert!(html.contains(r#"aria-label="Go: 30 (30%)""#), "{html}");
}

#[test]
fn a_donut_takes_the_shared_options() {
    let options = FieldOptions::new().data("kind", "share");
    let html = DonutChart::new("Languages", &SHARE)
        .desc("Rust leads.")
        .format(|v| format!("{v} users"))
        .legend(false)
        .table(TableMode::Visible)
        .hide_title()
        .class("small")
        .options(&options)
        .render()
        .into_string();
    assert!(html.contains("<desc>Rust leads.</desc>"), "{html}");
    assert!(html.contains("<td>50 users</td>"), "{html}");
    assert!(!html.contains("autumn-charts-legend"), "{html}");
    assert!(
        html.contains(r#"<table class="autumn-charts-table">"#),
        "{html}"
    );
    assert!(html.contains("autumn-charts-chart--donut small"), "{html}");
    assert!(html.contains(r#"data-kind="share""#), "{html}");
}

// ── Edge cases ──────────────────────────────────────────────────────────────

#[test]
fn hostile_text_is_escaped() {
    let evil = r#""><script>alert(1)</script>"#;
    let values = [1.0];
    let series = [Series::new(evil, &values)];
    let cats = [evil];
    for html in [
        LineChart::new(evil, &cats, &series)
            .x_label(evil)
            .y_label(evil)
            .render()
            .into_string(),
        BarChart::new(evil, &cats, &series)
            .focusable()
            .render()
            .into_string(),
        DonutChart::new(evil, &[(evil, 1.0)])
            .center_label(evil)
            .render()
            .into_string(),
    ] {
        assert!(!html.contains("<script>"), "{html}");
        assert!(html.contains("&lt;script&gt;"), "{html}");
    }
}

#[test]
fn empty_charts_render_with_no_data() {
    let no_series: [Series<'_>; 0] = [];
    for html in [
        LineChart::new("Empty", &[], &no_series)
            .render()
            .into_string(),
        BarChart::new("Empty", &MONTHS, &no_series)
            .render()
            .into_string(),
        AreaChart::new("Empty", &[], &one()).render().into_string(),
        DonutChart::new("Empty", &[]).render().into_string(),
    ] {
        assert!(html.contains("<desc>No data.</desc>"), "{html}");
        assert!(html.contains("autumn-charts-table"), "{html}");
        assert_csp_safe(&html);
    }
}

#[test]
fn output_is_deterministic() {
    let series = two();
    let a = BarChart::new("Traffic", &MONTHS, &series)
        .stacked()
        .render()
        .into_string();
    let b = BarChart::new("Traffic", &MONTHS, &series)
        .stacked()
        .render()
        .into_string();
    assert_eq!(a, b);
}

#[test]
fn a_chart_renders_in_a_maud_template() {
    let series = one();
    let chart = LineChart::new("Visits", &MONTHS, &series);
    let html = maud::html! { main { (chart) } }.into_string();
    assert!(html.starts_with("<main><figure"), "{html}");
}

fn value() -> impl Strategy<Value = f64> {
    prop_oneof![
        6 => -1e6..1e6f64,
        1 => any::<f64>(),
        1 => Just(f64::MAX),
        1 => Just(-f64::MAX),
        1 => Just(f64::NAN),
        1 => Just(f64::INFINITY),
        1 => Just(0.0),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn no_chart_writes_nan_or_inf(
        a in prop::collection::vec(value(), 0..8),
        b in prop::collection::vec(value(), 0..8),
        stacked in any::<bool>(),
        focus in any::<bool>(),
    ) {
        let cats = ["A", "B", "C", "D", "E", "F"];
        let series = [Series::new("One", &a), Series::new("Two", &b)];
        let slices: Vec<(&str, f64)> = a.iter().map(|v| ("S", *v)).collect();
        let mut line = LineChart::new("L", &cats, &series);
        let mut bar = BarChart::new("B", &cats, &series);
        let mut area = AreaChart::new("A", &cats, &series);
        let mut donut = DonutChart::new("D", &slices);
        if stacked {
            bar = bar.stacked();
            area = area.stacked();
        }
        if focus {
            line = line.focusable();
            donut = donut.focusable();
        }
        for html in [
            line.render().into_string(),
            bar.render().into_string(),
            area.render().into_string(),
            donut.render().into_string(),
        ] {
            prop_assert!(!html.contains("NaN"), "{html}");
            prop_assert!(!html.contains("inf"), "{html}");
            prop_assert!(!html.contains("style="), "{html}");
        }
    }
}
