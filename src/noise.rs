//! Offline unweighted noise measurements; never opens a device or runs on the audio thread.
use crate::{analyze_hum, dc_offset, Edition, Finding, SessionSummary, Severity};

/// Analyze at least one second of a user-captured terminated-input signal.
/// Levels are relative to digital full scale; microphone, gain and termination
/// conditions must be documented separately before comparing measurements.
pub fn noise_report(samples: &[f32], rate: f64) -> SessionSummary {
    let result = measure_noise(samples, rate);
    let finding = match result {
        Ok((rms_dbfs, dc, hum)) => Finding {
            id: "bench.noise",
            severity: Severity::Info,
            title: "Relative unweighted noise measurement".into(),
            evidence: format!("{} samples at {rate} Hz; DC-removed broadband RMS {rms_dbfs:.3} dBFS; DC {dc:.6}; dominant mains family {} Hz, {:.3} dB relative to AC power. Uncalibrated; full capture bandwidth, no weighting. A mains-family observation does not identify a fault or its cause.", samples.len(), hum.mains_hz, hum.family_db_rel_total),
            action: "Record input termination, gain, routing and sample rate; compare only matching conditions. No absolute hardware noise-floor or quality certification is inferred.".into(),
        },
        Err(reason) => Finding {
            id: "bench.noise",
            severity: Severity::Unavailable,
            title: "Noise measurement unavailable".into(),
            evidence: reason.into(),
            action: "Supply at least one second of finite, unclipped input capture with nonzero AC content and a valid sample rate.".into(),
        },
    };
    SessionSummary {
        edition: Edition::Engineer,
        findings: vec![finding],
    }
}

fn measure_noise(samples: &[f32], rate: f64) -> Result<(f64, f64, crate::HumReport), &'static str> {
    if !rate.is_finite() || rate <= 360.0 || (samples.len() as f64) < rate {
        return Err("invalid sample rate or less than one second of capture");
    }
    if samples.iter().any(|x| !x.is_finite() || x.abs() >= 1.0) {
        return Err("capture contains clipping or non-finite samples");
    }
    let dc = dc_offset(samples).ok_or("empty capture")?;
    let ac: Vec<f32> = samples.iter().map(|x| (*x as f64 - dc) as f32).collect();
    let power = ac.iter().map(|x| (*x as f64).powi(2)).sum::<f64>() / samples.len() as f64;
    if power <= 0.0 {
        return Err(
            "capture has no AC content; cannot distinguish digital silence from a measurement",
        );
    }
    let hum = analyze_hum(&ac, rate).ok_or("mains-family analysis unavailable")?;
    Ok((10.0 * power.log10(), dc, hum))
}
