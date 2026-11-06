//! Options and parts that all charts share.

use std::fmt;

use autumn_web::form::FieldOptions;

use crate::svg::{marker, num, style_class};

/// A number formatter for ticks, tips, the legend and the data table.
///
/// It can capture, for example a locale that the app reads for each
/// request.
pub type Formatter<'a> = &'a dyn Fn(f64) -> String;

/// How the data table shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum TableMode {
    /// Visually hidden. Screen readers read it.
    #[default]
    Hidden,
    /// In a `<details>` below the chart.
    Details,
    /// Shown below the chart.
    Visible,
}

/// The fixed words in the data table. The default is English.
///
/// ```rust
/// use autumn_charts_plugin::Words;
///
/// let words = Words::new().category("Monat").no_value("Kein Wert");
/// assert_eq!(words.no_value_text(), "Kein Wert");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Words<'a> {
    pub(crate) category: &'a str,
    pub(crate) value: &'a str,
    pub(crate) share: &'a str,
    pub(crate) no_value: &'a str,
}

impl Default for Words<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Words<'a> {
    /// The English words.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            category: "Category",
            value: "Value",
            share: "Share",
            no_value: "No value",
        }
    }

    /// Set the header of the category column. An `x_label` replaces it.
    #[must_use]
    pub const fn category(mut self, text: &'a str) -> Self {
        self.category = text;
        self
    }

    /// Set the header of the donut value column.
    #[must_use]
    pub const fn value(mut self, text: &'a str) -> Self {
        self.value = text;
        self
    }

    /// Set the header of the donut share column.
    #[must_use]
    pub const fn share(mut self, text: &'a str) -> Self {
        self.share = text;
        self
    }

    /// Set the text of a cell that has no value.
    #[must_use]
    pub const fn no_value(mut self, text: &'a str) -> Self {
        self.no_value = text;
        self
    }

    /// The text of a cell that has no value.
    #[must_use]
    pub const fn no_value_text(&self) -> &'a str {
        self.no_value
    }
}

/// The options that all charts share.
#[derive(Clone)]
pub struct Common<'a> {
    pub title: &'a str,
    pub desc: Option<&'a str>,
    pub options: Option<&'a FieldOptions<'a>>,
    pub class: Option<&'a str>,
    pub format: Option<Formatter<'a>>,
    pub legend: Option<bool>,
    pub focusable: bool,
    pub table: TableMode,
    pub table_label: &'a str,
    pub hide_title: bool,
    pub words: Words<'a>,
}

impl fmt::Debug for Common<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Common")
            .field("title", &self.title)
            .field("desc", &self.desc)
            .field("class", &self.class)
            .field("format", &self.format.map(|_| "fn"))
            .field("legend", &self.legend)
            .field("focusable", &self.focusable)
            .field("table", &self.table)
            .field("hide_title", &self.hide_title)
            .field("words", &self.words)
            .finish_non_exhaustive()
    }
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
            words: Words::new(),
        }
    }
}

/// The setters that all charts share. The type has a `common` field.
macro_rules! common_setters {
    () => {
        /// Set the summary that names the SVG for screen readers. The
        /// default gives the chart type, the counts and the value range.
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

        /// Set one more class on the root. A second call replaces the
        /// first.
        #[must_use]
        pub const fn class(mut self, class: &'a str) -> Self {
            self.common.class = Some(class);
            self
        }

        /// Format the numbers in ticks, tips, the legend, the summary and
        /// the table.
        #[must_use]
        pub const fn format(mut self, format: $crate::chart::Formatter<'a>) -> Self {
            self.common.format = Some(format);
            self
        }

        /// Show or hide the legend.
        #[must_use]
        pub const fn legend(mut self, show: bool) -> Self {
            self.common.legend = Some(show);
            self
        }

        /// Make each mark a tab stop that shows its value. Each mark adds
        /// one tab stop, so use it for charts with few marks.
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

        /// Set the fixed words in the data table.
        #[must_use]
        pub const fn words(mut self, words: $crate::chart::Words<'a>) -> Self {
            self.common.words = words;
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

/// What a legend swatch shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Swatch {
    /// A line and a marker.
    Line,
    /// A filled block with a marker cut out.
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

    /// The `<svg>`. The `<figcaption>` names the figure, so the summary
    /// names the SVG. With focusable marks, the SVG is a `group`: an
    /// `img` has presentational children.
    pub fn svg(&self, view_box: &str, summary: &str, body: &maud::Markup) -> maud::Markup {
        let summary = self.desc.unwrap_or(summary);
        maud::html! {
            @if self.focusable {
                svg class="autumn-charts-svg" viewBox=(view_box) role="group"
                    aria-label=(summary) { (body) }
            } @else {
                svg class="autumn-charts-svg" viewBox=(view_box) role="img"
                    aria-label=(summary) focusable="false" { (body) }
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
                    text class="autumn-charts-tip" x=(num(tip.x))
                        y=(num(tip.y)) text-anchor=(tip.anchor) { (label) }
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
                        svg class={ "autumn-charts-swatch " (style_class(*k)) }
                            viewBox="0 0 24 12" aria-hidden="true" focusable="false" {
                            @match swatch {
                                Swatch::Line => {
                                    path class="autumn-charts-swatch-line" d="M1,6H23" {}
                                    path class="autumn-charts-swatch-marker"
                                        d=(marker(*k, 12.0, 6.0, 3.0)) {}
                                }
                                Swatch::Fill => {
                                    path class="autumn-charts-swatch-fill" d="M2,1H22V11H2Z" {}
                                    path class="autumn-charts-swatch-glyph"
                                        d=(marker(*k, 12.0, 6.0, 2.5)) {}
                                }
                            }
                        }
                        (text)
                    }
                }
            }
        }
    }

    /// The data table, placed by [`TableMode`]. `unit` follows the title
    /// in the caption.
    pub fn table(
        &self,
        head: &[&str],
        rows: &[(&str, Vec<String>)],
        unit: Option<&str>,
    ) -> maud::Markup {
        let class = if self.table == TableMode::Hidden {
            "autumn-charts-table autumn-visually-hidden"
        } else {
            "autumn-charts-table"
        };
        let table = maud::html! {
            table class=(class) {
                caption {
                    (self.title)
                    @if let Some(unit) = unit { ": " (unit) }
                }
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
