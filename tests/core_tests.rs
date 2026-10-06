use audio_interface_diag::*;

#[test]
fn rms_full_scale_sine_like_constant_is_zero_dbfs() {
    assert!((dbfs_rms(&[1.0,1.0,1.0,1.0]).unwrap()-0.0).abs()<1e-12);
}

#[test]
fn dc_offset_detects_bias() {
    assert!((dc_offset(&[0.25,0.25,0.25,0.25]).unwrap()-0.25).abs()<1e-12);
}

#[test]
fn correlation_detects_inverted_polarity() {
    let a=[-1.0,-0.5,0.5,1.0];
    let b=[1.0,0.5,-0.5,-1.0];
    assert!(correlation(&a,&b).unwrap() < -0.999);
}

#[test]
fn delay_estimator_finds_known_delay() {
    let reference=[1.0,0.0,-1.0,0.5];
    let captured=[0.0,0.0,1.0,0.0,-1.0,0.5,0.0];
    assert_eq!(estimate_delay_samples(&reference,&captured,5),Some(2));
}

#[test]
fn ppm_error_is_correct() {
    let ppm=ppm_error(48_000.0,48_000.48).unwrap();
    assert!((ppm-10.0).abs()<1e-6);
}

#[test]
fn live_plan_is_non_intrusive() {
    assert!(plan(Edition::Live).iter().all(|t| !t.intrusive));
}

#[test]
fn daw_plan_is_non_intrusive() {
    assert!(plan(Edition::Daw).iter().all(|t| !t.intrusive));
}

#[test]
fn engineer_plan_contains_loopback_measurement() {
    assert!(plan(Edition::Engineer).iter().any(|t| t.kind==TestKind::RoundTripLatency && t.requires_loopback_cable));
}

#[test]
fn xruns_are_a_failure() {
    let f=assess_stream(StreamStats{sample_rate:48_000.0,buffer_frames:128,callback_count:1000,xruns:1,discontinuities:0,max_callback_ms:0.5,cpu_load:0.2});
    assert!(f.iter().any(|x| x.id=="stream.xruns" && x.severity==Severity::Fail));
}

#[test]
fn clean_stream_passes() {
    let f=assess_stream(StreamStats{sample_rate:48_000.0,buffer_frames:128,callback_count:1000,xruns:0,discontinuities:0,max_callback_ms:0.4,cpu_load:0.2});
    assert_eq!(f.len(),1);
    assert_eq!(f[0].severity,Severity::Pass);
}
