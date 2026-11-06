//! Line, bar and area charts: two axes, a grid and a legend.

use maud::{Markup, Render, html};

use crate::chart::{Common, NO_VALUE, Swatch, Tip, common_setters};
use crate::scale::{LinearScale, Ticks, domain, format_tick, nice_ticks};
use crate::series::Series;
use crate::svg::{count, marker, num, polyline, round, style_class, text_width};

/// The chart kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Line,
    Bar,
    Area,
}

#[derive(Debug, Clone)]
struct Cartesian<'a> {
    categories: &'a [&'a str],
    series: &'a [Series<'a>],
    x_label: Option<&'a str>,
    y_label: Option<&'a str>,
    min: Option<f64>,
    max: Option<f64>,
    ticks: usize,
    zero: bool,
    stacked: bool,
}

/// The `viewBox` width and height.
const W: f64 = 640.0;
const H: f64 = 320.0;
/// Past this many categories, markers hide until hover or focus.
const DENSE: usize = 40;
/// The marker radius.
const MARKER: f64 = 3.5;
/// The gap between the y axis and the first point of a line or an area.
const INSET: f64 = 12.0;

/// One drawn value: the category, the value, and its base and top on the
/// value axis.
#[derive(Debug, Clone, Copy)]
struct Point {
    index: usize,
    value: f64,
    base: f64,
    top: f64,
}

/// `a + b`, held in the finite range.
fn saturating_add(a: f64, b: f64) -> f64 {
    (a + b).clamp(-f64::MAX, f64::MAX)
}

impl Cartesian<'_> {
    /// The finite value of series `k` at category `i`.
    fn value(&self, k: usize, i: usize) -> Option<f64> {
        if i >= self.categories.len() {
            return None;
        }
        self.series
            .get(k)
            .and_then(|s| s.values().get(i))
            .copied()
            .filter(|v| v.is_finite())
    }

    /// The points of each series. A stack puts each value on the sum of the
    /// values before it with the same sign. In a stacked area, a missing
    /// value adds 0, so the shape has no hole.
    fn points(&self, kind: Kind) -> Vec<Vec<Point>> {
        let n = self.categories.len();
        let mut pos = vec![0.0; n];
        let mut neg = vec![0.0; n];
        let mut out = Vec::with_capacity(self.series.len());
        for k in 0..self.series.len() {
            let mut points = Vec::with_capacity(n);
            for i in 0..n {
                let value = self.value(k, i);
                if !self.stacked {
                    if let Some(v) = value {
                        points.push(Point {
                            index: i,
                            value: v,
                            base: 0.0,
                            top: v,
                        });
                    }
                    continue;
                }
                let v = match value {
                    Some(v) => v,
                    None if kind == Kind::Area => 0.0,
                    None => continue,
                };
                let acc = if v >= 0.0 { &mut pos[i] } else { &mut neg[i] };
                let base = *acc;
                *acc = saturating_add(base, v);
                points.push(Point {
                    index: i,
                    value: value.unwrap_or(f64::NAN),
                    base,
                    top: *acc,
                });
            }
            out.push(points);
        }
        out
    }

    /// The y domain: the data, zero if the kind needs it, then the app
    /// `min` and `max`.
    fn y_domain(&self, kind: Kind, points: &[Vec<Point>]) -> (f64, f64) {
        let include_zero = self.zero || kind != Kind::Line;
        let (lo, hi) = if self.stacked {
            domain(
                points.iter().flatten().flat_map(|p| [p.base, p.top]),
                include_zero,
            )
        } else {
            domain(points.iter().flatten().map(|p| p.top), include_zero)
        };
        let lo = self.min.filter(|v| v.is_finite()).unwrap_or(lo);
        let hi = self.max.filter(|v| v.is_finite()).unwrap_or(hi);
        domain([lo, hi], false)
    }

    /// The plot layout and the y scale.
    fn frame(&self, kind: Kind, points: &[Vec<Point>], common: &Common<'_>) -> Frame {
        let (lo, hi) = self.y_domain(kind, points);
        let ticks = nice_ticks(lo, hi, self.ticks);
        let tick_text: Vec<String> = ticks
            .values()
            .iter()
            .map(|t| {
                common
                    .format
                    .map_or_else(|| format_tick(*t, ticks.step()), |f| f(*t))
            })
            .collect();
        let widest = tick_text.iter().map(|t| text_width(t)).fold(0.0, f64::max);
        let y_title = if self.y_label.is_some() { 20.0 } else { 0.0 };
        let x_title = if self.x_label.is_some() { 20.0 } else { 0.0 };
        let x0 = (widest + 12.0).clamp(28.0, 160.0) + y_title;
        let (x1, y0, y1) = (W - 16.0, 16.0, H - 28.0 - x_title);
        let n = self.categories.len();
        // X labels thin out so they do not overlap.
        let widest_x = self
            .categories
            .iter()
            .map(|c| text_width(c))
            .fold(0.0, f64::max)
            + 8.0;
        let fit = ((x1 - x0) / widest_x).floor().max(1.0);
        let every = (1..=n.max(1))
            .find(|e| count(n) <= fit * count(*e))
            .unwrap_or(1);
        Frame {
            kind,
            n,
            x0,
            x1,
            y0,
            y1,
            y: LinearScale::new(ticks.bounds(), (y1, y0)),
            ticks,
            tick_text,
            every,
        }
    }

    /// The grid, the ticks, the axes and the axis titles.
    fn axes(&self, f: &Frame) -> Markup {
        let (blo, bhi) = f.ticks.bounds();
        let zero = f.zero_y();
        html! {
            g aria-hidden="true" {
                @for (t, text) in f.ticks.values().iter().zip(&f.tick_text) {
                    @let ty = round(f.y.map(*t));
                    line class="autumn-charts-grid" x1=(num(f.x0)) y1=(num(ty)) x2=(num(f.x1)) y2=(num(ty)) {}
                    text class="autumn-charts-tick autumn-charts-tick--y" x=(num(f.x0 - 6.0))
                        y=(num(ty + 4.0)) text-anchor="end" { (text) }
                }
                @if blo < 0.0 && bhi > 0.0 {
                    line class="autumn-charts-zero" x1=(num(f.x0)) y1=(num(zero)) x2=(num(f.x1)) y2=(num(zero)) {}
                }
                line class="autumn-charts-axis" x1=(num(f.x0)) y1=(num(f.y1)) x2=(num(f.x1)) y2=(num(f.y1)) {}
                line class="autumn-charts-axis" x1=(num(f.x0)) y1=(num(f.y0)) x2=(num(f.x0)) y2=(num(f.y1)) {}
                @for (i, cat) in self.categories.iter().enumerate().step_by(f.every) {
                    text class="autumn-charts-tick autumn-charts-tick--x" x=(num(f.x(i)))
                        y=(num(f.y1 + 18.0)) text-anchor="middle" { (cat) }
                }
                @if let Some(title) = self.x_label {
                    text class="autumn-charts-axis-label" x=(num(f64::midpoint(f.x0, f.x1)))
                        y=(num(H - 6.0)) text-anchor="middle" { (title) }
                }
                @if let Some(title) = self.y_label {
                    text class="autumn-charts-axis-label" x=(num(-f64::midpoint(f.y0, f.y1))) y="14"
                        transform="rotate(-90)" text-anchor="middle" { (title) }
                }
            }
        }
    }

    /// The text of one mark: series, category and value.
    fn label(&self, k: usize, p: &Point, common: &Common<'_>) -> String {
        format!(
            "{}, {}: {}",
            self.series[k].name(),
            self.categories[p.index],
            common.text(p.value)
        )
    }

    /// The bars of series `k`.
    fn bars(&self, f: &Frame, k: usize, points: &[Point], common: &Common<'_>) -> Markup {
        let band = f.band();
        let lanes = if self.stacked {
            1
        } else {
            self.series.len().max(1)
        };
        let width = band * if self.stacked { 0.6 } else { 0.8 } / count(lanes);
        let lane = if self.stacked { 0.0 } else { count(k) };
        let style = style_class(k);
        html! {
            @for p in points {
                @let left = count(lanes).mul_add(-width, band) / 2.0 + band.mul_add(count(p.index), f.x0);
                @let x = width.mul_add(lane, left);
                @let top = round(f.y.map(p.base.max(p.top)));
                @let bottom = round(f.y.map(p.base.min(p.top)));
                @let shape = html! {
                    rect class={ "autumn-charts-bar " (style) } x=(num(width.mul_add(0.05, x)))
                        y=(num(top)) width=(num(width * 0.9)) height=(num(bottom - top)) {}
                };
                (common.mark(&self.label(k, p, common), &shape, f.tip(width.mul_add(0.5, x), top)))
            }
        }
    }

    /// The area, the line and the markers of series `k`.
    fn lines(&self, f: &Frame, k: usize, points: &[Point], common: &Common<'_>) -> Markup {
        let runs = runs(points, self.stacked);
        let style = style_class(k);
        let area = if f.kind == Kind::Area {
            area_path(&runs, f)
        } else {
            String::new()
        };
        let line = line_path(&runs, f);
        html! {
            @if !area.is_empty() {
                path class={ "autumn-charts-area " (style) } d=(area) aria-hidden="true" {}
            }
            @if !line.is_empty() {
                path class={ "autumn-charts-line " (style) } d=(line) aria-hidden="true" {}
            }
            @for p in points.iter().filter(|p| p.value.is_finite()) {
                @let (x, y) = (f.x(p.index), f.y.map(p.top));
                @let shape = html! {
                    path class={ "autumn-charts-marker " (style) } d=(marker(k, x, y, MARKER)) {}
                };
                (common.mark(&self.label(k, p, common), &shape, f.tip(x, y - MARKER)))
            }
        }
    }

    /// One table row for each category: a cell for each series.
    fn rows(&self, common: &Common<'_>) -> Vec<(&str, Vec<String>)> {
        self.categories
            .iter()
            .enumerate()
            .map(|(i, cat)| {
                let cells = (0..self.series.len())
                    .map(|k| {
                        self.value(k, i)
                            .map_or_else(|| NO_VALUE.to_owned(), |v| common.text(v))
                    })
                    .collect();
                (*cat, cells)
            })
            .collect()
    }

    fn render(&self, kind: Kind, common: &Common<'_>) -> Markup {
        let points = self.points(kind);
        let f = self.frame(kind, &points, common);
        let body = html! {
            (self.axes(&f))
            @for (k, series_points) in points.iter().enumerate() {
                @if kind == Kind::Bar {
                    (self.bars(&f, k, series_points, common))
                } @else {
                    (self.lines(&f, k, series_points, common))
                }
            }
        };
        let legend: Vec<(usize, String)> = self
            .series
            .iter()
            .enumerate()
            .map(|(k, s)| (k, s.name().to_owned()))
            .collect();
        let swatch = if kind == Kind::Line {
            Swatch::Line
        } else {
            Swatch::Fill
        };
        let mut head = vec![self.x_label.unwrap_or("Category")];
        head.extend(self.series.iter().map(Series::name));
        common.root(
            kind.name(),
            f.n > DENSE,
            &html! {
                (common.svg(&format!("0 0 {} {}", num(W), num(H)), &self.desc(kind, &points, common), &body))
                @if common.shows_legend(legend.len(), legend.len() > 1) {
                    (Common::legend(swatch, &legend))
                }
                (common.table(&head, &self.rows(common)))
            },
        )
    }

    /// The default `<desc>`: what the chart shows and the value range.
    fn desc(&self, kind: Kind, points: &[Vec<Point>], common: &Common<'_>) -> String {
        let (lo, hi) = points
            .iter()
            .flatten()
            .map(|p| p.value)
            .filter(|v| v.is_finite())
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
                (lo.min(v), hi.max(v))
            });
        if lo > hi {
            return "No data.".to_owned();
        }
        let name = match (kind, self.stacked) {
            (Kind::Line, _) => "Line chart",
            (Kind::Bar, false) => "Bar chart",
            (Kind::Bar, true) => "Stacked bar chart",
            (Kind::Area, false) => "Area chart",
            (Kind::Area, true) => "Stacked area chart",
        };
        let n = self.categories.len();
        format!(
            "{name} with {} series and {n} {}. Values range from {} to {}.",
            self.series.len(),
            if n == 1 { "category" } else { "categories" },
            common.text(lo),
            common.text(hi),
        )
    }
}

impl Kind {
    const fn name(self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::Bar => "bar",
            Self::Area => "area",
        }
    }
}

/// The plot layout and the y scale of one chart.
struct Frame {
    kind: Kind,
    n: usize,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    y: LinearScale,
    ticks: Ticks,
    tick_text: Vec<String>,
    /// Show each `every`-th x label.
    every: usize,
}

impl Frame {
    /// The width of one category band.
    fn band(&self) -> f64 {
        (self.x1 - self.x0) / count(self.n.max(1))
    }

    /// The x of category `i`: the band centre for bars, else spread from
    /// end to end.
    fn x(&self, i: usize) -> f64 {
        let width = self.x1 - self.x0;
        if self.kind == Kind::Bar {
            self.band().mul_add(count(i) + 0.5, self.x0)
        } else if self.n <= 1 {
            self.x0 + width / 2.0
        } else {
            // Keep the first and the last point off the axes.
            2.0f64
                .mul_add(-INSET, width)
                .mul_add(count(i) / count(self.n - 1), self.x0 + INSET)
        }
    }

    /// The y of the zero line.
    fn zero_y(&self) -> f64 {
        round(self.y.map(0.0))
    }

    /// Where the tip of a mark at `x` with its top at `top` goes.
    fn tip(&self, x: f64, top: f64) -> Tip {
        let width = self.x1 - self.x0;
        let anchor = if x < width.mul_add(0.2, self.x0) {
            "start"
        } else if x > width.mul_add(0.8, self.x0) {
            "end"
        } else {
            "middle"
        };
        let y = if top - 10.0 < 14.0 {
            top + 22.0
        } else {
            top - 10.0
        };
        Tip { x, y, anchor }
    }
}

/// Split points into runs with no gap. In a stack, each category has a
/// point, so there is one run.
fn runs(points: &[Point], stacked: bool) -> Vec<Vec<Point>> {
    let mut out: Vec<Vec<Point>> = Vec::new();
    let mut last: Option<usize> = None;
    for p in points {
        let joined = last.is_some_and(|l| stacked || l + 1 == p.index);
        if !joined {
            out.push(Vec::new());
        }
        if let Some(run) = out.last_mut() {
            run.push(*p);
        }
        last = Some(p.index);
    }
    out
}

fn line_path(runs: &[Vec<Point>], f: &Frame) -> String {
    runs.iter()
        .map(|run| {
            let coords: Vec<(f64, f64)> =
                run.iter().map(|p| (f.x(p.index), f.y.map(p.top))).collect();
            polyline(&coords)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The filled shape: along the tops, then back along the bases.
fn area_path(runs: &[Vec<Point>], f: &Frame) -> String {
    runs.iter()
        .map(|run| {
            let tops = run.iter().map(|p| (f.x(p.index), f.y.map(p.top)));
            let bases = run.iter().rev().map(|p| (f.x(p.index), f.y.map(p.base)));
            format!("{} Z", polyline(&tops.chain(bases).collect::<Vec<_>>()))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

macro_rules! cartesian_chart {
    ($(#[$doc:meta])* $name:ident, $kind:expr) => {
        $(#[$doc])*
        #[derive(Debug, Clone)]
        pub struct $name<'a> {
            common: Common<'a>,
            inner: Cartesian<'a>,
        }

        impl<'a> $name<'a> {
            /// A chart of `series` over `categories`.
            #[must_use]
            pub const fn new(
                title: &'a str,
                categories: &'a [&'a str],
                series: &'a [Series<'a>],
            ) -> Self {
                Self {
                    common: Common::new(title),
                    inner: Cartesian {
                        categories,
                        series,
                        x_label: None,
                        y_label: None,
                        min: None,
                        max: None,
                        ticks: 5,
                        zero: false,
                        stacked: false,
                    },
                }
            }

            common_setters!();

            /// Set the x axis title.
            #[must_use]
            pub const fn x_label(mut self, label: &'a str) -> Self {
                self.inner.x_label = Some(label);
                self
            }

            /// Set the y axis title.
            #[must_use]
            pub const fn y_label(mut self, label: &'a str) -> Self {
                self.inner.y_label = Some(label);
                self
            }

            /// Set the y axis minimum.
            #[must_use]
            pub const fn min(mut self, min: f64) -> Self {
                self.inner.min = Some(min);
                self
            }

            /// Set the y axis maximum.
            #[must_use]
            pub const fn max(mut self, max: f64) -> Self {
                self.inner.max = Some(max);
                self
            }

            /// Set about how many y ticks to show.
            #[must_use]
            pub const fn ticks(mut self, count: usize) -> Self {
                self.inner.ticks = count;
                self
            }

            /// Write the chart.
            #[must_use]
            pub fn render(&self) -> Markup {
                self.inner.render($kind, &self.common)
            }
        }

        impl Render for $name<'_> {
            fn render(&self) -> Markup {
                $name::render(self)
            }
        }
    };
}

cartesian_chart! {
    /// A line chart: one line for each series. A missing value breaks
    /// the line.
    LineChart, Kind::Line
}
cartesian_chart! {
    /// A bar chart: the bars of a category sit side by side, or stack
    /// with [`BarChart::stacked`].
    BarChart, Kind::Bar
}
cartesian_chart! {
    /// An area chart: each series fills down to zero, or stacks with
    /// [`AreaChart::stacked`].
    AreaChart, Kind::Area
}

impl LineChart<'_> {
    /// Put zero in the y domain.
    #[must_use]
    pub const fn zero_baseline(mut self) -> Self {
        self.inner.zero = true;
        self
    }
}

impl BarChart<'_> {
    /// Stack the series.
    #[must_use]
    pub const fn stacked(mut self) -> Self {
        self.inner.stacked = true;
        self
    }
}

impl AreaChart<'_> {
    /// Stack the series.
    #[must_use]
    pub const fn stacked(mut self) -> Self {
        self.inner.stacked = true;
        self
    }
}
