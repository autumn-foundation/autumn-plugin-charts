//! Scales, ticks and number labels. Pure math, no markup.
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
/// give one tick. The result has 1 to [`MAX_TICKS`] values.
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

/// The default number label: up to three decimals, no trailing zeros.
/// Exponent notation from `1e15` up and below `1e-4`.
#[must_use]
pub fn format_number(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value.is_infinite() {
        return if value > 0.0 { "∞" } else { "-∞" }.to_owned();
    }
    let abs = value.abs();
    if abs != 0.0 && !(1e-4..1e15).contains(&abs) {
        return exponent(value, 3);
    }
    trim(&format!("{value:.3}"))
}

/// A tick label: the value rounded to the precision of `step`.
#[must_use]
pub fn format_tick(value: f64, step: f64) -> String {
    if !value.is_finite() || !step.is_finite() || step <= 0.0 {
        return format_number(value);
    }
    if value.abs() < step / 2.0 {
        return "0".to_owned();
    }
    let decimals = (0..=12_u8)
        .find(|&d| {
            let scaled = step * pow10(i32::from(d));
            (scaled - scaled.round()).abs() <= scaled * 1e-9
        })
        .map_or(13, usize::from);
    let abs = value.abs();
    if !(1e-4..1e15).contains(&abs) || decimals > 6 {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "clamped to 0..=6"
        )]
        let digits = (abs.log10().ceil() - step.log10().floor()).clamp(0.0, 6.0) as usize;
        return exponent(value, digits);
    }
    trim(&format!("{value:.decimals$}"))
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
