use audio_interface_diag::*;

#[test]
fn compensation_retains_signed_residual_and_units() {
    let result = recording_offset(&[480.0; 5], 48_000.0, 48_000.0, 400.0).unwrap();
    assert_eq!(result.residual_samples, 80.0);
    assert!((result.residual_ms - 1.6666666667).abs() < 1e-8);
    assert_eq!(
        recording_offset(&[480.0; 5], 48_000.0, 48_000.0, 600.0)
            .unwrap()
            .residual_samples,
        -120.0
    );
}

#[test]
fn compensation_rejects_stale_uncertain_or_unavailable_measurements() {
    assert!(recording_offset(&[480.0; 4], 48_000.0, 48_000.0, 0.0).is_err());
    assert!(recording_offset(&[480.0; 5], 44_100.0, 48_000.0, 0.0).is_err());
    assert!(recording_offset(
        &[480.0, 480.0, 480.0, 480.0, 490.0],
        48_000.0,
        48_000.0,
        0.0
    )
    .is_err());
    for invalid in [f64::NAN, f64::INFINITY, -1.0] {
        assert!(recording_offset(&[480.0; 5], 48_000.0, 48_000.0, invalid).is_err());
        assert!(recording_offset(&[invalid; 5], 48_000.0, 48_000.0, 0.0).is_err());
    }
}
