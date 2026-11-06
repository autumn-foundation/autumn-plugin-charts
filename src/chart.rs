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
