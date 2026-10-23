use audio_interface_diag::*;

#[test]
fn windowed_fft_preserves_known_tone_level_frequency_and_dc() {
    let samples: Vec<f32> = (0..4096)
        .map(|i| 0.1 + 0.5 * (std::f64::consts::TAU * 64.0 * i as f64 / 4096.0).sin() as f32)
        .collect();
    let spectrum = analyze_spectrum(&samples, 48_000.0, 4096).unwrap();
    assert_eq!(spectrum.magnitudes_dbfs.len(), 2049);
    assert_eq!(spectrum.bin_hz, 11.71875);
    assert!((spectrum.magnitudes_dbfs[64] + 6.0206).abs() < 0.01);
    assert!((spectrum.magnitudes_dbfs[0] + 20.0).abs() < 0.01);
    assert!(spectrum.magnitudes_dbfs[200] < -100.0);
}

#[test]
fn spectrum_and_scope_are_bounded_and_reject_invalid_inputs() {
    assert!(analyze_spectrum(&[0.0; 4096], 48_000.0, 4096)
        .unwrap()
        .magnitudes_dbfs
        .iter()
        .all(|x| *x == SPECTRUM_FLOOR_DBFS));
    for rate in [0.0, f64::NAN, f64::INFINITY] {
        assert!(analyze_spectrum(&[0.1; 4096], rate, 4096).is_none());
    }
    for frames in [0, 128, 300, 8192, 32768] {
        assert!(analyze_spectrum(&[0.1; 4096], 48_000.0, frames).is_none());
    }
    let mut corrupt = vec![0.0; 4096];
    corrupt[20] = f32::NAN;
    assert!(analyze_spectrum(&corrupt, 48_000.0, 4096).is_none());
    assert!(scope_envelope(&corrupt, 128).is_none());
    assert!(scope_envelope(&[], 128).is_none());
    assert!(scope_envelope(&[0.0; 10], 0).is_none());
    let mut signal = vec![0.0; 48_000];
    signal[10] = -0.8;
    signal[200] = 0.9;
    let scope = scope_envelope(&signal, 128).unwrap();
    assert_eq!(scope.len(), 128);
    assert!(scope.iter().any(|span| span.min == -0.8));
    assert!(scope.iter().any(|span| span.max == 0.9));
}
