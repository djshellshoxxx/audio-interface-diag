use audio_interface_diag::*;

#[test]
fn level_validation_requires_ack_above_minus_12() {
    assert!(validate_generator_level(DEFAULT_GENERATOR_DBFS, false).is_ok());
    assert!(validate_generator_level(-6.0, false).is_err());
    assert_eq!(validate_generator_level(-6.0, true), Ok(-6.0));
    assert!(validate_generator_level(1.0, true).is_err());
    assert!(validate_generator_level(f64::NAN, true).is_err());
}

#[test]
fn fade_ramps_edges_to_zero() {
    let mut s = vec![1.0f32; 100];
    apply_fade(&mut s, 10);
    assert_eq!(s[0], 0.0);
    assert_eq!(s[99], 0.0);
    assert_eq!(s[50], 1.0);
}

#[test]
fn measure_delay_finds_noise_burst_with_confidence() {
    let reference = noise_burst(-20.0, 1024, 7).unwrap();
    let mut captured = vec![0.0f32; 8000];
    captured[3210..3210 + 1024].copy_from_slice(&reference);
    let m = measure_delay(&reference, &captured, 6000).unwrap();
    assert_eq!(m.delay_samples, 3210);
    assert!(m.confidence >= MIN_DELAY_CONFIDENCE);
}

#[test]
fn measure_delay_rejects_unrelated_capture() {
    let reference = noise_burst(-20.0, 1024, 7).unwrap();
    let captured = noise_burst(-20.0, 8000, 99).unwrap();
    assert!(measure_delay(&reference, &captured, 6000).is_none());
    assert!(measure_delay(&reference, &vec![0.0; 8000], 6000).is_none());
}

#[test]
fn summarize_runs_median_and_spread() {
    let s = summarize_runs(&[10.0, 12.0, 11.0, 30.0]).unwrap();
    assert_eq!(s.median, 11.5);
    assert_eq!(s.spread, 20.0);
    assert_eq!(s.runs, 4);
    assert!(summarize_runs(&[]).is_none());
}

#[test]
fn hum_detects_60hz_family() {
    let mut s = sine(60.0, -20.0, 48_000.0, 48_000).unwrap();
    let noise = noise_burst(-50.0, 48_000, 3).unwrap();
    for (a, b) in s.iter_mut().zip(noise) {
        *a += b;
    }
    let hum = analyze_hum(&s, 48_000.0).unwrap();
    assert_eq!(hum.mains_hz, 60);
    assert!(hum.family_db_rel_total > -1.0);
}

#[test]
fn goertzel_matches_sine_mean_square() {
    let s = sine(1000.0, -6.0206, 48_000.0, 48_000).unwrap();
    let p = goertzel_power(&s, 1000.0, 48_000.0).unwrap();
    assert!((p - 0.125).abs() < 1e-3);
}

#[test]
fn reports_render_unavailable_and_escape() {
    let summary = SessionSummary {
        edition: Edition::Daw,
        findings: vec![Finding {
            id: "x.y",
            severity: Severity::Unavailable,
            title: "Quote \" and, comma".into(),
            evidence: "line1\nline2".into(),
            action: "none".into(),
        }],
    };
    assert!(render_text(&summary).contains("[UNAVAILABLE]"));
    let json = render_json(&summary);
    assert!(json.contains("\\\"") && json.contains("\\n") && json.contains("UNAVAILABLE"));
    let csv = render_csv(&summary);
    assert!(csv.contains("\"Quote \"\" and, comma\""));
}
