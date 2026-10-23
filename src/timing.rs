//! Fixed-size callback timing histogram. Bucketing performs no allocation; read
//! percentiles on a non-realtime thread. Upper bounds are deliberately explicit.
pub const CALLBACK_TIMING_BINS: usize = 64;

/// Three buckets per octave starting at 1 microsecond. The final bucket is
/// overflow, not a finite timing claim. Invalid durations are not observations.
pub fn callback_timing_bucket(milliseconds: f64) -> Option<usize> {
    if !milliseconds.is_finite() || milliseconds < 0.0 {
        return None;
    }
    let index = if milliseconds <= 0.001 {
        0
    } else {
        ((milliseconds / 0.001).log2() * 3.0).ceil() as usize
    };
    Some(index.min(CALLBACK_TIMING_BINS - 1))
}

/// Nearest-rank percentile's histogram upper bound in ms; at most 26% bucket
/// width (except the first <=1 us bucket). Overflow/empty/invalid is unavailable.
pub fn timing_percentile_upper_ms(
    bins: &[u64; CALLBACK_TIMING_BINS],
    quantile: f64,
) -> Option<f64> {
    if !quantile.is_finite() || quantile <= 0.0 || quantile > 1.0 {
        return None;
    }
    let total = bins
        .iter()
        .fold(0u64, |sum, count| sum.saturating_add(*count));
    if total == 0 {
        return None;
    }
    let rank = (quantile * total as f64).ceil().max(1.0) as u64;
    let mut cumulative = 0u64;
    for (index, count) in bins.iter().enumerate() {
        cumulative = cumulative.saturating_add(*count);
        if cumulative >= rank {
            return (index != CALLBACK_TIMING_BINS - 1)
                .then(|| 0.001 * 2.0f64.powf(index as f64 / 3.0));
        }
    }
    None
}
