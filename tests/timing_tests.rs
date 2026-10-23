use audio_interface_diag::*;

#[test]
fn bounded_timing_histogram_reports_percentile_upper_bounds() {
    let mut bins = [0u64; CALLBACK_TIMING_BINS];
    for (ms, count) in [(0.1, 90), (1.0, 5), (10.0, 5)] {
        bins[callback_timing_bucket(ms).unwrap()] += count;
    }
    let p95 = timing_percentile_upper_ms(&bins, 0.95).unwrap();
    let p99 = timing_percentile_upper_ms(&bins, 0.99).unwrap();
    assert!((1.0..=1.26).contains(&p95));
    assert!((10.0..=12.6).contains(&p99));
    assert!(timing_percentile_upper_ms(&[0; CALLBACK_TIMING_BINS], 0.95).is_none());
    for q in [0.0, -1.0, 1.1, f64::NAN] {
        assert!(timing_percentile_upper_ms(&bins, q).is_none());
    }
    for ms in [-1.0, f64::INFINITY, f64::NAN] {
        assert!(callback_timing_bucket(ms).is_none());
    }
    let mut overflow = [0; CALLBACK_TIMING_BINS];
    overflow[callback_timing_bucket(100_000.0).unwrap()] = 1;
    assert!(timing_percentile_upper_ms(&overflow, 0.99).is_none());
    for ms in [0.0, 0.0001, 0.001, 0.01, 0.1, 1.0, 10.0, 100.0, 1000.0] {
        let mut one = [0; CALLBACK_TIMING_BINS];
        one[callback_timing_bucket(ms).unwrap()] = 1;
        assert!(timing_percentile_upper_ms(&one, 1.0).unwrap() >= ms);
    }
}
