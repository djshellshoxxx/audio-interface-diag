use std::process::Command;

#[test]
fn cli_exports_valid_capture_and_rejects_missing_or_malformed_data() {
    let dir = std::env::temp_dir().join(format!("aid-bench-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let capture = dir.join("tone.f32le");
    let bytes: Vec<_> = (0..4800)
        .flat_map(|i| {
            (0.25 * (std::f64::consts::TAU * 1000.0 * i as f64 / 48_000.0).sin() as f32)
                .to_le_bytes()
        })
        .collect();
    std::fs::write(&capture, bytes).unwrap();
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_audio-interface-diag"))
            .args(["tone", "48000", "1000"])
            .arg(&capture)
            .output()
            .unwrap()
    };
    let output = run();
    assert!(output.status.success());
    let json = String::from_utf8(output.stdout).unwrap();
    assert!(json.contains("bench.distortion"));
    assert!(json.contains("THD+N"));
    assert!(!json.contains("UNAVAILABLE"));
    std::fs::write(&capture, [0u8; 4 * 4800]).unwrap();
    let silent = run();
    assert_eq!(silent.status.code(), Some(1));
    assert!(String::from_utf8(silent.stdout)
        .unwrap()
        .contains("UNAVAILABLE"));
    std::fs::write(&capture, [0u8; 3]).unwrap();
    assert_eq!(run().status.code(), Some(2));
    let oversized = std::fs::File::create(&capture).unwrap();
    oversized.set_len(32 * 1024 * 1024 + 4).unwrap();
    drop(oversized);
    assert_eq!(run().status.code(), Some(2));
    std::fs::remove_file(&capture).unwrap();
    assert_eq!(run().status.code(), Some(2));
    std::fs::remove_dir(&dir).unwrap();
}
