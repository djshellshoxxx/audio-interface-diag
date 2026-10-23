use audio_interface_diag::*;

#[test]
fn passive_alsa_snapshot_retains_reported_format_units_and_state() {
    let dir = std::env::temp_dir().join(format!("aid-alsa-{}", std::process::id()));
    let pcm = dir.join("card0/pcm2c/sub0");
    std::fs::create_dir_all(&pcm).unwrap();
    std::fs::write(dir.join("card0/id"), "USBInterface\n").unwrap();
    std::fs::write(pcm.join("hw_params"), "access: RW_INTERLEAVED\nformat: S24_LE\nchannels: 2\nrate: 48000 (48000/1)\nperiod_size: 256\nbuffer_size: 1024\n").unwrap();
    std::fs::write(
        pcm.join("status"),
        "state: RUNNING\nowner_pid: 123\ndelay: 512\navail: 0\n",
    )
    .unwrap();
    let report = alsa_pcm_report(&dir);
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].severity, Severity::Info);
    let evidence = &report.findings[0].evidence;
    assert!(evidence.contains("48000 Hz"));
    assert!(evidence.contains("256 frames"));
    assert!(evidence.contains("1024 frames"));
    assert!(evidence.contains("512 samples"));
    assert!(evidence.contains("input"));
    assert!(evidence.contains("RUNNING"));
    assert!(!evidence.contains("123")); // Owner processes are not collected.
    std::fs::write(pcm.join("status"), "state: XRUN\n").unwrap();
    assert_eq!(alsa_pcm_report(&dir).findings[0].severity, Severity::Warn);
    std::fs::write(pcm.join("hw_params"), "closed\n").unwrap();
    std::fs::write(pcm.join("status"), "closed\n").unwrap();
    assert_eq!(
        alsa_pcm_report(&dir).findings[0].severity,
        Severity::Unavailable
    );
    std::fs::write(pcm.join("hw_params"), vec![b'x'; 16_385]).unwrap();
    assert_eq!(
        alsa_pcm_report(&dir).findings[0].severity,
        Severity::Unavailable
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn missing_alsa_inventory_cannot_report_healthy() {
    let report = alsa_pcm_report(std::path::Path::new("/nonexistent/aid-alsa-test"));
    assert!(report
        .findings
        .iter()
        .all(|f| f.severity == Severity::Unavailable));
}
