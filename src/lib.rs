//! Charts v2: a widget pack for `autumn-web`.
//!
//! The server writes each chart as SVG. There is no script. The pack adds
//! to the core `sparkline`, `bar_chart` and `line_chart`:
//!
//! - axes, nice ticks and axis titles,
//! - legends,
//! - many series in line, bar and area charts,
//! - stacked bars and stacked areas,
//! - donut charts.
//!
//! Each chart writes a data table: the text alternative. Each mark has a
//! `<title>`. With `focusable`, each mark is a tab stop with an
//! `aria-label` and a tip.
//!
//! # Install
//!
//! ```rust
//! use autumn_charts_plugin::ChartsPlugin;
//!
//! fn app() -> autumn_web::app::AppBuilder {
//!     autumn_web::app().plugin(ChartsPlugin)
//! }
//! ```
//!
//! In the layout `<head>`, after the core widget CSS:
//!
//! ```rust
//! let head = autumn_charts_plugin::pack().head().into_string();
//! assert!(head.contains("charts"));
//! ```
//!
//! # Use
//!
//! ```rust
//! use autumn_charts_plugin::{BarChart, DonutChart, LineChart, Series, TableMode};
//!
//! let months = ["Jan", "Feb", "Mar"];
//! let web = [120.0, 132.0, 101.0];
//! let mobile = [220.0, 182.0, f64::NAN]; // NaN is a missing value.
//! let series = [Series::new("Web", &web), Series::new("Mobile", &mobile)];
//!
//! let line = LineChart::new("Visits", &months, &series)
//!     .x_label("Month")
//!     .y_label("Visits")
//!     .render()
//!     .into_string();
//! assert!(line.contains("autumn-charts-legend"));
//! assert!(!line.contains("<script"));
//!
//! let bars = BarChart::new("Visits", &months, &series)
//!     .stacked()
//!     .focusable()
//!     .table(TableMode::Details)
//!     .render()
//!     .into_string();
//! assert!(bars.contains(r#"tabindex="0""#));
//!
//! let donut = DonutChart::new("Plans", &[("Free", 120.0), ("Team", 45.0)])
//!     .center_label("Accounts")
//!     .render()
//!     .into_string();
//! assert!(donut.contains("Free: 120"));
//! ```

mod cartesian;
mod chart;
mod donut;
pub mod scale;
mod series;
mod stories;
mod svg;

use autumn_web::app::AppBuilder;
use autumn_web::assets::PluginAssets;
use autumn_web::plugin::Plugin;
use autumn_web::widget_pack::WidgetPack;

pub use cartesian::{AreaChart, BarChart, LineChart};
pub use chart::{Formatter, TableMode, Words};
pub use donut::DonutChart;
pub use series::Series;

/// The pack CSS.
pub static ASSETS: PluginAssets =
    PluginAssets::from_files("charts", &[("charts.css", include_bytes!("charts.css"))]);

/// The pack: its stories and its assets.
#[must_use]
pub fn pack() -> WidgetPack {
    WidgetPack::new("charts")
        .extend(stories::all())
        .assets(&ASSETS)
}

/// The plugin. It installs [`pack`].
#[derive(Debug, Default, Clone, Copy)]
pub struct ChartsPlugin;

impl Plugin for ChartsPlugin {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(env!("CARGO_PKG_NAME"))
    }

    fn build(self, app: AppBuilder) -> AppBuilder {
        app.widget_pack(pack())
    }
}
