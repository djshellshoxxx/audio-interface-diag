//! Offline bench analysis. Call on a worker thread, never in an audio callback.
//! A coherent capture contains an integer number of cycles of the known stimulus.
use crate::{dbfs_rms, Edition, Finding, SessionSummary, Severity};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToneMeasurement {
    pub fundamental_rms_dbfs: f64,
    pub thd_percent: f64,
    /// Full capture bandwidth, excluding DC and the fundamental.
    pub thdn_percent: f64,
    pub dc: f64,
    pub harmonics: usize,
}

/// Measures a known, coherent sine capture. Rejects clipping, weak stimulus,
/// non-finite samples and non-coherent records rather than reporting leakage as THD.
/// Harmonics 2..10 strictly below Nyquist are included; this is not an AES17 filter.
pub fn measure_tone(
    samples: &[f32],
    rate: f64,
    frequency: f64,
) -> Result<ToneMeasurement, &'static str> {
    if samples.len() < 256
        || !rate.is_finite()
        || !frequency.is_finite()
        || rate <= 0.0
        || frequency <= 0.0
        || frequency * 3.0 >= rate
    {
        return Err("invalid format or insufficient harmonic bandwidth");
    }
    if samples.iter().any(|x| !x.is_finite() || x.abs() >= 1.0) {
        return Err("capture contains clipping or non-finite samples");
    }
    let n = samples.len() as f64;
    let cycles = frequency / rate * n;
    if cycles < 10.0 || (cycles - cycles.round()).abs() > 1e-6 {
        return Err("capture must contain at least ten whole stimulus cycles");
    }
    let dc = samples.iter().map(|x| *x as f64).sum::<f64>() / n;
    let omega = std::f64::consts::TAU * frequency / rate;
    let projection = |harmonic: usize| {
        let (mut sin, mut cos) = (0.0, 0.0);
        for (i, sample) in samples.iter().enumerate() {
            let phase = omega * harmonic as f64 * i as f64;
            sin += (*sample as f64 - dc) * phase.sin();
            cos += (*sample as f64 - dc) * phase.cos();
        }
        (2.0 * sin / n, 2.0 * cos / n)
    };
    let (sin, cos) = projection(1);
    let fundamental = (sin * sin + cos * cos) / 2.0;
    if fundamental < 1e-8 {
        return Err("fundamental is below -80 dBFS RMS");
    }
    let mut harmonic_power = 0.0;
    let mut harmonics = 0;
    for h in 2..=10 {
        if frequency * h as f64 >= rate / 2.0 {
            break;
        }
        let (s, c) = projection(h);
        harmonic_power += (s * s + c * c) / 2.0;
        harmonics += 1;
    }
    // Direct residual avoids subtracting nearly equal powers at very low distortion.
    let residual = samples
        .iter()
        .enumerate()
        .map(|(i, x)| {
            let phase = omega * i as f64;
            (*x as f64 - dc - sin * phase.sin() - cos * phase.cos()).powi(2)
        })
        .sum::<f64>()
        / n;
    if fundamental / (fundamental + residual) < 0.5 {
        return Err("known fundamental does not dominate the capture");
    }
    Ok(ToneMeasurement {
        fundamental_rms_dbfs: 10.0 * fundamental.log10(),
        thd_percent: 100.0 * (harmonic_power / fundamental).sqrt(),
        thdn_percent: 100.0 * (residual / fundamental).sqrt(),
        dc,
        harmonics,
    })
}

pub fn tone_report(samples: &[f32], rate: f64, frequency: f64) -> SessionSummary {
    let finding = match measure_tone(samples, rate, frequency) {
        Ok(m) => Finding {
            id: "bench.distortion", severity: Severity::Info,
            title: "Relative sine distortion measurement".into(),
            evidence: format!("{} samples at {rate} Hz; stimulus {frequency} Hz; fundamental {:.3} dBFS RMS; THD {:.6}% ({} harmonics); full-band THD+N {:.6}%; DC {:.8}; capture RMS {:.3} dBFS. Uncalibrated.", samples.len(), m.fundamental_rms_dbfs, m.thd_percent, m.harmonics, m.thdn_percent, m.dc, dbfs_rms(samples).unwrap()),
            action: "Compare captures with the same gain, cabling, bandwidth and calibration. No device quality threshold is assumed.".into(),
        },
        Err(reason) => Finding {
            id: "bench.distortion", severity: Severity::Unavailable,
            title: "Distortion measurement unavailable".into(), evidence: reason.into(),
            action: "Capture a steady known sine with a whole number of cycles; remove clipped or transient portions.".into(),
        },
    };
    SessionSummary {
        edition: Edition::Engineer,
        findings: vec![finding],
    }
}
