//! SVG number and shape helpers.

use std::fmt::Write as _;

/// The number of series styles: color, dash and marker. Then they repeat.
pub const STYLES: usize = 6;

/// The style class of series `k`.
pub fn style_class(k: usize) -> String {
    format!("autumn-charts-s{}", k % STYLES)
}

/// A coordinate with at most two decimals. A non-finite value is `0`.
pub fn num(v: f64) -> String {
    if !v.is_finite() {
        return "0".to_owned();
    }
    let text = format!("{:.2}", round(v));
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" || text.is_empty() {
        "0".to_owned()
    } else {
        text.to_owned()
    }
}

/// `v` rounded to two decimals, as [`num`] writes it. Use it before you
/// add or subtract two coordinates, so the written values agree.
pub fn round(v: f64) -> f64 {
    if v.is_finite() {
        let r = (v * 100.0).round() / 100.0;
        if r.is_finite() { r } else { v }
    } else {
        0.0
    }
}

/// A path through `points`: `M x,y L x,y …`.
pub fn polyline(points: &[(f64, f64)]) -> String {
    let mut d = String::new();
    for (i, (x, y)) in points.iter().enumerate() {
        let cmd = if i == 0 { 'M' } else { 'L' };
        if i > 0 {
            d.push(' ');
        }
        let _ = write!(d, "{cmd}{},{}", num(*x), num(*y));
    }
    d
}

/// The marker shape of series `k`, centred on `(x, y)`.
///
/// Circle, square, triangle, diamond, down triangle, plus.
pub fn marker(k: usize, x: f64, y: f64, r: f64) -> String {
    let pt = |dx: f64, dy: f64| format!("{},{}", num(dx.mul_add(r, x)), num(dy.mul_add(r, y)));
    match k % STYLES {
        0 => format!(
            "M{} A{},{} 0 1,0 {} A{},{} 0 1,0 {} Z",
            pt(-1.0, 0.0),
            num(r),
            num(r),
            pt(1.0, 0.0),
            num(r),
            num(r),
            pt(-1.0, 0.0)
        ),
        1 => polygon(&[pt(-0.9, -0.9), pt(0.9, -0.9), pt(0.9, 0.9), pt(-0.9, 0.9)]),
        2 => polygon(&[pt(0.0, -1.2), pt(1.1, 0.8), pt(-1.1, 0.8)]),
        3 => polygon(&[pt(0.0, -1.3), pt(1.3, 0.0), pt(0.0, 1.3), pt(-1.3, 0.0)]),
        4 => polygon(&[pt(-1.1, -0.8), pt(1.1, -0.8), pt(0.0, 1.2)]),
        _ => polygon(&[
            pt(-0.4, -1.2),
            pt(0.4, -1.2),
            pt(0.4, -0.4),
            pt(1.2, -0.4),
            pt(1.2, 0.4),
            pt(0.4, 0.4),
            pt(0.4, 1.2),
            pt(-0.4, 1.2),
            pt(-0.4, 0.4),
            pt(-1.2, 0.4),
            pt(-1.2, -0.4),
            pt(-0.4, -0.4),
        ]),
    }
}

fn polygon(points: &[String]) -> String {
    let mut d = String::new();
    for (i, point) in points.iter().enumerate() {
        d.push(if i == 0 { 'M' } else { 'L' });
        d.push_str(point);
        d.push(' ');
    }
    d.push('Z');
    d
}

/// `n` as an `f64`. A count past `u32::MAX` saturates. A chart never has
/// that many items.
pub fn count(n: usize) -> f64 {
    u32::try_from(n).map_or(4_294_967_295.0, f64::from)
}

/// The rough width of `text` in SVG units, at a 12 unit font.
pub fn text_width(text: &str) -> f64 {
    count(text.chars().count()) * 7.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn num_trims_and_hides_non_finite_values() {
        assert_eq!(num(1.0), "1");
        assert_eq!(num(1.256), "1.26");
        assert_eq!(num(-0.001), "0");
        assert_eq!(num(f64::NAN), "0");
        assert_eq!(num(f64::INFINITY), "0");
    }

    #[test]
    fn each_marker_is_a_closed_path() {
        for k in 0..STYLES {
            let d = marker(k, 10.0, 10.0, 3.0);
            assert!(d.starts_with('M') && d.ends_with('Z'), "{d}");
        }
    }

    #[test]
    fn styles_repeat() {
        assert_eq!(style_class(0), style_class(STYLES));
    }
}
