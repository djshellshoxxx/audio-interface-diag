use audio_interface_diag::*;

#[test]
fn active_test_cannot_start_without_arming() {
    let test = plan(Edition::Engineer)
        .into_iter()
        .find(|test| test.intrusive)
        .unwrap();
    let mut session = TestSession::new(Edition::Engineer, test);

    session.prepare().unwrap();

    assert_eq!(session.start(), Err("active test must be armed"));
    assert!(!session.output_enabled);
}

#[test]
fn active_test_arms_only_in_engineer_mode() {
    let mut test = plan(Edition::Engineer)
        .into_iter()
        .find(|test| test.intrusive)
        .unwrap();
    test.intrusive = true;

    let mut session = TestSession::new(Edition::Live, test);
    session.prepare().unwrap();

    assert_eq!(
        session.arm(),
        Err("active tests require Engineer edition")
    );
}

#[test]
fn abort_always_silences_output() {
    let test = plan(Edition::Engineer)
        .into_iter()
        .find(|test| test.intrusive)
        .unwrap();
    let mut session = TestSession::new(Edition::Engineer, test);

    session.prepare().unwrap();
    session.arm().unwrap();
    session.start().unwrap();

    assert!(session.output_enabled);

    session.abort();

    assert!(!session.output_enabled);
    assert_eq!(session.state, RunState::Aborted);
}

#[test]
fn minus_six_dbfs_converts_to_expected_linear_gain() {
    let amplitude = dbfs_to_linear(-6.020599913).unwrap();

    assert!((amplitude - 0.5).abs() < 1e-6);
}

#[test]
fn sine_rejects_frequency_at_or_above_nyquist() {
    assert!(sine(24_000.0, -20.0, 48_000.0, 64).is_err());
}

#[test]
fn clipping_counter_uses_absolute_level() {
    assert_eq!(
        clipped_sample_count(&[-1.0, -0.2, 0.2, 1.0], 0.99),
        2
    );
}

#[test]
fn gain_mismatch_reports_channel_delta() {
    let a = [0.5f32; 16];
    let b = [0.25f32; 16];
    let delta = gain_mismatch_db(&a, &b).unwrap();

    assert!((delta - 6.020599913).abs() < 1e-5);
}
