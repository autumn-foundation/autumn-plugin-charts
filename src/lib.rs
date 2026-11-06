//! Charts v2: a widget pack for `autumn-web`.

mod cartesian;
mod chart;
mod donut;
pub mod scale;
mod series;
mod stories;

use autumn_web::app::AppBuilder;
use autumn_web::assets::PluginAssets;
use autumn_web::plugin::Plugin;
use autumn_web::widget_pack::WidgetPack;

pub use cartesian::{AreaChart, BarChart, LineChart};
pub use chart::{Formatter, TableMode};
pub use donut::DonutChart;
pub use series::Series;

/// The pack CSS.
pub static ASSETS: PluginAssets =
    PluginAssets::from_files("charts", &[("charts.css", include_bytes!("charts.css"))]);

/// The pack: its stories and its assets.
#[must_use]
pub fn pack() -> WidgetPack {
    WidgetPack::new("charts").assets(&ASSETS)
}

/// The plugin. It installs [`pack`].
#[derive(Debug, Default, Clone, Copy)]
pub struct ChartsPlugin;

impl Plugin for ChartsPlugin {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("")
    }

    fn build(self, app: AppBuilder) -> AppBuilder {
        app
    }
}
