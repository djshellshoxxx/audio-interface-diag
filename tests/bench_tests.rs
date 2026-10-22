use audio_interface_diag::*;

fn capture(rate: f64, frequency: f64, frames: usize, harmonic: f64) -> Vec<f32> {
    (0..frames)
        .map(|i| {
            let p = std::f64::consts::TAU * frequency * i as f64 / rate;
            (0.02 + 0.25 * p.sin() + harmonic * (2.0 * p).sin()) as f32
        })
        .collect()
}

#[test]
fn known_harmonic_is_measured_independently_of_dc() {
    let samples = capture(48_000.0, 1000.0, 48_000, 0.0025);
    let m = measure_tone(&samples, 48_000.0, 1000.0).unwrap();
    assert!((m.thd_percent - 1.0).abs() < 0.0001);
    assert!((m.thdn_percent - 1.0).abs() < 0.0001);
    assert!((m.dc - 0.02).abs() < 1e-6);
    assert!((m.fundamental_rms_dbfs + 15.0515).abs() < 0.001);
}

#[test]
fn invalid_captures_never_produce_measurements() {
    let samples = capture(48_000.0, 1000.0, 4800, 0.0);
    for rate in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(measure_tone(&samples, rate, 1000.0).is_err());
    }
    for frequency in [0.0, -1.0, 1000.1, 12_000.0, 15_000.0, 16_000.0, f64::NAN] {
        assert!(measure_tone(&samples, 48_000.0, frequency).is_err());
    }
    assert!(measure_tone(&[], 48_000.0, 1000.0).is_err());
    assert!(measure_tone(&vec![0.0; 4800], 48_000.0, 1000.0).is_err());
    for invalid in [f32::NAN, f32::INFINITY, 1.0, -1.0] {
        let mut bad = samples.clone();
        bad[100] = invalid;
        assert!(measure_tone(&bad, 48_000.0, 1000.0).is_err());
        assert_eq!(
            tone_report(&bad, 48_000.0, 1000.0).findings[0].severity,
            Severity::Unavailable
        );
    }
    let wrong = capture(48_000.0, 2000.0, 4800, 0.0);
    assert!(measure_tone(&wrong, 48_000.0, 1000.0).is_err());
}

#[test]
fn stress_rates_durations_levels_and_polarity() {
    for rate in [44_100, 48_000, 96_000, 192_000] {
        for divisor in [1, 2, 10] {
            let frames = rate / divisor;
            for gain in [-1.0f32, 0.01, 0.5, 1.0] {
                let mut samples = capture(rate as f64, 1000.0, frames, 0.0025);
                samples.iter_mut().for_each(|x| *x *= gain);
                let m = measure_tone(&samples, rate as f64, 1000.0).unwrap();
                assert!((m.thd_percent - 1.0).abs() < 0.001);
                assert!((m.thdn_percent - 1.0).abs() < 0.001);
            }
        }
    }
}
