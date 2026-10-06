use audio_interface_diag::*;

#[test]
fn invalid_sample_rate_never_reports_clean_pass() {
    let findings = assess_stream(StreamStats {
        sample_rate: 0.0,
        buffer_frames: 128,
        callback_count: 100,
        xruns: 0,
        discontinuities: 0,
        max_callback_ms: 0.1,
        cpu_load: 0.1,
    });

    assert!(findings
        .iter()
        .any(|finding| finding.severity == Severity::Fail));
    assert!(!findings
        .iter()
        .any(|finding| finding.id == "stream.clean"));
}

#[test]
fn non_finite_stream_values_never_report_clean_pass() {
    let findings = assess_stream(StreamStats {
        sample_rate: 48_000.0,
        buffer_frames: 128,
        callback_count: 100,
        xruns: 0,
        discontinuities: 0,
        max_callback_ms: f64::NAN,
        cpu_load: f64::NAN,
    });

    assert!(findings
        .iter()
        .any(|finding| finding.severity == Severity::Fail));
    assert!(!findings
        .iter()
        .any(|finding| finding.id == "stream.clean"));
}

#[test]
fn sine_rejects_non_finite_parameters() {
    assert!(sine(f64::NAN, -20.0, 48_000.0, 64).is_err());
    assert!(sine(1_000.0, -20.0, f64::NAN, 64).is_err());
}

#[test]
fn sine_rejects_zero_length_generation() {
    assert!(sine(1_000.0, -20.0, 48_000.0, 0).is_err());
}

#[test]
fn sweep_rejects_non_finite_parameters() {
    assert!(linear_sweep(f64::NAN, 10_000.0, -20.0, 48_000.0, 1024).is_err());
    assert!(linear_sweep(20.0, f64::NAN, -20.0, 48_000.0, 1024).is_err());
    assert!(linear_sweep(20.0, 10_000.0, -20.0, f64::NAN, 1024).is_err());
}

#[test]
fn passive_session_cannot_be_armed() {
    let test = plan(Edition::Live).into_iter().next().unwrap();
    let mut session = TestSession::new(Edition::Live, test);

    session.prepare().unwrap();

    assert_eq!(
        session.arm(),
        Err("passive tests do not require arming")
    );
    assert_eq!(session.state, RunState::Ready);
}

#[test]
fn zero_callbacks_never_report_clean_pass() {
    let findings = assess_stream(StreamStats {
        sample_rate: 48_000.0,
        buffer_frames: 128,
        callback_count: 0,
        xruns: 0,
        discontinuities: 0,
        max_callback_ms: 0.0,
        cpu_load: 0.0,
    });

    assert!(findings
        .iter()
        .any(|finding| finding.severity == Severity::Fail));
    assert!(!findings
        .iter()
        .any(|finding| finding.id == "stream.clean"));
}

#[test]
fn delay_estimator_rejects_silent_or_non_finite_measurements() {
    assert_eq!(
        estimate_delay_samples(&[0.0, 0.0, 0.0], &[0.0, 0.0, 0.0], 2),
        None
    );
    assert_eq!(
        estimate_delay_samples(&[1.0, f32::NAN], &[0.0, 1.0, 0.0], 2),
        None
    );
}

#[test]
fn level_analysis_rejects_non_finite_samples() {
    assert_eq!(dbfs_rms(&[0.5, f32::NAN]), None);
    assert_eq!(peak_dbfs(&[0.5, f32::INFINITY]), None);
    assert_eq!(dc_offset(&[0.5, f32::NAN]), None);
}

#[test]
fn active_start_rechecks_engineer_edition_before_enabling_output() {
    let test = plan(Edition::Engineer)
        .into_iter()
        .find(|test| test.intrusive)
        .unwrap();
    let mut session = TestSession::new(Edition::Live, test);

    session.prepare().unwrap();
    // Publicly-restored or corrupted state must not bypass the edition gate.
    session.state = RunState::Armed;

    assert_eq!(
        session.start(),
        Err("active tests require Engineer edition")
    );
    assert!(!session.output_enabled);
    assert_eq!(session.state, RunState::Armed);
}
