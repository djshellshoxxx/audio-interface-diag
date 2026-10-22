use crate::summarize_runs;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecordingOffset {
    /// Positive means capture is still late after the entered compensation.
    pub residual_samples: f64,
    pub residual_ms: f64,
}

/// A recording-offset aid, not an automatic DAW setting change. The supplied host
/// compensation must describe the same loopback path, including both directions.
pub fn recording_offset(
    runs: &[f64],
    measured_rate: f64,
    current_rate: f64,
    host_compensation_samples: f64,
) -> Result<RecordingOffset, &'static str> {
    if !measured_rate.is_finite() || measured_rate <= 0.0 || measured_rate != current_rate {
        return Err("measurement sample rate does not match the current path");
    }
    if !host_compensation_samples.is_finite() || host_compensation_samples < 0.0 {
        return Err("enter nonnegative host compensation for this loopback path");
    }
    if runs.len() < 5 || runs.iter().any(|x| !x.is_finite() || *x < 0.0) {
        return Err("at least five valid nonnegative loopback runs are required");
    }
    let summary = summarize_runs(runs).ok_or("loopback measurements are unavailable")?;
    if summary.spread > 2.0 {
        return Err("loopback spread exceeds two samples; repeat under stable conditions");
    }
    let residual_samples = summary.median - host_compensation_samples;
    let residual_ms = residual_samples / measured_rate * 1000.0;
    if !residual_ms.is_finite() {
        return Err("offset is outside the supported numeric range");
    }
    Ok(RecordingOffset {
        residual_samples,
        residual_ms,
    })
}
