//! The pack passes the static widget conformance checks.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test fails on a panic"
)]

use autumn_charts_plugin::{ASSETS, ChartsPlugin, pack};
use autumn_web::plugin::Plugin;
use autumn_web::plugin_conformance::widgets::run_widget_conformance;

#[test]
fn the_pack_conforms() {
    let report = run_widget_conformance(&pack());
    assert!(report.passed(), "{}", report.to_text_report());
}

#[test]
fn the_pack_has_a_story_for_each_chart_type_and_feature() {
    let pack = pack();
    let slugs: Vec<&str> = pack
        .stories()
        .iter()
        .map(autumn_web::stories::Story::slug)
        .collect();
    for slug in [
        "charts-line",
        "charts-bar",
        "charts-bar-stacked",
        "charts-bar-negative",
        "charts-area",
        "charts-area-stacked",
        "charts-donut",
        "charts-hover-details",
        "charts-data-table",
        "charts-empty",
    ] {
        assert!(slugs.contains(&slug), "no story `{slug}` in {slugs:?}");
    }
}

#[test]
fn the_pack_links_its_css_and_no_script() {
    let pack = pack();
    assert_eq!(pack.name(), "charts");
    assert_eq!(ASSETS.namespace(), "charts");
    let head = pack.head().into_string();
    assert!(head.contains("/charts/charts."), "{head}");
    assert!(head.contains(".css"), "{head}");
    assert!(head.contains("integrity="), "{head}");
    assert!(!head.contains("<script"), "{head}");
}

#[test]
fn the_pack_has_no_script_asset() {
    assert!(
        ASSETS
            .iter()
            .all(|asset| !asset.logical_path().to_ascii_lowercase().ends_with(".js")),
        "the charts need no script"
    );
}

#[test]
fn no_story_writes_a_script_or_a_style() {
    for story in pack().stories() {
        let html = story.render().unwrap().into_string();
        assert!(!html.contains("<script"), "{}", story.slug());
        assert!(!html.contains("style="), "{}", story.slug());
    }
}

#[test]
fn the_plugin_has_a_name() {
    assert_eq!(ChartsPlugin.name(), "autumn-charts-plugin");
}

#[test]
fn the_plugin_installs_on_an_app() {
    let _app = autumn_web::app().plugin(ChartsPlugin);
}
