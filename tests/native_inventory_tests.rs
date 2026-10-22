use audio_interface_diag::*;

#[test]
fn inventory_reports_advertised_capabilities_without_claiming_stream_qualification() {
    let device = NativeDeviceSummary {
        host_api: "ALSA".into(),
        name: "Test \"interface\"".into(),
        is_default_input: true,
        is_default_output: false,
        input_formats: Ok(vec![NativeFormatRange {
            channels: 2,
            min_rate: 44_100,
            max_rate: 96_000,
            sample_format: "F32".into(),
            buffer_frames: None,
        }]),
        output_formats: Err("not supported".into()),
    };
    let report = native_inventory_report(Ok(vec![device]));
    assert_eq!(report.findings[0].severity, Severity::Info);
    assert!(report.findings[0].evidence.contains("44100..96000 Hz"));
    assert!(report.findings[0].evidence.contains("unknown buffer range"));
    assert!(report.findings[0].evidence.contains("not supported"));
    assert!(report.findings[0]
        .action
        .contains("not stream-open qualification"));
    assert!(render_json(&report).contains("\\\"interface\\\""));
}

#[test]
fn missing_native_support_or_devices_is_unavailable() {
    for result in [
        Err("native-audio feature is not enabled".into()),
        Ok(vec![]),
    ] {
        let report = native_inventory_report(result);
        assert_eq!(report.findings[0].severity, Severity::Unavailable);
    }
}
