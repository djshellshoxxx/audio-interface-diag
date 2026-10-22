use std::process::Command;

#[test]
fn paired_capture_measures_known_crosstalk_and_rejects_unmatched_evidence() {
    let dir = std::env::temp_dir().join(format!("aid-crosstalk-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let driven = dir.join("driven.f32le");
    let quiet = dir.join("quiet.f32le");
    // Convert fixture to float32, the documented input format.
    let signal: Vec<f32> = (0..4800)
        .map(|i| (0.25 * (std::f64::consts::TAU * 1000.0 * i as f64 / 48_000.0).sin()) as f32)
        .collect();
    let bytes = |gain: f32| {
        signal
            .iter()
            .flat_map(|x| (x * gain).to_le_bytes())
            .collect::<Vec<_>>()
    };
    std::fs::write(&driven, bytes(1.0)).unwrap();
    std::fs::write(&quiet, bytes(0.001)).unwrap();
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_audio-interface-diag"))
            .args(["crosstalk", "48000", "1000"])
            .arg(&driven)
            .arg(&quiet)
            .output()
            .unwrap()
    };
    let output = run();
    assert!(output.status.success());
    let json = String::from_utf8(output.stdout).unwrap();
    assert!(json.contains("bench.crosstalk"), "{json}");
    assert!(json.contains("-60.000 dB"), "{json}");
    std::fs::write(&quiet, &bytes(0.001)[..4000]).unwrap();
    assert_eq!(run().status.code(), Some(1));
    std::fs::write(&quiet, [0u8; 3]).unwrap();
    assert_eq!(run().status.code(), Some(2));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn crosstalk_signal_quality_and_reference_requirements_are_enforced() {
    use audio_interface_diag::{crosstalk_report, Severity};
    let driven: Vec<f32> = (0..4800)
        .map(|i| (0.25 * (std::f64::consts::TAU * 1000.0 * i as f64 / 48_000.0).sin()) as f32)
        .collect();
    for victim in [
        vec![0.0; 4800],
        vec![1.0; 4800],
        vec![f32::INFINITY; 4800],
        vec![f32::NAN; 4800],
    ] {
        assert_eq!(
            crosstalk_report(&driven, &victim, 48_000.0, 1000.0).findings[0].severity,
            Severity::Unavailable
        );
    }
    let inverted: Vec<f32> = driven.iter().map(|x| -x * 0.01 + 0.001).collect();
    let report = crosstalk_report(&driven, &inverted, 48_000.0, 1000.0);
    assert_eq!(report.findings[0].severity, Severity::Info);
    assert!(report.findings[0].evidence.contains("-40.000 dB"));
    for rate in [0.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            crosstalk_report(&driven, &inverted, rate, 1000.0).findings[0].severity,
            Severity::Unavailable
        );
    }
    assert_eq!(
        crosstalk_report(&vec![0.0; 4800], &inverted, 48_000.0, 1000.0).findings[0].severity,
        Severity::Unavailable
    );
}
