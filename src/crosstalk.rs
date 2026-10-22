//! Offline pairwise coherent-tone leakage; the caller owns acquisition/routing.
use crate::{measure_tone, Edition, Finding, SessionSummary, Severity};

/// Compare simultaneous driven/victim captures at the stimulus frequency.
/// Measures a single route and frequency, never a complete channel matrix.
pub fn crosstalk_report(
    driven: &[f32],
    victim: &[f32],
    rate: f64,
    frequency: f64,
) -> SessionSummary {
    let result = measure_crosstalk(driven, victim, rate, frequency);
    let finding = match result {
        Ok(ratio_db) => Finding {
            id: "bench.crosstalk",
            severity: Severity::Info,
            title: "Relative pairwise crosstalk measurement".into(),
            evidence: format!("{} samples per channel at {rate} Hz; stimulus {frequency} Hz; victim/driven coherent tone level {ratio_db:.3} dB. Uncalibrated; one channel pair and frequency only.", driven.len()),
            action: "Record driven/victim channel IDs, gain, cabling and termination. Repeat every required route/frequency. Coincident external tones can contaminate this observation; no hardware pass threshold is assumed.".into(),
        },
        Err(reason) => Finding {
            id: "bench.crosstalk", severity: Severity::Unavailable,
            title: "Crosstalk measurement unavailable".into(), evidence: reason.into(),
            action: "Supply equal-length simultaneous coherent captures of the driven and quiet channels, with a strong unclipped reference.".into(),
        },
    };
    SessionSummary {
        edition: Edition::Engineer,
        findings: vec![finding],
    }
}

fn measure_crosstalk(
    driven: &[f32],
    victim: &[f32],
    rate: f64,
    frequency: f64,
) -> Result<f64, &'static str> {
    let reference = measure_tone(driven, rate, frequency)?;
    if victim.len() != driven.len() {
        return Err("driven and victim captures must have equal length");
    }
    if victim.iter().any(|x| !x.is_finite() || x.abs() >= 1.0) {
        return Err("victim capture contains clipping or non-finite samples");
    }
    let n = victim.len() as f64;
    let dc = victim.iter().map(|x| *x as f64).sum::<f64>() / n;
    let omega = std::f64::consts::TAU * frequency / rate;
    let (mut sine, mut cosine) = (0.0, 0.0);
    for (i, sample) in victim.iter().enumerate() {
        let phase = omega * i as f64;
        sine += (*sample as f64 - dc) * phase.sin();
        cosine += (*sample as f64 - dc) * phase.cos();
    }
    let power = 2.0 * (sine * sine + cosine * cosine) / (n * n);
    if power <= 0.0 {
        return Err(
            "no measurable victim tone; cannot report infinite isolation from digital silence",
        );
    }
    Ok(10.0 * power.log10() - reference.fundamental_rms_dbfs)
}
