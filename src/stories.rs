//! One story for each chart type and feature.

use autumn_web::stories::{Story, story};
use maud::{Markup, html};

use crate::{AreaChart, BarChart, DonutChart, LineChart, Series, TableMode};

const MONTHS: [&str; 6] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];
const WEB: [f64; 6] = [120.0, 132.0, 101.0, 134.0, 90.0, 230.0];
const MOBILE: [f64; 6] = [220.0, 182.0, 191.0, 234.0, 290.0, 330.0];
const DESKTOP: [f64; 6] = [150.0, 232.0, 201.0, 154.0, 190.0, 330.0];
const NET: [f64; 6] = [-40.0, 25.0, 60.0, -15.0, f64::NAN, 80.0];

const fn traffic() -> [Series<'static>; 3] {
    [
        Series::new("Web", &WEB),
        Series::new("Mobile", &MOBILE),
        Series::new("Desktop", &DESKTOP),
    ]
}

fn line() -> Markup {
    let series = traffic();
    LineChart::new("Visits by device", &MONTHS, &series)
        .x_label("Month")
        .y_label("Visits")
        .render()
}

fn bar() -> Markup {
    let series = traffic();
    BarChart::new("Visits by device", &MONTHS, &series).render()
}

fn bar_stacked() -> Markup {
    let series = traffic();
    BarChart::new("Visits by device, stacked", &MONTHS, &series)
        .stacked()
        .render()
}

fn bar_negative() -> Markup {
    let series = [Series::new("Net change", &NET)];
    BarChart::new("Net change", &MONTHS, &series)
        .desc("Net change by month. May has no value.")
        .render()
}

fn area() -> Markup {
    let series = [Series::new("Web", &WEB), Series::new("Mobile", &MOBILE)];
    AreaChart::new("Visits, web and mobile", &MONTHS, &series).render()
}

fn area_stacked() -> Markup {
    let series = traffic();
    AreaChart::new("Visits by device, stacked", &MONTHS, &series)
        .stacked()
        .render()
}

fn donut() -> Markup {
    DonutChart::new(
        "Browser share",
        &[
            ("Chrome", 64.0),
            ("Safari", 19.0),
            ("Firefox", 9.0),
            ("Other", 8.0),
        ],
    )
    .center_label("Percent")
    .render()
}

fn hover_details() -> Markup {
    let series = [Series::new("Mobile", &MOBILE)];
    let area = [Series::new("Web", &WEB), Series::new("Desktop", &DESKTOP)];
    html! {
        (BarChart::new("Mobile visits", &MONTHS, &series).focusable().render())
        (AreaChart::new("Web and desktop visits", &MONTHS, &area).focusable().render())
        (DonutChart::new("Plan mix", &[("Free", 120.0), ("Team", 45.0), ("Business", 12.0)])
            .focusable()
            .center_label("Accounts")
            .render())
    }
}

fn data_table() -> Markup {
    let series = [Series::new("Web", &WEB), Series::new("Mobile", &MOBILE)];
    html! {
        (LineChart::new("Visits, table in a disclosure", &MONTHS, &series)
            .table(TableMode::Details)
            .render())
        (BarChart::new("Visits, table shown", &MONTHS, &series)
            .table(TableMode::Visible)
            .render())
    }
}

fn format() -> Markup {
    let rate = [0.12, 0.18, 0.15, 0.22, 0.31, 0.27];
    let series = [Series::new("Conversion", &rate)];
    LineChart::new("Conversion rate", &MONTHS, &series)
        .zero_baseline()
        .format(&|v| format!("{:.0}%", v * 100.0))
        .render()
}

fn empty() -> Markup {
    let series: [Series<'_>; 0] = [];
    html! {
        (LineChart::new("No visits yet", &[], &series).render())
        (DonutChart::new("No plans yet", &[]).render())
    }
}

/// The pack stories.
pub fn all() -> Vec<Story> {
    vec![
        story! { "Charts", "Charts line", { line() } },
        story! { "Charts", "Charts bar", { bar() } },
        story! { "Charts", "Charts bar stacked", { bar_stacked() } },
        story! { "Charts", "Charts bar negative", { bar_negative() } },
        story! { "Charts", "Charts area", { area() } },
        story! { "Charts", "Charts area stacked", { area_stacked() } },
        story! { "Charts", "Charts donut", { donut() } },
        story! { "Charts", "Charts hover details", { hover_details() } },
        story! { "Charts", "Charts data table", { data_table() } },
        story! { "Charts", "Charts format", { format() } },
        story! { "Charts", "Charts empty", { empty() } },
    ]
}
