//! Scales, ticks and number labels. This module has only math. It writes
//! no markup. The API is stable: a change to an output is a breaking
//! change.
//!
//! Each function takes any `f64`, `NaN` and the infinities too. Each output
//! is finite. `tests/scale.rs` checks this with proptests.

/// The most ticks [`nice_ticks`] returns.
pub const MAX_TICKS: usize = 24;

/// The largest tick count that [`nice_ticks`] tries for.
const MAX_TARGET: usize = MAX_TICKS - 6;

/// Above this, an `f64` does not hold each integer. A tick index past it is
/// not exact.
const EXACT_INT: f64 = 9_007_199_254_740_992.0;

/// Ticks for one axis.
#[derive(Debug, Clone, PartialEq)]
pub struct Ticks {
    values: Vec<f64>,
    step: f64,
}

impl Ticks {
    /// The tick values. They are finite and strictly ascending.
    #[must_use]
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// The step between two ticks. It is finite and not negative.
    #[must_use]
    pub const fn step(&self) -> f64 {
        self.step
    }

    /// The first and the last tick.
    #[must_use]
    pub fn bounds(&self) -> (f64, f64) {
        let first = self.values.first().copied().unwrap_or(0.0);
        let last = self.values.last().copied().unwrap_or(first);
        (first, last)
    }
}

/// Make about `count` "nice" ticks (1, 2 or 5 × 10ⁿ apart) that cover
/// `min..=max`.
///
/// The ends can be in any order. A non-finite end gives `0..=1`. Equal ends
/// give one tick. `count` clamps to `1..=18`. The result has 1 to
/// [`MAX_TICKS`] values.
///
/// ```rust
/// use autumn_charts_plugin::scale::nice_ticks;
///
/// let ticks = nice_ticks(3.0, 97.0, 5);
/// assert_eq!(ticks.values(), &[0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);
/// assert_eq!(ticks.step(), 20.0);
/// ```
#[must_use]
pub fn nice_ticks(min: f64, max: f64, count: usize) -> Ticks {
    let (lo, hi) = if min.is_finite() && max.is_finite() {
        (min.min(max), min.max(max))
    } else {
        (0.0, 1.0)
    };
    if lo >= hi {
        return Ticks {
            values: vec![zero_sign(lo)],
            step: 0.0,
        };
    }
    nice_steps(lo, hi, count).unwrap_or_else(|| ends(lo, hi))
}

/// The nice ticks, or `None` when the float math cannot make them.
fn nice_steps(lo: f64, hi: f64, count: usize) -> Option<Ticks> {
    #[allow(
        clippy::cast_precision_loss,
        reason = "the count is at most MAX_TARGET"
    )]
    let n = count.clamp(1, MAX_TARGET) as f64;
    // Halves do not overflow for any two finite values.
    let half_span = hi / 2.0 - lo / 2.0;
    let raw_half = half_span / n;
    if !(raw_half.is_finite() && raw_half > 0.0) {
        return None;
    }
    #[allow(
        clippy::cast_possible_truncation,
        reason = "log10 of a finite positive f64 is in -324..=309"
    )]
    let exp = (raw_half.log10() + std::f64::consts::LOG10_2).floor() as i32;
    let unit = pow10(exp);
    if !(unit.is_finite() && unit > 0.0) {
        return None;
    }
    let frac = raw_half / unit * 2.0;
    let nice = [1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .find(|&m| frac <= m * (1.0 + 1e-9))
        .unwrap_or(10.0);
    let step = nice * unit;
    if !(step.is_finite() && step > 0.0) {
        return None;
    }
    let tick = |k: f64| {
        if exp < 0 {
            k * nice / pow10(-exp)
        } else {
            k * nice * unit
        }
    };
    let mut k0 = (lo / step).floor();
    let mut k1 = (hi / step).ceil();
    if !(k0.abs() < EXACT_INT && k1.abs() < EXACT_INT) {
        return None;
    }
    if tick(k0) > lo {
        k0 -= 1.0;
    }
    if tick(k1) < hi {
        k1 += 1.0;
    }
    #[allow(clippy::cast_precision_loss, reason = "MAX_TICKS is small")]
    if k1 - k0 > MAX_TICKS as f64 {
        return None;
    }
    let mut values: Vec<f64> = Vec::with_capacity(MAX_TICKS);
    // `k1 - k0` is at most MAX_TICKS, so 256 steps reach `k1`.
    for k in (0..=u8::MAX)
        .map(|i| k0 + f64::from(i))
        .take_while(|k| *k <= k1)
    {
        let v = zero_sign(tick(k));
        if v.is_finite() && values.last().is_none_or(|last| v > *last) {
            values.push(v);
        }
    }
    // An end tick past `f64::MAX` is not finite. Use the data end.
    if values.first().is_none_or(|first| *first > lo) {
        values.insert(0, lo);
    }
    if values.last().is_none_or(|last| *last < hi) {
        values.push(hi);
    }
    if values.len() < 2 || values.len() > MAX_TICKS {
        return None;
    }
    Some(Ticks { values, step })
}

/// The fallback: one tick at each end.
fn ends(lo: f64, hi: f64) -> Ticks {
    let step = hi / 2.0 - lo / 2.0;
    Ticks {
        values: vec![lo, hi],
        step: if step.is_finite() { step * 2.0 } else { 0.0 }.min(f64::MAX),
    }
}

/// `10^exp`, correctly rounded. `0` below the subnormal range.
fn pow10(exp: i32) -> f64 {
    format!("1e{exp}").parse().unwrap_or(0.0)
}

/// `-0.0` as `0.0`, so a label never shows `-0`.
const fn zero_sign(v: f64) -> f64 {
    if v == 0.0 { 0.0 } else { v }
}

/// The value domain of `values`: the finite minimum and maximum.
///
/// With `include_zero`, the domain holds 0. No finite value gives `0..1`.
/// Equal values get a span around them. The result is finite and
/// `lo < hi`.
#[must_use]
pub fn domain(values: impl IntoIterator<Item = f64>, include_zero: bool) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for v in values.into_iter().filter(|v| v.is_finite()) {
        lo = lo.min(v);
        hi = hi.max(v);
    }
    if lo > hi {
        return (0.0, 1.0);
    }
    if include_zero {
        lo = lo.min(0.0);
        hi = hi.max(0.0);
    }
    if lo < hi {
        return (lo, hi);
    }
    if lo == 0.0 {
        return (0.0, 1.0);
    }
    if lo - 1.0 < lo && lo + 1.0 > lo {
        return (lo - 1.0, lo + 1.0);
    }
    let pad = lo.abs() / 2.0;
    ((lo - pad).max(-f64::MAX), (lo + pad).min(f64::MAX))
}

/// A linear map from a value domain to a pixel range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearScale {
    domain: (f64, f64),
    range: (f64, f64),
}

impl LinearScale {
    /// A scale from `domain` to `range`. The range can go down, for a y
    /// axis.
    #[must_use]
    pub const fn new(domain: (f64, f64), range: (f64, f64)) -> Self {
        Self { domain, range }
    }

    /// Map `value` into the range.
    ///
    /// A value outside the domain clamps to the range end. `NaN` maps to
    /// the range start. A zero or non-finite domain span maps to the
    /// middle. The result is finite when the range is finite.
    #[must_use]
    pub fn map(&self, value: f64) -> f64 {
        let (d0, d1) = self.domain;
        let (r0, r1) = self.range;
        let span = d1 / 2.0 - d0 / 2.0;
        let frac = if value.is_nan() {
            0.0
        } else if span == 0.0 || !span.is_finite() {
            0.5
        } else {
            let f = (value / 2.0 - d0 / 2.0) / span;
            if f.is_nan() { 0.5 } else { f.clamp(0.0, 1.0) }
        };
        r0.mul_add(1.0 - frac, r1 * frac)
    }
}

/// The default number label, with no trailing zeros.
///
/// Up to three decimals from 1 up. Below 1, up to three significant
/// decimals, so `0.0004` stays `0.0004`. Exponent notation from `1e15` up
/// and below `1e-4`.
#[must_use]
pub fn format_number(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value.is_infinite() {
        return if value > 0.0 { "∞" } else { "-∞" }.to_owned();
    }
    let abs = value.abs();
    if abs == 0.0 {
        return "0".to_owned();
    }
    if !(1e-4..1e15).contains(&abs) {
        return exponent(value, 3);
    }
    let decimals = if abs < 1.0 {
        usize::try_from((2 - exp10(abs)).clamp(3, 6)).unwrap_or(3)
    } else {
        3
    };
    trim(&format!("{value:.decimals$}"))
}

/// How the labels of one axis are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Notation {
    /// This many decimals.
    Fixed(usize),
    /// This many mantissa decimals.
    Exponent(usize),
}

/// The power of ten of `v`: `floor(log10 |v|)`. `0` for zero.
fn exp10(v: f64) -> i32 {
    let e = v.abs().log10().floor();
    if e.is_finite() {
        #[allow(
            clippy::cast_possible_truncation,
            reason = "the log10 of a finite f64 is in -324..=309"
        )]
        let e = e as i32;
        e
    } else {
        0
    }
}

/// The notation that shows each multiple of `step` up to `max_abs` with a
/// label of its own.
fn notation(max_abs: f64, step: f64) -> Notation {
    let decimals = (0..=15_u8)
        .find(|&d| {
            let scaled = step * pow10(i32::from(d));
            (scaled - scaled.round()).abs() <= scaled * 1e-9
        })
        .map_or(16, usize::from);
    let int_digits = usize::try_from(exp10(max_abs) + 1).unwrap_or(1).max(1);
    if max_abs >= 1e15 || (max_abs > 0.0 && max_abs < 1e-4) || int_digits + decimals > 15 {
        let digits = (exp10(max_abs) - exp10(step)).clamp(0, 16);
        Notation::Exponent(usize::try_from(digits).unwrap_or(0))
    } else {
        Notation::Fixed(decimals)
    }
}

/// `value` in `notation`. A value near zero is `0`.
fn write(value: f64, step: f64, notation: Notation) -> String {
    if value.abs() < step / 2.0 {
        return "0".to_owned();
    }
    // A data end is not a multiple of the step. Give it three more digits.
    let k = value / step;
    let extra = if (k - k.round()).abs() > 1e-6 { 3 } else { 0 };
    match notation {
        Notation::Fixed(d) => {
            let d = (d + extra).min(15);
            trim(&format!("{value:.d$}"))
        }
        Notation::Exponent(d) => exponent(value, (d + extra).min(16)),
    }
}

/// A tick label: the value rounded to the precision of `step`.
///
/// For the labels of a whole axis, use [`tick_labels`]: it uses one
/// notation for all of them.
#[must_use]
pub fn format_tick(value: f64, step: f64) -> String {
    if !value.is_finite() || !step.is_finite() || step <= 0.0 {
        return format_number(value);
    }
    write(value, step, notation(value.abs(), step))
}

/// The label of each tick, in one notation. Adjacent labels differ.
///
/// ```rust
/// use autumn_charts_plugin::scale::{nice_ticks, tick_labels};
///
/// let ticks = nice_ticks(1.0, 1.000_000_5, 5);
/// let labels = tick_labels(&ticks);
/// assert_eq!(labels.first().map(String::as_str), Some("1"));
/// assert!(labels.windows(2).all(|w| w[0] != w[1]));
/// ```
#[must_use]
pub fn tick_labels(ticks: &Ticks) -> Vec<String> {
    let step = ticks.step();
    if !step.is_finite() || step <= 0.0 {
        return ticks.values().iter().map(|t| format_number(*t)).collect();
    }
    let max_abs = ticks.values().iter().fold(0.0_f64, |m, t| m.max(t.abs()));
    let notation = notation(max_abs, step);
    ticks
        .values()
        .iter()
        .map(|t| write(*t, step, notation))
        .collect()
}

/// `value` as `<mantissa>e<exp>`, with no trailing zeros in the mantissa.
fn exponent(value: f64, digits: usize) -> String {
    let text = format!("{value:.digits$e}");
    match text.split_once('e') {
        Some((mantissa, exp)) => format!("{}e{exp}", trim(mantissa)),
        None => text,
    }
}

/// Trim trailing zeros and a trailing point. `-0` becomes `0`.
fn trim(text: &str) -> String {
    let out = if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.')
    } else {
        text
    };
    if out == "-0" {
        "0".to_owned()
    } else {
        out.to_owned()
    }
}
