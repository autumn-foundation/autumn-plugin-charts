//! Chart data.

/// One named series: one value for each category.
///
/// A non-finite value (`NaN`, `inf`) is a missing value.
///
/// ```rust
/// use autumn_charts_plugin::Series;
///
/// let series = Series::new("Visits", &[120.0, f64::NAN]);
/// assert_eq!(series.name(), "Visits");
/// assert_eq!(series.values().len(), 2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Series<'a> {
    name: &'a str,
    values: &'a [f64],
}

impl<'a> Series<'a> {
    /// A series.
    #[must_use]
    pub const fn new(name: &'a str, values: &'a [f64]) -> Self {
        Self { name, values }
    }

    /// The series name.
    #[must_use]
    pub const fn name(&self) -> &'a str {
        self.name
    }

    /// The raw values.
    #[must_use]
    pub const fn values(&self) -> &'a [f64] {
        self.values
    }
}
