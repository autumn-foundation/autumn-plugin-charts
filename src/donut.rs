//! Donut charts.

use std::f64::consts::{FRAC_PI_2, TAU};

use maud::{Markup, Render, html};

use crate::chart::{Common, NO_VALUE, Swatch, Tip, common_setters};
use crate::svg::{num, style_class};

/// The `viewBox` size, the centre and the outer and inner radius.
const SIZE: f64 = 200.0;
const C: f64 = 100.0;
const R: f64 = 96.0;
const INNER: f64 = 60.0;

/// One drawn slice.
struct Slice<'s> {
    /// The index in the input.
    index: usize,
    label: &'s str,
    value: f64,
    style: usize,
    start: f64,
    sweep: f64,
    share: f64,
}

/// A donut chart: one slice for each positive value, with the total in
/// the centre.
///
/// A zero, negative or non-finite value has no slice and no legend item.
/// The data table still lists it.
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

    /// The drawn slices and the total. Shares come from values divided by
    /// the largest value, so the sum does not overflow.
    fn slices(&self) -> (Vec<Slice<'a>>, f64) {
        let valid = |v: f64| v.is_finite() && v > 0.0;
        let largest = self
            .slices
            .iter()
            .map(|(_, v)| *v)
            .filter(|v| valid(*v))
            .fold(0.0, f64::max);
        if largest <= 0.0 {
            return (Vec::new(), 0.0);
        }
        let scaled: f64 = self
            .slices
            .iter()
            .filter(|(_, v)| valid(*v))
            .map(|(_, v)| v / largest)
            .sum();
        let total = self
            .slices
            .iter()
            .filter(|(_, v)| valid(*v))
            .fold(0.0_f64, |acc, (_, v)| (acc + v).min(f64::MAX));
        let mut start = -FRAC_PI_2;
        let mut out = Vec::new();
        for (index, (label, value)) in self
            .slices
            .iter()
            .enumerate()
            .filter(|(_, (_, v))| valid(*v))
        {
            let share = (value / largest / scaled).clamp(0.0, 1.0);
            let sweep = share * TAU;
            out.push(Slice {
                index,
                label,
                value: *value,
                style: out.len(),
                start,
                sweep,
                share,
            });
            start += sweep;
        }
        (out, total)
    }

    /// The share as a percent, with at most one decimal.
    fn percent(share: f64) -> String {
        let text = format!("{:.1}", share * 100.0);
        format!("{}%", text.trim_end_matches('0').trim_end_matches('.'))
    }

    fn label(&self, slice: &Slice<'_>) -> String {
        format!(
            "{}: {} ({})",
            slice.label,
            self.common.text(slice.value),
            Self::percent(slice.share)
        )
    }

    /// Write the chart.
    #[must_use]
    pub fn render(&self) -> Markup {
        let (slices, total) = self.slices();
        let full = slices.len() == 1;
        let (total_y, label_y) = if self.center_label.is_some() {
            (C + 2.0, C + 22.0)
        } else {
            (C + 9.0, C)
        };
        let tip = Tip {
            x: C,
            y: C + 5.0,
            anchor: "middle",
        };
        let body = html! {
            @for slice in &slices {
                @let class = format!("autumn-charts-slice {}", style_class(slice.style));
                @let shape = if full {
                    html! { path class=(class) d=(ring()) fill-rule="evenodd" {} }
                } else {
                    html! { path class=(class) d=(sector(slice.start, slice.sweep)) {} }
                };
                (self.common.mark(&self.label(slice), &shape, tip))
            }
            @if !slices.is_empty() {
                g class="autumn-charts-center" aria-hidden="true" {
                    text class="autumn-charts-total" x=(num(C)) y=(num(total_y))
                        text-anchor="middle" { (self.common.text(total)) }
                    @if let Some(label) = self.center_label {
                        text class="autumn-charts-center-label" x=(num(C)) y=(num(label_y))
                            text-anchor="middle" { (label) }
                    }
                }
            }
        };
        let desc = if slices.is_empty() {
            "No data.".to_owned()
        } else {
            format!(
                "Donut chart with {} {}. Total {}.",
                slices.len(),
                if slices.len() == 1 { "slice" } else { "slices" },
                self.common.text(total)
            )
        };
        let legend: Vec<(usize, String)> =
            slices.iter().map(|s| (s.style, self.label(s))).collect();
        let rows: Vec<(&str, Vec<String>)> = self
            .slices
            .iter()
            .enumerate()
            .map(|(i, (label, value))| {
                let shown = slices.iter().find(|s| s.index == i);
                let value_text = if value.is_finite() {
                    self.common.text(*value)
                } else {
                    NO_VALUE.to_owned()
                };
                let share = shown.map_or_else(|| NO_VALUE.to_owned(), |s| Self::percent(s.share));
                (*label, vec![value_text, share])
            })
            .collect();
        self.common.root(
            "donut",
            false,
            &html! {
                (self.common.svg(&format!("0 0 {0} {0}", num(SIZE)), &desc, &body))
                @if self.common.shows_legend(legend.len(), true) {
                    (Common::legend(Swatch::Fill, &legend))
                }
                (self.common.table(&["Category", "Value", "Share"], &rows))
            },
        )
    }
}

impl Render for DonutChart<'_> {
    fn render(&self) -> Markup {
        DonutChart::render(self)
    }
}

/// The point at `angle` on a circle of radius `r` around the centre.
fn at(angle: f64, r: f64) -> String {
    format!(
        "{},{}",
        num(r.mul_add(angle.cos(), C)),
        num(r.mul_add(angle.sin(), C))
    )
}

/// An annular sector from `start`, clockwise by `sweep`.
fn sector(start: f64, sweep: f64) -> String {
    let end = start + sweep;
    let large = u8::from(sweep > std::f64::consts::PI);
    format!(
        "M{} A{R},{R} 0 {large},1 {} L{} A{INNER},{INNER} 0 {large},0 {} Z",
        at(start, R),
        at(end, R),
        at(end, INNER),
        at(start, INNER),
    )
}

/// A full ring: two half arcs out, two half arcs in. `fill-rule="evenodd"`
/// cuts the hole.
fn ring() -> String {
    let (r, i) = (num(R), num(INNER));
    format!(
        "M{0},{c} A{r},{r} 0 1,1 {1},{c} A{r},{r} 0 1,1 {0},{c} Z M{2},{c} A{i},{i} 0 1,0 {3},{c} A{i},{i} 0 1,0 {2},{c} Z",
        num(C + R),
        num(C - R),
        num(C + INNER),
        num(C - INNER),
        c = num(C),
    )
}
