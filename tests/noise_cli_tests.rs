use std::process::Command;

#[test]
fn noise_capture_reports_relative_level_dc_and_mains_evidence() {
    let path = std::env::temp_dir().join(format!("aid-noise-{}.f32le", std::process::id()));
    let bytes: Vec<_> = (0..48_000)
        .flat_map(|i| {
            (0.01 + 0.1 * (std::f64::consts::TAU * 60.0 * i as f64 / 48_000.0).sin() as f32)
                .to_le_bytes()
        })
        .collect();
    std::fs::write(&path, bytes).unwrap();
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_audio-interface-diag"))
            .args(["noise", "48000"])
            .arg(&path)
            .output()
            .unwrap()
    };
    let output = run();
    assert!(output.status.success());
    let report = String::from_utf8(output.stdout).unwrap();
    assert!(report.contains("bench.noise"), "{report}");
    assert!(report.contains("60 Hz"));
    assert!(report.contains("-23.010"));
    assert!(report.contains("DC 0.010000"));
    assert!(report.contains("Uncalibrated"));

    for samples in [
        vec![0.0; 48_000],
        vec![1.0; 48_000],
        vec![f32::NAN; 48_000],
        vec![0.1; 100],
    ] {
        std::fs::write(
            &path,
            samples
                .iter()
                .flat_map(|x| x.to_le_bytes())
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let invalid = run();
        assert_eq!(invalid.status.code(), Some(1));
        assert!(String::from_utf8(invalid.stdout)
            .unwrap()
            .contains("UNAVAILABLE"));
    }
    std::fs::write(&path, [0u8; 3]).unwrap();
    assert_eq!(run().status.code(), Some(2));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn noise_rate_and_capture_validity_gates_are_explicit() {
    use audio_interface_diag::{noise_report, Severity};
    let capture: Vec<f32> = (0..48_000)
        .map(|i| if i % 2 == 0 { 0.01 } else { -0.01 })
        .collect();
    for rate in [f64::NAN, f64::INFINITY, -48_000.0, 0.0, 360.0, 96_000.0] {
        let report = noise_report(&capture, rate);
        assert_eq!(report.findings[0].severity, Severity::Unavailable);
    }
    assert_eq!(
        noise_report(&[], 48_000.0).findings[0].severity,
        Severity::Unavailable
    );
    for rate in [44_100, 48_000, 96_000, 192_000] {
        for mains in [50, 60] {
            let capture: Vec<f32> = (0..rate)
                .map(|i| {
                    -0.025
                        + 0.01
                            * (std::f64::consts::TAU * mains as f64 * i as f64 / rate as f64).sin()
                                as f32
                })
                .collect();
            let report = noise_report(&capture, rate as f64);
            assert_eq!(report.findings[0].severity, Severity::Info);
            assert!(report.findings[0].evidence.contains(&format!("{mains} Hz")));
            assert!(report.findings[0].evidence.contains("-43.010"));
            assert!(report.findings[0].evidence.contains("DC -0.025000"));
        }
    }
}
