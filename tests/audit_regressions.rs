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

    assert!(findings.iter().any(|f| f.severity == Severity::Fail));
    assert!(!findings.iter().any(|f| f.id == "stream.clean"));
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

    assert!(findings.iter().any(|f| f.severity == Severity::Fail));
    assert!(!findings.iter().any(|f| f.id == "stream.clean"));
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

    assert_eq!(session.arm(), Err("passive tests do not require arming"));
    assert_eq!(session.state, RunState::Ready);
}
