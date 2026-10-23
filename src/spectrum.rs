//! Bounded worker-thread visual analysis. These functions allocate and must never
//! run in a real-time callback. Values are digital, unweighted and uncalibrated.
use rustfft::{num_complex::Complex, FftPlanner};

/// Display floor, not a measured analog noise floor.
pub const SPECTRUM_FLOOR_DBFS: f64 = -150.0;

#[derive(Debug, Clone)]
pub struct Spectrum {
    pub bin_hz: f64,
    pub frames: usize,
    /// Single-sided peak amplitudes, Hann coherent-gain corrected. DC/Nyquist
    /// are not doubled. A full-scale coherent sine has a 0 dBFS center bin.
    pub magnitudes_dbfs: Vec<f64>,
}

/// Analyze the final power-of-two window, between 256 and 16384 samples.
/// This is a magnitude display, not integrated PSD, THD or a noise certification.
pub fn analyze_spectrum(samples: &[f32], rate: f64, frames: usize) -> Option<Spectrum> {
    if !rate.is_finite()
        || rate <= 0.0
        || !(256..=16_384).contains(&frames)
        || !frames.is_power_of_two()
        || samples.len() < frames
    {
        return None;
    }
    let samples = &samples[samples.len() - frames..];
    if samples.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let mut coherent_gain = 0.0;
    let mut buffer: Vec<_> = samples
        .iter()
        .enumerate()
        .map(|(i, sample)| {
            // Periodic Hann: known FFT-bin tones retain their correct amplitude.
            let weight = 0.5 - 0.5 * (std::f64::consts::TAU * i as f64 / frames as f64).cos();
            coherent_gain += weight;
            Complex {
                re: *sample as f64 * weight,
                im: 0.0,
            }
        })
        .collect();
    FftPlanner::<f64>::new()
        .plan_fft_forward(frames)
        .process(&mut buffer);
    let magnitudes_dbfs = buffer[..=frames / 2]
        .iter()
        .enumerate()
        .map(|(bin, value)| {
            let sided = if bin == 0 || bin == frames / 2 {
                1.0
            } else {
                2.0
            };
            let amplitude = value.norm() * sided / coherent_gain;
            (20.0 * amplitude.log10()).max(SPECTRUM_FLOOR_DBFS)
        })
        .collect();
    Some(Spectrum {
        bin_hz: rate / frames as f64,
        frames,
        magnitudes_dbfs,
    })
}

#[derive(Debug, Clone, Copy)]
pub struct ScopeSpan {
    pub min: f32,
    pub max: f32,
}

/// Min/max envelopes preserve brief peaks when reducing a capture for display.
/// Returns at most 1024 spans and rejects invalid evidence rather than drawing it.
pub fn scope_envelope(samples: &[f32], points: usize) -> Option<Vec<ScopeSpan>> {
    if samples.is_empty() || points == 0 || samples.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let count = points.min(1024).min(samples.len());
    Some(
        (0..count)
            .map(|i| {
                let begin = i * samples.len() / count;
                let end = (i + 1) * samples.len() / count;
                let slice = &samples[begin..end];
                ScopeSpan {
                    min: slice.iter().copied().fold(f32::INFINITY, f32::min),
                    max: slice.iter().copied().fold(f32::NEG_INFINITY, f32::max),
                }
            })
            .collect(),
    )
}
