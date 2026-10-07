use crate::model::{Finding, Severity, StreamStats};

pub fn dbfs_rms(samples: &[f32]) -> Option<f64> {
    if samples.is_empty() || samples.iter().any(|sample| !sample.is_finite()) {
        return None;
    }

    let sum = samples
        .iter()
        .map(|x| (*x as f64) * (*x as f64))
        .sum::<f64>();
    let rms = (sum / samples.len() as f64).sqrt();

    if rms == 0.0 {
        Some(f64::NEG_INFINITY)
    } else {
        Some(20.0 * rms.log10())
    }
}

pub fn peak_dbfs(samples: &[f32]) -> Option<f64> {
    if samples.iter().any(|sample| !sample.is_finite()) {
        return None;
    }

    let peak = samples.iter().map(|x| x.abs() as f64).reduce(f64::max)?;

    if peak == 0.0 {
        Some(f64::NEG_INFINITY)
    } else {
        Some(20.0 * peak.log10())
    }
}

pub fn dc_offset(samples: &[f32]) -> Option<f64> {
    if samples.is_empty() || samples.iter().any(|sample| !sample.is_finite()) {
        return None;
    }

    Some(samples.iter().map(|x| *x as f64).sum::<f64>() / samples.len() as f64)
}

pub fn correlation(a: &[f32], b: &[f32]) -> Option<f64> {
    let n = a.len().min(b.len());
    if n < 2
        || a[..n].iter().any(|sample| !sample.is_finite())
        || b[..n].iter().any(|sample| !sample.is_finite())
    {
        return None;
    }

    let (a, b) = (&a[..n], &b[..n]);
    let ma = a.iter().map(|x| *x as f64).sum::<f64>() / n as f64;
    let mb = b.iter().map(|x| *x as f64).sum::<f64>() / n as f64;

    let mut numerator = 0.0;
    let mut a_energy = 0.0;
    let mut b_energy = 0.0;

    for i in 0..n {
        let xa = a[i] as f64 - ma;
        let xb = b[i] as f64 - mb;
        numerator += xa * xb;
        a_energy += xa * xa;
        b_energy += xb * xb;
    }

    let denominator = (a_energy * b_energy).sqrt();
    if denominator == 0.0 {
        None
    } else {
        Some((numerator / denominator).clamp(-1.0, 1.0))
    }
}

pub fn estimate_delay_samples(
    reference: &[f32],
    captured: &[f32],
    max_delay: usize,
) -> Option<usize> {
    if reference.is_empty()
        || captured.is_empty()
        || reference.iter().any(|sample| !sample.is_finite())
        || captured.iter().any(|sample| !sample.is_finite())
    {
        return None;
    }

    let limit = max_delay.min(captured.len().saturating_sub(1));
    let mut best: Option<(usize, f64)> = None;

    for delay in 0..=limit {
        let n = reference.len().min(captured.len() - delay);
        if n == 0 {
            continue;
        }

        let score = (0..n)
            .map(|i| reference[i] as f64 * captured[i + delay] as f64)
            .sum::<f64>()
            .abs();

        if best.map(|(_, current)| score > current).unwrap_or(true) {
            best = Some((delay, score));
        }
    }

    best.and_then(|(delay, score)| if score > 0.0 { Some(delay) } else { None })
}

pub fn ppm_error(nominal_rate: f64, measured_rate: f64) -> Option<f64> {
    if !nominal_rate.is_finite()
        || !measured_rate.is_finite()
        || nominal_rate <= 0.0
        || measured_rate <= 0.0
    {
        return None;
    }

    Some((measured_rate - nominal_rate) / nominal_rate * 1_000_000.0)
}

pub fn assess_stream(stats: StreamStats) -> Vec<Finding> {
    let mut findings = Vec::new();

    if !stats.sample_rate.is_finite()
        || stats.sample_rate <= 0.0
        || stats.buffer_frames == 0
        || stats.callback_count == 0
        || !stats.max_callback_ms.is_finite()
        || stats.max_callback_ms < 0.0
        || !stats.cpu_load.is_finite()
        || stats.cpu_load < 0.0
    {
        findings.push(Finding {
            id: "stream.invalid_stats",
            severity: Severity::Fail,
            title: "Invalid stream telemetry".into(),
            evidence: format!(
                "sample_rate={}, buffer_frames={}, callback_count={}, max_callback_ms={}, cpu_load={}",
                stats.sample_rate,
                stats.buffer_frames,
                stats.callback_count,
                stats.max_callback_ms,
                stats.cpu_load
            ),
            action: "Discard this measurement and verify the audio backend telemetry source.".into(),
        });
        return findings;
    }

    if stats.xruns > 0 {
        findings.push(Finding {
            id: "stream.xruns",
            severity: Severity::Fail,
            title: "Buffer underrun/overrun detected".into(),
            evidence: format!("{} xruns observed", stats.xruns),
            action: "Increase buffer size, reduce competing load, and retest.".into(),
        });
    }

    if stats.discontinuities > 0 {
        findings.push(Finding {
            id: "stream.discontinuity",
            severity: Severity::Fail,
            title: "Audio discontinuity detected".into(),
            evidence: format!("{} discontinuities observed", stats.discontinuities),
            action: "Inspect driver, transport, clocking and device-reset events.".into(),
        });
    }

    let period_ms = stats.buffer_frames as f64 / stats.sample_rate * 1000.0;

    if stats.max_callback_ms >= period_ms {
        findings.push(Finding {
            id: "stream.callback_deadline",
            severity: Severity::Fail,
            title: "Callback exceeded audio period".into(),
            evidence: format!(
                "{:.3} ms worst callback vs {:.3} ms buffer period",
                stats.max_callback_ms, period_ms
            ),
            action:
                "Increase buffer size or reduce real-time load before continuing critical work."
                    .into(),
        });
    } else if stats.max_callback_ms > period_ms * 0.8 {
        findings.push(Finding {
            id: "stream.callback_margin",
            severity: Severity::Warn,
            title: "Low callback timing margin".into(),
            evidence: format!(
                "{:.3} ms worst callback vs {:.3} ms buffer period",
                stats.max_callback_ms, period_ms
            ),
            action: "Reduce DSP/system load or increase buffer size before critical work.".into(),
        });
    }

    if stats.cpu_load > 0.85 {
        findings.push(Finding {
            id: "stream.cpu",
            severity: Severity::Warn,
            title: "High audio processing load".into(),
            evidence: format!("{:.0}% measured audio load", stats.cpu_load * 100.0),
            action: "Create more CPU headroom before reducing latency.".into(),
        });
    }

    if findings.is_empty() {
        findings.push(Finding {
            id: "stream.clean",
            severity: Severity::Pass,
            title: "Stream stable".into(),
            evidence: "No xruns or discontinuities and adequate callback margin.".into(),
            action: "No corrective action required.".into(),
        });
    }

    findings
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DelayMeasurement {
    pub delay_samples: usize,
    /// Ratio of the correlation peak to the mean absolute correlation. Higher is more trustworthy.
    pub confidence: f64,
}

/// Minimum peak-to-mean correlation ratio for a delay measurement to be considered valid.
pub const MIN_DELAY_CONFIDENCE: f64 = 8.0;

/// Cross-correlation delay estimate with a confidence figure. Returns `None` when the capture is
/// invalid or no clear correlation peak exists (e.g. no loopback connected).
pub fn measure_delay(
    reference: &[f32],
    captured: &[f32],
    max_delay: usize,
) -> Option<DelayMeasurement> {
    if reference.is_empty()
        || captured.len() < reference.len()
        || reference.iter().any(|sample| !sample.is_finite())
        || captured.iter().any(|sample| !sample.is_finite())
    {
        return None;
    }

    let limit = max_delay.min(captured.len() - reference.len());
    let mut best = (0usize, 0.0f64);
    let mut total = 0.0f64;

    for delay in 0..=limit {
        let score = reference
            .iter()
            .zip(&captured[delay..])
            .map(|(r, c)| *r as f64 * *c as f64)
            .sum::<f64>()
            .abs();
        total += score;
        if score > best.1 {
            best = (delay, score);
        }
    }

    let mean = total / (limit + 1) as f64;
    if best.1 <= 0.0 || mean <= 0.0 {
        return None;
    }

    let confidence = best.1 / mean;
    (confidence >= MIN_DELAY_CONFIDENCE).then_some(DelayMeasurement {
        delay_samples: best.0,
        confidence,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RepeatSummary {
    pub median: f64,
    pub min: f64,
    pub max: f64,
    pub spread: f64,
    pub runs: usize,
}

/// Median/min/max/spread of repeated measurements (e.g. round-trip latency runs).
pub fn summarize_runs(values: &[f64]) -> Option<RepeatSummary> {
    if values.is_empty() || values.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let n = sorted.len();
    let median = if n.is_multiple_of(2) {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    } else {
        sorted[n / 2]
    };
    Some(RepeatSummary {
        median,
        min: sorted[0],
        max: sorted[n - 1],
        spread: sorted[n - 1] - sorted[0],
        runs: n,
    })
}

/// Goertzel tone power for `frequency_hz`, returned as the tone's mean-square contribution.
pub fn goertzel_power(samples: &[f32], frequency_hz: f64, sample_rate: f64) -> Option<f64> {
    if samples.is_empty()
        || !frequency_hz.is_finite()
        || !sample_rate.is_finite()
        || frequency_hz <= 0.0
        || frequency_hz >= sample_rate / 2.0
        || samples.iter().any(|s| !s.is_finite())
    {
        return None;
    }
    let coeff = 2.0 * (2.0 * std::f64::consts::PI * frequency_hz / sample_rate).cos();
    let (mut s1, mut s2) = (0.0f64, 0.0f64);
    for x in samples {
        let s0 = *x as f64 + coeff * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    let magnitude_sq = s1 * s1 + s2 * s2 - coeff * s1 * s2;
    let n = samples.len() as f64;
    // |X| = A*N/2 for a sine of amplitude A; mean square of that sine is A^2/2.
    Some(2.0 * magnitude_sq / (n * n))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HumReport {
    /// 50 or 60
    pub mains_hz: u32,
    /// Power of the fundamental plus 2nd/3rd harmonic relative to total signal power, in dB.
    pub family_db_rel_total: f64,
}

/// Observes the dominant mains-hum family (50 or 60 Hz plus 2nd/3rd harmonics). This is an
/// observation, not a definitive diagnosis.
pub fn analyze_hum(samples: &[f32], sample_rate: f64) -> Option<HumReport> {
    let total =
        samples.iter().map(|x| (*x as f64).powi(2)).sum::<f64>() / samples.len().max(1) as f64;
    if total <= 0.0 {
        return None;
    }
    let family = |base: f64| -> Option<f64> {
        Some(
            goertzel_power(samples, base, sample_rate)?
                + goertzel_power(samples, base * 2.0, sample_rate)?
                + goertzel_power(samples, base * 3.0, sample_rate)?,
        )
    };
    let (p50, p60) = (family(50.0)?, family(60.0)?);
    let (mains_hz, power) = if p50 >= p60 { (50, p50) } else { (60, p60) };
    Some(HumReport {
        mains_hz,
        family_db_rel_total: 10.0 * (power.max(1e-30) / total).log10(),
    })
}
