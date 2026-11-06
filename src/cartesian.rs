//! Line, bar and area charts: two axes, a grid and a legend.

use maud::{Markup, Render, html};

use crate::chart::{Common, common_setters};
use crate::series::Series;

#[derive(Debug, Clone)]
struct Cartesian<'a> {
    common: Common<'a>,
    categories: &'a [&'a str],
    series: &'a [Series<'a>],
    x_label: Option<&'a str>,
    y_label: Option<&'a str>,
    min: Option<f64>,
    max: Option<f64>,
    ticks: usize,
    zero: bool,
    stacked: bool,
}

macro_rules! cartesian_chart {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone)]
        pub struct $name<'a> {
            common: Common<'a>,
            inner: Cartesian<'a>,
        }

        impl<'a> $name<'a> {
            /// A chart of `series` over `categories`.
            #[must_use]
            pub const fn new(
                title: &'a str,
                categories: &'a [&'a str],
                series: &'a [Series<'a>],
            ) -> Self {
                Self {
                    common: Common::new(title),
                    inner: Cartesian {
                        common: Common::new(title),
                        categories,
                        series,
                        x_label: None,
                        y_label: None,
                        min: None,
                        max: None,
                        ticks: 5,
                        zero: false,
                        stacked: false,
                    },
                }
            }

            common_setters!();

            /// Set the x axis title.
            #[must_use]
            pub const fn x_label(mut self, label: &'a str) -> Self {
                self.inner.x_label = Some(label);
                self
            }

            /// Set the y axis title.
            #[must_use]
            pub const fn y_label(mut self, label: &'a str) -> Self {
                self.inner.y_label = Some(label);
                self
            }

            /// Set the y axis minimum.
            #[must_use]
            pub const fn min(mut self, min: f64) -> Self {
                self.inner.min = Some(min);
                self
            }

            /// Set the y axis maximum.
            #[must_use]
            pub const fn max(mut self, max: f64) -> Self {
                self.inner.max = Some(max);
                self
            }

            /// Set about how many y ticks to show.
            #[must_use]
            pub const fn ticks(mut self, count: usize) -> Self {
                self.inner.ticks = count;
                self
            }

            /// Write the chart.
            #[must_use]
            pub fn render(&self) -> Markup {
                let _ = (&self.common, &self.inner);
                html! {}
            }
        }

        impl Render for $name<'_> {
            fn render(&self) -> Markup {
                $name::render(self)
            }
        }
    };
}

cartesian_chart! {
    /// A line chart.
    LineChart
}
cartesian_chart! {
    /// A bar chart.
    BarChart
}
cartesian_chart! {
    /// An area chart.
    AreaChart
}

impl LineChart<'_> {
    /// Put zero in the y domain.
    #[must_use]
    pub const fn zero_baseline(mut self) -> Self {
        self.inner.zero = true;
        self
    }
}

impl BarChart<'_> {
    /// Stack the series.
    #[must_use]
    pub const fn stacked(mut self) -> Self {
        self.inner.stacked = true;
        self
    }
}

impl AreaChart<'_> {
    /// Stack the series.
    #[must_use]
    pub const fn stacked(mut self) -> Self {
        self.inner.stacked = true;
        self
    }
}
