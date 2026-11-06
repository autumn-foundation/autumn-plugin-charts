//! Donut charts.

use maud::{Markup, Render, html};

use crate::chart::{Common, common_setters};

/// A donut chart.
#[derive(Debug, Clone)]
pub struct DonutChart<'a> {
    common: Common<'a>,
    slices: &'a [(&'a str, f64)],
    center_label: Option<&'a str>,
}

impl<'a> DonutChart<'a> {
    /// A donut of `slices`.
    #[must_use]
    pub const fn new(title: &'a str, slices: &'a [(&'a str, f64)]) -> Self {
        Self {
            common: Common::new(title),
            slices,
            center_label: None,
        }
    }

    common_setters!();

    /// Write a label under the total in the centre.
    #[must_use]
    pub const fn center_label(mut self, label: &'a str) -> Self {
        self.center_label = Some(label);
        self
    }

    /// Write the chart.
    #[must_use]
    pub fn render(&self) -> Markup {
        let _ = (&self.common, self.slices, self.center_label);
        html! {}
    }
}

impl Render for DonutChart<'_> {
    fn render(&self) -> Markup {
        DonutChart::render(self)
    }
}
