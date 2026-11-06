//! Scale, domain and tick properties: no NaN, no overflow, monotonic ticks.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::float_cmp,
    reason = "a test fails on a panic, and exact values are the point"
)]

use autumn_charts_plugin::scale::{
    LinearScale, MAX_TICKS, domain, format_number, format_tick, nice_ticks, tick_labels,
};
use proptest::prelude::*;

// ── Unit cases ──────────────────────────────────────────────────────────────

#[test]
fn ticks_for_zero_to_one_hundred_are_steps_of_twenty() {
    let ticks = nice_ticks(0.0, 100.0, 5);
    assert_eq!(ticks.values(), &[0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);
    assert_eq!(ticks.step(), 20.0);
}

#[test]
fn ticks_extend_past_the_data_to_a_nice_value() {
    let ticks = nice_ticks(3.0, 97.0, 5);
    assert_eq!(ticks.values().first(), Some(&0.0));
    assert_eq!(ticks.values().last(), Some(&100.0));
}

#[test]
fn ticks_cover_a_negative_domain() {
    let ticks = nice_ticks(-7.0, 12.0, 5);
    assert!(ticks.values()[0] <= -7.0);
    assert!(*ticks.values().last().unwrap() >= 12.0);
    assert!(ticks.values().contains(&0.0), "{:?}", ticks.values());
}

#[test]
fn ticks_with_reversed_ends_are_still_ascending() {
    let ticks = nice_ticks(100.0, 0.0, 5);
    assert_eq!(ticks.values().first(), Some(&0.0));
    assert_eq!(ticks.values().last(), Some(&100.0));
}

#[test]
fn ticks_for_the_full_f64_range_stay_finite() {
    let ticks = nice_ticks(-f64::MAX, f64::MAX, 5);
    assert!(ticks.values().iter().all(|v| v.is_finite()));
    assert_eq!(ticks.values().first(), Some(&-f64::MAX));
    assert_eq!(ticks.values().last(), Some(&f64::MAX));
}

#[test]
fn ticks_at_the_float_precision_limit_do_not_repeat() {
    let ticks = nice_ticks(1e15, 1e15 + 1.0, 10);
    for pair in ticks.values().windows(2) {
        assert!(pair[0] < pair[1], "{:?}", ticks.values());
    }
}

#[test]
fn ticks_for_a_non_finite_domain_fall_back_to_zero_to_one() {
    for (a, b) in [
        (f64::NAN, 1.0),
        (0.0, f64::INFINITY),
        (f64::NEG_INFINITY, f64::NAN),
    ] {
        let ticks = nice_ticks(a, b, 5);
        assert_ne!(ticks.values().len(), 0);
        assert!(ticks.values().iter().all(|v| v.is_finite()), "{ticks:?}");
    }
}

#[test]
fn ticks_for_a_zero_span_are_one_tick() {
    let ticks = nice_ticks(4.0, 4.0, 5);
    assert_eq!(ticks.values(), &[4.0]);
}

#[test]
fn ticks_for_a_subnormal_span_do_not_loop() {
    let ticks = nice_ticks(0.0, f64::MIN_POSITIVE / 4.0, 5);
    assert_ne!(ticks.values().len(), 0);
    assert!(ticks.values().len() <= MAX_TICKS);
}

#[test]
fn domain_of_equal_values_has_a_span() {
    let (lo, hi) = domain([5.0, 5.0], false);
    assert!(lo < 5.0 && hi > 5.0, "{lo}..{hi}");
}

#[test]
fn domain_of_equal_huge_values_has_a_span() {
    let (lo, hi) = domain([f64::MAX, f64::MAX], false);
    assert!(lo < hi && hi.is_finite(), "{lo}..{hi}");
}

#[test]
fn domain_with_zero_holds_zero() {
    assert_eq!(domain([12.0, 19.0, 7.0], true), (0.0, 19.0));
    assert_eq!(domain([-10.0, -5.0], true), (-10.0, 0.0));
}

#[test]
fn domain_ignores_non_finite_values() {
    assert_eq!(
        domain([f64::NAN, 2.0, f64::INFINITY, 8.0], false),
        (2.0, 8.0)
    );
}

#[test]
fn domain_of_no_values_is_zero_to_one() {
    assert_eq!(domain([], false), (0.0, 1.0));
    assert_eq!(domain([f64::NAN], true), (0.0, 1.0));
}

#[test]
fn scale_maps_ends_and_middle() {
    let scale = LinearScale::new((0.0, 100.0), (300.0, 20.0));
    assert_eq!(scale.map(0.0), 300.0);
    assert_eq!(scale.map(100.0), 20.0);
    assert_eq!(scale.map(50.0), 160.0);
}

#[test]
fn scale_clamps_out_of_domain_values() {
    let scale = LinearScale::new((0.0, 10.0), (0.0, 100.0));
    assert_eq!(scale.map(-5.0), 0.0);
    assert_eq!(scale.map(50.0), 100.0);
}

#[test]
fn scale_of_a_zero_span_maps_to_the_middle() {
    let scale = LinearScale::new((3.0, 3.0), (0.0, 100.0));
    assert_eq!(scale.map(3.0), 50.0);
}

#[test]
fn scale_of_the_full_range_does_not_overflow() {
    let scale = LinearScale::new((-f64::MAX, f64::MAX), (0.0, 100.0));
    assert_eq!(scale.map(0.0), 50.0);
    assert_eq!(scale.map(f64::MAX), 100.0);
}

#[test]
fn tick_labels_round_to_the_step() {
    assert_eq!(format_tick(0.1 + 0.2, 0.1), "0.3");
    assert_eq!(format_tick(20.0, 20.0), "20");
    assert_eq!(format_tick(-0.0, 1.0), "0");
    assert_eq!(format_tick(2.5, 0.5), "2.5");
}

#[test]
fn large_and_small_numbers_use_exponents() {
    assert_eq!(format_number(1.5e300), "1.5e300");
    assert_eq!(format_number(2e-7), "2e-7");
    assert_eq!(format_tick(4e20, 2e20), "4e20");
}

#[test]
fn plain_numbers_trim_zeros() {
    assert_eq!(format_number(12.0), "12");
    assert_eq!(format_number(-3.25), "-3.25");
    assert_eq!(format_number(1.0 / 3.0), "0.333");
    assert_eq!(format_number(-0.0), "0");
}

#[test]
fn non_finite_numbers_have_a_label() {
    assert_eq!(format_number(f64::NAN), "NaN");
    assert_eq!(format_number(f64::INFINITY), "∞");
}

#[test]
fn the_tick_count_sets_the_step() {
    assert_eq!(nice_ticks(0.0, 100.0, 10).step(), 10.0);
    assert_eq!(nice_ticks(0.0, 100.0, 2).step(), 50.0);
}

#[test]
fn a_negative_zero_tick_is_positive_zero() {
    for ticks in [nice_ticks(-0.0, -0.0, 5), nice_ticks(-10.0, 10.0, 4)] {
        for t in ticks.values() {
            if *t == 0.0 {
                assert!(t.is_sign_positive(), "{:?}", ticks.values());
            }
        }
    }
}

#[test]
fn tick_labels_stay_distinct_for_a_small_span() {
    for (lo, hi) in [
        (1.0, 1.000_000_5),
        (100.0, 100.000_001),
        (1e15, 1e15 + 1e9),
        (1e-5, 1e-5 + 1e-11),
        (0.0, 0.001),
        (-1e-5, 1e-5),
    ] {
        let ticks = nice_ticks(lo, hi, 5);
        let labels = tick_labels(&ticks);
        for pair in labels.windows(2) {
            assert_ne!(pair[0], pair[1], "{lo}..{hi}: {labels:?}");
        }
    }
}

#[test]
fn tick_labels_use_one_notation_on_an_axis() {
    let labels = tick_labels(&nice_ticks(9e14, 1.1e15, 4));
    assert!(labels.iter().all(|l| l.contains('e')), "{labels:?}");
    let labels = tick_labels(&nice_ticks(0.0, 2e-4, 4));
    assert!(labels.iter().all(|l| !l.contains('e')), "{labels:?}");
}

#[test]
fn small_numbers_keep_their_digits() {
    assert_eq!(format_number(0.0004), "0.0004");
    assert_eq!(format_number(0.00012), "0.00012");
    assert_eq!(format_number(-0.0004), "-0.0004");
    assert_eq!(format_number(0.0015), "0.0015");
}

#[test]
fn scale_maps_nan_to_the_range_start() {
    let scale = LinearScale::new((0.0, 10.0), (300.0, 20.0));
    assert_eq!(scale.map(f64::NAN), 300.0);
}

// ── Properties ──────────────────────────────────────────────────────────────

fn any_f64() -> impl Strategy<Value = f64> {
    prop_oneof![
        8 => -1e6..1e6f64,
        2 => any::<f64>(),
        1 => Just(f64::MAX),
        1 => Just(-f64::MAX),
        1 => Just(f64::MIN_POSITIVE),
        1 => Just(0.0),
        1 => Just(f64::NAN),
        1 => Just(f64::INFINITY),
    ]
}

fn finite_f64() -> impl Strategy<Value = f64> {
    prop_oneof![
        8 => -1e6..1e6f64,
        2 => any::<f64>().prop_filter("finite", |v| v.is_finite()),
        1 => Just(f64::MAX),
        1 => Just(-f64::MAX),
        1 => Just(f64::MIN_POSITIVE),
        1 => Just(0.0),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    #[test]
    fn ticks_are_finite_ascending_and_bounded(a in any_f64(), b in any_f64(), n in 0usize..40) {
        let ticks = nice_ticks(a, b, n);
        let v = ticks.values();
        prop_assert!(!v.is_empty());
        prop_assert!(v.len() <= MAX_TICKS, "{} ticks", v.len());
        prop_assert!(v.iter().all(|t| t.is_finite()), "{v:?}");
        for pair in v.windows(2) {
            prop_assert!(pair[0] < pair[1], "not ascending: {v:?}");
        }
        prop_assert!(ticks.step().is_finite() && ticks.step() >= 0.0);
    }

    #[test]
    fn ticks_cover_a_finite_domain(a in finite_f64(), b in finite_f64(), n in 1usize..12) {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        let v = nice_ticks(a, b, n).values().to_vec();
        prop_assert!(v[0] <= lo, "first {} > {lo}", v[0]);
        prop_assert!(*v.last().unwrap() >= hi, "last {} < {hi}", v.last().unwrap());
    }

    #[test]
    fn ticks_are_not_wider_than_one_step_past_the_data(a in -1e9..1e9f64, b in -1e9..1e9f64, n in 1usize..12) {
        prop_assume!(a != b);
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        let ticks = nice_ticks(a, b, n);
        let v = ticks.values();
        let step = ticks.step();
        prop_assert!(v[0] > step.mul_add(-1.000_001, lo), "{v:?} step {step}");
        prop_assert!(*v.last().unwrap() < step.mul_add(1.000_001, hi), "{v:?} step {step}");
        for t in &v[1..v.len() - 1] {
            let k = t / step;
            prop_assert!((k - k.round()).abs() < 1e-6, "{t} is not a multiple of {step}");
        }
    }

    #[test]
    fn adjacent_tick_labels_differ(base in -1e12..1e12f64, exp in -6i32..6, n in 2usize..12) {
        let span = base.abs().max(1.0) * 10f64.powi(exp) * 1e-6;
        let ticks = nice_ticks(base, base + span, n);
        let labels = tick_labels(&ticks);
        for pair in labels.windows(2) {
            prop_assert_ne!(&pair[0], &pair[1], "{:?} {:?}", ticks.values(), labels);
        }
        for label in &labels {
            prop_assert!(label.len() <= 24, "{label}");
        }
    }

    #[test]
    fn domain_is_finite_and_ordered(values in prop::collection::vec(any_f64(), 0..20), zero in any::<bool>()) {
        let (lo, hi) = domain(values.iter().copied(), zero);
        prop_assert!(lo.is_finite() && hi.is_finite(), "{lo}..{hi}");
        prop_assert!(lo < hi, "{lo}..{hi}");
        if zero {
            prop_assert!(lo <= 0.0 && hi >= 0.0);
        }
        for v in values.iter().filter(|v| v.is_finite()) {
            prop_assert!(lo <= *v && *v <= hi);
        }
    }

    #[test]
    fn scale_output_is_finite_and_in_range(
        d0 in any_f64(), d1 in any_f64(), v in any_f64(),
        r0 in -1e4..1e4f64, r1 in -1e4..1e4f64,
    ) {
        let out = LinearScale::new((d0, d1), (r0, r1)).map(v);
        prop_assert!(out.is_finite(), "{out}");
        prop_assert!(out >= r0.min(r1) - 1e-9 && out <= r0.max(r1) + 1e-9, "{out} outside {r0}..{r1}");
    }

    #[test]
    fn scale_is_monotonic(a in any_f64(), b in any_f64(), p in -1e9..1e9f64, q in -1e9..1e9f64) {
        let (x, y) = if p <= q { (p, q) } else { (q, p) };
        let (d0, d1) = domain([a, b], false);
        let up = LinearScale::new((d0, d1), (0.0, 100.0));
        prop_assert!(up.map(x) <= up.map(y));
        let down = LinearScale::new((d0, d1), (100.0, 0.0));
        prop_assert!(down.map(x) >= down.map(y));
    }

    #[test]
    fn labels_are_short_and_never_nan(v in finite_f64(), s in any_f64()) {
        for label in [format_number(v), format_tick(v, s)] {
            prop_assert!(!label.is_empty());
            prop_assert!(label.len() <= 24, "{label}");
            prop_assert!(!label.contains("NaN") && !label.contains("inf"), "{label}");
        }
    }
}
