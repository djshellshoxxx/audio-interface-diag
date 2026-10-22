use audio_interface_diag::*;
fn telemetry() -> BurnInTelemetry {
    BurnInTelemetry {
        epoch: 1,
        callbacks: 1,
        faults: 0,
        sample_rate: 48_000.0,
    }
}

#[test]
fn eight_hour_stress_preserves_faults_without_growing_storage() {
    let mut now = telemetry();
    let mut run = BurnIn::new(28_800.0, now).unwrap();
    for second in 1..=28_800 {
        now.callbacks += 188;
        if second % 1000 == 0 {
            now.faults += 1;
        }
        run.observe(second as f64, now);
    }
    assert_eq!(run.state, BurnInState::Complete);
    assert_eq!(run.faults, 28);
    assert_eq!(run.callbacks, 28_800 * 188);
    let complete = run.elapsed_secs;
    run.observe(99_999.0, telemetry());
    assert_eq!(run.elapsed_secs, complete);
}

#[test]
fn gaps_resets_and_no_progress_cannot_pass() {
    let initial = telemetry();
    for bad in [
        BurnInTelemetry {
            epoch: 2,
            ..initial
        },
        BurnInTelemetry {
            callbacks: 0,
            ..initial
        },
        BurnInTelemetry {
            sample_rate: 96_000.0,
            ..initial
        },
    ] {
        let mut run = BurnIn::new(10.0, initial).unwrap();
        run.observe(1.0, bad);
        assert_eq!(run.state, BurnInState::Unavailable);
    }
    let mut run = BurnIn::new(10.0, initial).unwrap();
    run.observe(2.0, initial);
    assert_eq!(run.state, BurnInState::Unavailable);
    let mut run = BurnIn::new(10.0, initial).unwrap();
    run.observe(
        10.0,
        BurnInTelemetry {
            callbacks: 100,
            ..initial
        },
    );
    assert_eq!(run.state, BurnInState::Unavailable);
    for duration in [0.0, f64::NAN, f64::INFINITY, 604_801.0] {
        assert!(BurnIn::new(duration, initial).is_err());
    }
    let mut run = BurnIn::new(10.0, initial).unwrap();
    run.abort();
    run.observe(
        1.0,
        BurnInTelemetry {
            callbacks: 100,
            ..initial
        },
    );
    assert_eq!(run.state, BurnInState::Aborted);
}
