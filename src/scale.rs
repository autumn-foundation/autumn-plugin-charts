//! Scales, ticks and number labels. Pure math, no markup.

/// The most ticks [`nice_ticks`] returns.
pub const MAX_TICKS: usize = 24;

/// Ticks for one axis.
#[derive(Debug, Clone, PartialEq)]
pub struct Ticks {
    values: Vec<f64>,
    step: f64,
}

impl Ticks {
    /// The tick values, in ascending order.
    #[must_use]
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// The step between two ticks.
    #[must_use]
    pub const fn step(&self) -> f64 {
        self.step
    }
}

/// Make about `count` "nice" ticks that cover `min..=max`.
#[must_use]
pub fn nice_ticks(_min: f64, _max: f64, _count: usize) -> Ticks {
    Ticks {
        values: Vec::new(),
        step: 0.0,
    }
}

/// The value domain of `values`.
#[must_use]
pub fn domain(_values: impl IntoIterator<Item = f64>, _include_zero: bool) -> (f64, f64) {
    (0.0, 0.0)
}

/// A linear map from a value domain to a pixel range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearScale {
    domain: (f64, f64),
    range: (f64, f64),
}

impl LinearScale {
    /// A scale from `domain` to `range`.
    #[must_use]
    pub const fn new(domain: (f64, f64), range: (f64, f64)) -> Self {
        Self { domain, range }
    }

    /// Map `value` into the range.
    #[must_use]
    pub const fn map(&self, _value: f64) -> f64 {
        0.0
    }
}

/// The default number label.
#[must_use]
pub fn format_number(_value: f64) -> String {
    String::new()
}

/// A tick label, rounded to the precision of `step`.
#[must_use]
pub fn format_tick(_value: f64, _step: f64) -> String {
    String::new()
}
