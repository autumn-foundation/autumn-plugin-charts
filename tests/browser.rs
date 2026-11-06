//! Browser tests: the plugin conformance gate (axe in both schemes,
//! console, keyboard) and the focusable mark tip.
//!
//! Run:
//!
//! ```text
//! AUTUMN_CHROMIUM=/path/to/chrome \
//!   cargo test --features system-tests --test browser -- --include-ignored
//! ```

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test fails on a panic"
)]
#![cfg(feature = "system-tests")]

use std::time::Duration;

use autumn_charts_plugin::{ASSETS, BarChart, DonutChart, Series, pack};
use autumn_web::plugin_conformance::widgets::browser::run_widget_browser_conformance;
use autumn_web::plugin_conformance::widgets::run_widget_conformance;
use autumn_web::prelude::*;
use autumn_web::system_test::{Page, SystemTest, SystemTestRunner};
use maud::{DOCTYPE, Markup, html};

const QUARTERS: [&str; 3] = ["Q1", "Q2", "Q3"];
const SALES: [f64; 3] = [12.0, 19.0, 7.0];

fn layout(body: &Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                title { "Charts" }
                link rel="stylesheet" href="/static/css/autumn-widgets.css";
                (pack().head())
            }
            body {
                main {
                    button id="start" type="button" { "Start" }
                    (body)
                }
            }
        }
    }
}

#[get("/bars")]
async fn bars() -> Markup {
    let series = [Series::new("Sales", &SALES)];
    layout(
        &BarChart::new("Sales", &QUARTERS, &series)
            .focusable()
            .render(),
    )
}

#[get("/donut")]
async fn donut() -> Markup {
    let slices = [("Rust", 45.0), ("Go", 30.0), ("Zig", 25.0)];
    layout(&DonutChart::new("Share", &slices).focusable().render())
}

fn axe_source() -> String {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/axe-core/axe.min.js");
    std::fs::read_to_string(path).expect("axe-core fixture")
}

async fn runner() -> SystemTestRunner {
    let mut config = autumn_web::config::AutumnConfig::default();
    config.security.csrf.enabled = false;
    let state = autumn_web::AppState::for_test().with_profile("test");
    state.insert_extension(config);
    Box::pin(
        SystemTest::new()
            .state(state)
            .routes(routes![bars, donut])
            .plugin_assets(&ASSETS)
            .build(),
    )
    .await
    .expect("start runner")
}

async fn eval<T: serde::de::DeserializeOwned + Default>(page: &Page, js: &str) -> T {
    page.evaluate(js)
        .await
        .ok()
        .and_then(|r| r.into_value::<T>().ok())
        .unwrap_or_default()
}

async fn poll(page: &Page, js: &str) -> bool {
    for _ in 0..100 {
        if eval::<bool>(page, js).await {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    false
}

async fn open(page: &Page, path: &str) {
    page.visit(path).await.expect("visit");
    assert!(
        poll(page, "document.readyState === 'complete'").await,
        "page did not load"
    );
    assert!(
        poll(
            page,
            "[...document.styleSheets].some(s => (s.href || '').includes('charts'))"
        )
        .await,
        "the pack CSS did not load"
    );
}

/// The visibility of each tip, in mark order.
const TIPS_JS: &str = r"JSON.stringify([...document.querySelectorAll('.autumn-charts-tip')]
  .map(t => getComputedStyle(t).visibility))";

/// The `aria-label` of the focused element.
const ACTIVE_JS: &str =
    r"(document.activeElement && document.activeElement.getAttribute('aria-label')) || ''";

#[tokio::test]
#[ignore = "requires Chromium"]
async fn the_pack_passes_the_browser_gate() {
    let pack = pack();
    let report = run_widget_conformance(&pack);
    assert!(report.passed(), "{}", report.to_text_report());
    let report = run_widget_browser_conformance(&pack, &axe_source())
        .await
        .expect("browser conformance runs");
    assert!(report.passed(), "{}", report.to_text_report());
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn tab_moves_through_the_bars_and_shows_each_tip() {
    let runner = runner().await;
    let page = runner.page().await.expect("page");
    open(&page, "/bars").await;
    assert_eq!(
        eval::<String>(&page, TIPS_JS).await,
        r#"["hidden","hidden","hidden"]"#,
        "no tip shows before focus"
    );
    page.evaluate("document.getElementById('start').focus()")
        .await
        .expect("focus start");
    for (i, label) in ["Sales, Q1: 12", "Sales, Q2: 19", "Sales, Q3: 7"]
        .iter()
        .enumerate()
    {
        page.press("Tab").await.expect("tab");
        assert_eq!(eval::<String>(&page, ACTIVE_JS).await, *label);
        assert_eq!(
            eval::<String>(
                &page,
                "getComputedStyle(document.activeElement).outlineStyle"
            )
            .await,
            "solid",
            "the focused mark has an outline"
        );
        let tips: Vec<String> =
            serde_json::from_str(&eval::<String>(&page, TIPS_JS).await).expect("tips");
        for (j, tip) in tips.iter().enumerate() {
            let want = if i == j { "visible" } else { "hidden" };
            assert_eq!(tip, want, "tip {j} with mark {i} focused");
        }
    }
    page.press("Tab").await.expect("tab");
    assert_eq!(
        eval::<String>(&page, TIPS_JS).await,
        r#"["hidden","hidden","hidden"]"#,
        "focus leaves the chart"
    );
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn a_focused_donut_slice_replaces_the_total() {
    let runner = runner().await;
    let page = runner.page().await.expect("page");
    open(&page, "/donut").await;
    let center = "getComputedStyle(document.querySelector('.autumn-charts-center')).visibility";
    assert_eq!(eval::<String>(&page, center).await, "visible");
    page.evaluate("document.getElementById('start').focus()")
        .await
        .expect("focus start");
    page.press("Tab").await.expect("tab");
    assert_eq!(eval::<String>(&page, ACTIVE_JS).await, "Rust: 45 (45%)");
    assert_eq!(eval::<String>(&page, center).await, "hidden");
    assert_eq!(
        eval::<String>(&page, TIPS_JS).await,
        r#"["visible","hidden","hidden"]"#
    );
    let violations = page.axe_violations(&axe_source()).await.expect("axe");
    assert!(violations.is_empty(), "{violations:?}");
}
