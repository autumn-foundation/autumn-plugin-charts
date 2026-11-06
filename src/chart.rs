//! Options that all charts share.

use autumn_web::form::FieldOptions;

/// A number formatter for ticks, tips and the data table.
pub type Formatter = fn(f64) -> String;

/// How the data table shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TableMode {
    /// Visually hidden. Screen readers read it.
    #[default]
    Hidden,
    /// In a `<details>` below the chart.
    Details,
    /// Shown below the chart.
    Visible,
}

/// The options that all charts share.
#[derive(Debug, Clone)]
pub struct Common<'a> {
    pub title: &'a str,
    pub desc: Option<&'a str>,
    pub options: Option<&'a FieldOptions<'a>>,
    pub class: Option<&'a str>,
    pub format: Option<Formatter>,
    pub legend: Option<bool>,
    pub focusable: bool,
    pub table: TableMode,
    pub table_label: &'a str,
    pub hide_title: bool,
}

impl<'a> Common<'a> {
    pub const fn new(title: &'a str) -> Self {
        Self {
            title,
            desc: None,
            options: None,
            class: None,
            format: None,
            legend: None,
            focusable: false,
            table: TableMode::Hidden,
            table_label: "Show the data",
            hide_title: false,
        }
    }
}

/// The setters that all charts share. The type has a `common` field.
macro_rules! common_setters {
    () => {
        /// Set the long description (`<desc>`). The default sums up the
        /// data.
        #[must_use]
        pub const fn desc(mut self, desc: &'a str) -> Self {
            self.common.desc = Some(desc);
            self
        }

        /// Add the app `data-*` and `hx-*` passthrough to the root.
        #[must_use]
        pub const fn options(mut self, options: &'a autumn_web::form::FieldOptions<'a>) -> Self {
            self.common.options = Some(options);
            self
        }

        /// Add a class to the root.
        #[must_use]
        pub const fn class(mut self, class: &'a str) -> Self {
            self.common.class = Some(class);
            self
        }

        /// Format the numbers in ticks, tips and the table.
        #[must_use]
        pub const fn format(mut self, format: $crate::chart::Formatter) -> Self {
            self.common.format = Some(format);
            self
        }

        /// Show or hide the legend.
        #[must_use]
        pub const fn legend(mut self, show: bool) -> Self {
            self.common.legend = Some(show);
            self
        }

        /// Let each mark take focus and show its value.
        #[must_use]
        pub const fn focusable(mut self) -> Self {
            self.common.focusable = true;
            self
        }

        /// Set how the data table shows.
        #[must_use]
        pub const fn table(mut self, mode: $crate::chart::TableMode) -> Self {
            self.common.table = mode;
            self
        }

        /// Set the `<summary>` text for [`TableMode::Details`](crate::TableMode::Details).
        #[must_use]
        pub const fn table_label(mut self, label: &'a str) -> Self {
            self.common.table_label = label;
            self
        }

        /// Hide the caption visually. Screen readers still read it.
        #[must_use]
        pub const fn hide_title(mut self) -> Self {
            self.common.hide_title = true;
            self
        }
    };
}

pub(crate) use common_setters;

/// The text for a cell with no value.
pub const NO_VALUE: &str = "No value";

/// What a legend swatch shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Swatch {
    /// A line and a marker.
    Line,
    /// A filled block.
    Fill,
}

/// The tip of a focusable mark: where its text goes.
#[derive(Debug, Clone, Copy)]
pub struct Tip {
    pub x: f64,
    pub y: f64,
    pub anchor: &'static str,
}

impl Common<'_> {
    /// `value` as text, with the app formatter if it has one.
    pub fn text(&self, value: f64) -> String {
        self.format
            .map_or_else(|| crate::scale::format_number(value), |f| f(value))
    }

    /// `true` when the legend shows for `count` entries.
    pub fn shows_legend(&self, count: usize, default: bool) -> bool {
        self.legend.unwrap_or(default) && count > 0
    }

    /// The `<figure>` root: the caption, then `body`.
    pub fn root(&self, kind: &str, dense: bool, body: &maud::Markup) -> maud::Markup {
        let mut class = format!("autumn-charts-chart autumn-charts-chart--{kind}");
        if dense {
            class.push_str(" autumn-charts-chart--dense");
        }
        if let Some(extra) = self.class.filter(|c| !c.trim().is_empty()) {
            class.push(' ');
            class.push_str(extra.trim());
        }
        let caption = if self.hide_title {
            "autumn-charts-title autumn-visually-hidden"
        } else {
            "autumn-charts-title"
        };
        let none = FieldOptions::new();
        autumn_web::widget_pack::element(
            "figure",
            &[("class", &class)],
            self.options.unwrap_or(&none),
            maud::html! {
                figcaption class=(caption) { (self.title) }
                (body)
            },
        )
    }

    /// The `<svg>`: `role="img"`, or `role="group"` with focusable marks.
    pub fn svg(&self, view_box: &str, desc: &str, body: &maud::Markup) -> maud::Markup {
        let desc = self.desc.unwrap_or(desc);
        maud::html! {
            @if self.focusable {
                svg class="autumn-charts-svg" viewBox=(view_box) role="group"
                    aria-label=(self.title) {
                    title { (self.title) }
                    desc { (desc) }
                    (body)
                }
            } @else {
                svg class="autumn-charts-svg" viewBox=(view_box) role="img"
                    aria-label=(self.title) focusable="false" {
                    title { (self.title) }
                    desc { (desc) }
                    (body)
                }
            }
        }
    }

    /// One mark: `shape`, and its `label` as a `<title>`, or as an
    /// `aria-label` and a tip when marks take focus.
    pub fn mark(&self, label: &str, shape: &maud::Markup, tip: Tip) -> maud::Markup {
        maud::html! {
            @if self.focusable {
                g class="autumn-charts-mark" tabindex="0" role="img" aria-label=(label) {
                    (shape)
                    text class="autumn-charts-tip" x=(crate::svg::num(tip.x))
                        y=(crate::svg::num(tip.y)) text-anchor=(tip.anchor) { (label) }
                }
            } @else {
                g class="autumn-charts-mark" {
                    title { (label) }
                    (shape)
                }
            }
        }
    }

    /// The legend: one item for each `(style, text)`.
    pub fn legend(swatch: Swatch, items: &[(usize, String)]) -> maud::Markup {
        maud::html! {
            ul class="autumn-charts-legend" {
                @for (k, text) in items {
                    li class="autumn-charts-legend-item" {
                        svg class={ "autumn-charts-swatch " (crate::svg::style_class(*k)) }
                            viewBox="0 0 24 12" aria-hidden="true" focusable="false" {
                            @match swatch {
                                Swatch::Line => {
                                    path class="autumn-charts-swatch-line" d="M1,6H23" {}
                                    path class="autumn-charts-swatch-marker"
                                        d=(crate::svg::marker(*k, 12.0, 6.0, 3.0)) {}
                                }
                                Swatch::Fill => {
                                    path class="autumn-charts-swatch-fill" d="M2,1H22V11H2Z" {}
                                }
                            }
                        }
                        (text)
                    }
                }
            }
        }
    }

    /// The data table, placed by [`TableMode`].
    pub fn table(&self, head: &[&str], rows: &[(&str, Vec<String>)]) -> maud::Markup {
        let class = if self.table == TableMode::Hidden {
            "autumn-charts-table autumn-visually-hidden"
        } else {
            "autumn-charts-table"
        };
        let table = maud::html! {
            table class=(class) {
                caption { (self.title) }
                thead { tr { @for h in head { th scope="col" { (h) } } } }
                tbody {
                    @for (label, cells) in rows {
                        tr {
                            th scope="row" { (label) }
                            @for cell in cells { td { (cell) } }
                        }
                    }
                }
            }
        };
        maud::html! {
            @if self.table == TableMode::Details {
                details class="autumn-charts-data" {
                    summary { (self.table_label) }
                    (table)
                }
            } @else {
                (table)
            }
        }
    }
}
