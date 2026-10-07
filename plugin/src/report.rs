//! Builds findings from live telemetry using the shared core rules and exports reports.

use crate::shared::{Event, Shared, MINUS_INF_DB};
use crate::Mode;
use audio_interface_diag::{
    assess_stream, csv_field, json_escape, render_csv, render_json, render_text, summarize_runs,
    Finding, SessionSummary, Severity, StreamStats,
};
use std::path::PathBuf;
use std::sync::atomic::Ordering::Relaxed;
use std::time::{SystemTime, UNIX_EPOCH};

fn finding(
    id: &'static str,
    severity: Severity,
    title: &str,
    evidence: String,
    action: &str,
) -> Finding {
    Finding {
        id,
        severity,
        title: title.into(),
        evidence,
        action: action.into(),
    }
}

pub fn fmt_db(db: f32) -> String {
    if db <= MINUS_INF_DB + 0.5 {
        "-inf".into()
    } else {
        format!("{db:.1}")
    }
}

/// Builds every finding for the current session. Values are labelled as measured or inferred.
pub fn build_findings(shared: &Shared) -> Vec<Finding> {
    let mut findings = Vec::new();
    let callbacks = shared.callbacks.load(Relaxed);
    let sr = shared.sample_rate.load(Relaxed) as f64;
    let channels = shared.channels.load(Relaxed) as usize;
    let had_signal = shared.had_signal.load(Relaxed);

    if callbacks == 0 {
        findings.push(finding(
            "stream.no_data",
            Severity::Unavailable,
            "No audio callbacks observed yet",
            "The host has not processed audio through AID in this session.".into(),
            "Start playback/monitoring or enable the audio device, then re-check.",
        ));
    } else {
        let dropouts = shared.dropouts.load(Relaxed);
        let non_finite = shared.non_finite.load(Relaxed);
        findings.extend(assess_stream(StreamStats {
            sample_rate: sr,
            buffer_frames: shared.last_block.load(Relaxed),
            callback_count: callbacks,
            xruns: 0,
            discontinuities: dropouts + non_finite,
            max_callback_ms: shared.max_callback_ms.load(Relaxed) as f64,
            cpu_load: shared.avg_load.load(Relaxed) as f64,
        }));
        if dropouts > 0 {
            findings.push(finding(
                "stream.callback_gap",
                Severity::Warn,
                "Suspected dropouts (inferred from callback gaps)",
                format!(
                    "{dropouts} callback intervals exceeded 2.5x the buffer period. Inferred from \
                     timing, not driver-reported xruns."
                ),
                "Increase the buffer size, close competing apps, check USB/power, then re-test.",
            ));
        }
        if non_finite > 0 {
            findings.push(finding(
                "signal.non_finite",
                Severity::Fail,
                "NaN/Inf samples on input",
                format!("{non_finite} blocks contained non-finite samples (measured)."),
                "Bypass upstream plugins one at a time to find the source of invalid samples.",
            ));
        }
    }

    // Clipping (warn or fail depending on recurrence).
    let runs = shared.clip_runs.load(Relaxed);
    let clipped = shared.clipped_samples.load(Relaxed);
    findings.push(if !had_signal {
        finding(
            "signal.clipping",
            Severity::Unavailable,
            "Clipping check needs signal",
            "No non-silent input observed.".into(),
            "Send signal through AID to evaluate headroom.",
        )
    } else if runs == 0 {
        finding(
            "signal.clipping",
            Severity::Pass,
            "No clipping",
            format!(
                "Peak hold L {} / R {} dBFS (measured).",
                fmt_db(shared.peak_hold_db[0].load(Relaxed)),
                fmt_db(shared.peak_hold_db[1].load(Relaxed))
            ),
            "No corrective action required.",
        )
    } else {
        finding(
            "signal.clipping",
            if runs >= 10 {
                Severity::Fail
            } else {
                Severity::Warn
            },
            "Clipping detected",
            format!("{runs} clip runs, {clipped} samples at/above threshold (measured)."),
            "Reduce input gain or upstream level to restore headroom.",
        )
    });

    if had_signal {
        let dc = (0..channels.min(2))
            .map(|ch| shared.dc[ch].load(Relaxed).abs())
            .fold(0.0f32, f32::max);
        findings.push(finding(
            "signal.dc_offset",
            if dc > 0.01 {
                Severity::Warn
            } else {
                Severity::Pass
            },
            "DC offset",
            format!("Largest smoothed DC mean {dc:.5} (linear, measured)."),
            if dc > 0.01 {
                "Check the source/interface input for DC; enable a high-pass filter if needed."
            } else {
                "No corrective action required."
            },
        ));
    }

    if channels >= 2 && shared.correlation_valid.load(Relaxed) {
        let corr = shared.correlation.load(Relaxed);
        findings.push(finding(
            "signal.polarity",
            if corr < -0.5 {
                Severity::Warn
            } else {
                Severity::Info
            },
            "Stereo correlation",
            format!("Pearson correlation {corr:+.2} over the last 300 ms window (measured)."),
            if corr < -0.5 {
                "Strong negative correlation: check for a polarity-inverted channel or cable."
            } else {
                "Informational."
            },
        ));
    }

    let lost = shared.signal_lost.load(Relaxed);
    if lost > 0 {
        findings.push(finding(
            "signal.lost",
            Severity::Warn,
            "Signal lost",
            format!("{lost} transitions to >2 s digital silence after signal (measured)."),
            "Check cables, device connection and upstream routing at the marked times.",
        ));
    }

    let hum_hz = shared.hum_hz.load(Relaxed);
    findings.push(if hum_hz == 0 {
        finding(
            "noise.hum",
            Severity::Unavailable,
            "Mains hum observation",
            "Not enough signal analysed yet.".into(),
            "Let AID observe at least 1 s of input.",
        )
    } else {
        let rel = shared.hum_db_rel.load(Relaxed);
        let present = shared.hum_present.load(Relaxed);
        finding(
            "noise.hum",
            if present {
                Severity::Warn
            } else {
                Severity::Info
            },
            if present {
                "Mains hum observed"
            } else {
                "Mains hum not dominant"
            },
            format!(
                "{hum_hz} Hz family (f, 2f, 3f) at {rel:.1} dB relative to total input power; \
                 input RMS {} dBFS. Observation only, not a diagnosis.",
                fmt_db(shared.hum_signal_db.load(Relaxed))
            ),
            if present {
                "Check grounding, cable shielding, power supplies and ground loops."
            } else {
                "Informational."
            },
        )
    });

    let latency = shared.latency.lock().unwrap().clone();
    findings.push(match summarize_runs(&latency.runs_samples) {
        Some(s) => {
            let ms = |samples: f64| samples / latency.sample_rate as f64 * 1000.0;
            finding(
                "engineer.round_trip",
                if s.spread > 2.0 {
                    Severity::Warn
                } else {
                    Severity::Pass
                },
                "Measured round-trip latency",
                format!(
                    "median {:.0} samples ({:.2} ms), min {:.0}, max {:.0}, spread {:.0} samples \
                     over {} runs at {} Hz; {} invalid runs; last confidence {:.1}. Measured by \
                     noise-burst cross-correlation through the host path (includes host buffers).",
                    s.median,
                    ms(s.median),
                    s.min,
                    s.max,
                    s.spread,
                    s.runs,
                    latency.sample_rate,
                    latency.failures,
                    latency.last_confidence
                ),
                if s.spread > 2.0 {
                    "Latency is not stable between runs: check clocking/driver and repeat."
                } else {
                    "Use the median for recording-offset compensation."
                },
            )
        }
        None if latency.failures > 0 => finding(
            "engineer.round_trip",
            Severity::Unavailable,
            "Round-trip latency could not be measured",
            format!("{} runs had no clear return correlation.", latency.failures),
            "Check the loopback cable/route, return source and generator level.",
        ),
        None => finding(
            "engineer.round_trip",
            Severity::NotRun,
            "Round-trip latency not measured",
            "Run it from the Engineer screen with a loopback connected.".into(),
            "Optional.",
        ),
    });

    findings
}

pub fn summary(shared: &Shared, mode: Mode) -> SessionSummary {
    SessionSummary {
        edition: mode.edition(),
        findings: build_findings(shared),
    }
}

/// Civil date from a UNIX timestamp (UTC), avoiding a date/time dependency.
pub fn utc_timestamp(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

pub struct Context {
    pub timestamp: String,
    pub api: String,
    pub mode: Mode,
    pub sample_rate: f32,
    pub max_buffer: u32,
    pub last_block: u32,
    pub channels: u32,
    pub session_secs: f64,
    pub callbacks: u64,
}

pub fn context(shared: &Shared, mode: Mode) -> Context {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Context {
        timestamp: utc_timestamp(now),
        api: shared.plugin_api.lock().unwrap().clone(),
        mode,
        sample_rate: shared.sample_rate.load(Relaxed),
        max_buffer: shared.max_buffer.load(Relaxed),
        last_block: shared.last_block.load(Relaxed),
        channels: shared.channels.load(Relaxed),
        session_secs: shared.session_secs.load(),
        callbacks: shared.callbacks.load(Relaxed),
    }
}

pub fn render_full_text(shared: &Shared, mode: Mode, events: &[Event], notes: &str) -> String {
    let c = context(shared, mode);
    let mut out = format!(
        "Generated: {}\nAID version: {}\nHost wrapper/API: {} (host owns the device; values are \
         host-provided)\nMode: {:?}\nSample rate: {} Hz (host-reported)\nMax buffer: {} frames \
         (host-reported), last block {} frames (observed)\nChannels: {}\nSession: {:.1} s, {} \
         callbacks\n\n",
        c.timestamp,
        env!("CARGO_PKG_VERSION"),
        c.api,
        c.mode,
        c.sample_rate,
        c.max_buffer,
        c.last_block,
        c.channels,
        c.session_secs,
        c.callbacks
    );
    out.push_str(&render_text(&summary(shared, mode)));
    if !notes.trim().is_empty() {
        out.push_str(&format!("\nTechnician notes:\n{}\n", notes.trim()));
    }
    out.push_str("\nEvent timeline (session seconds):\n");
    for e in events {
        out.push_str(&format!(
            "  {:>9.2}s  {}  {:.2}\n",
            e.t,
            e.kind.label(),
            e.value
        ));
    }
    let dropped = shared.events_dropped.load(Relaxed);
    if dropped > 0 {
        out.push_str(&format!(
            "  ({dropped} events dropped: GUI was closed and the queue filled)\n"
        ));
    }
    out
}

pub fn render_full_json(shared: &Shared, mode: Mode, events: &[Event], notes: &str) -> String {
    let c = context(shared, mode);
    let events_json = events
        .iter()
        .map(|e| {
            format!(
                "{{\"t\":{:.4},\"kind\":\"{}\",\"value\":{}}}",
                e.t,
                json_escape(e.kind.label()),
                if e.value.is_finite() { e.value } else { 0.0 }
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"context\":{{\"timestamp\":\"{}\",\"version\":\"{}\",\"api\":\"{}\",\"mode\":\"{:?}\",\
         \"sample_rate\":{},\"max_buffer\":{},\"last_block\":{},\"channels\":{},\
         \"session_secs\":{:.3},\"callbacks\":{}}},\"notes\":\"{}\",\"report\":{},\"events\":[{}]}}",
        c.timestamp,
        env!("CARGO_PKG_VERSION"),
        json_escape(&c.api),
        c.mode,
        c.sample_rate,
        c.max_buffer,
        c.last_block,
        c.channels,
        c.session_secs,
        c.callbacks,
        json_escape(notes),
        render_json(&summary(shared, mode)),
        events_json
    )
}

pub fn render_events_csv(events: &[Event]) -> String {
    let mut out = String::from("t_secs,event,value\n");
    for e in events {
        out.push_str(&format!(
            "{:.4},{},{}\n",
            e.t,
            csv_field(e.kind.label()),
            e.value
        ));
    }
    out
}

pub fn report_dir() -> PathBuf {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    home.join("Documents").join("AudioInterfaceDiag")
}

/// Writes .txt, .json, findings .csv and events .csv. Returns the base path written.
pub fn export(shared: &Shared, mode: Mode, notes: &str) -> std::io::Result<PathBuf> {
    let events = shared.events();
    let dir = report_dir();
    std::fs::create_dir_all(&dir)?;
    let stamp = context(shared, mode).timestamp.replace(':', "-");
    let base = dir.join(format!("aid-report-{stamp}"));
    std::fs::write(
        base.with_extension("txt"),
        render_full_text(shared, mode, &events, notes),
    )?;
    std::fs::write(
        base.with_extension("json"),
        render_full_json(shared, mode, &events, notes),
    )?;
    std::fs::write(
        base.with_extension("csv"),
        render_csv(&summary(shared, mode)),
    )?;
    std::fs::write(
        dir.join(format!("aid-report-{stamp}-events.csv")),
        render_events_csv(&events),
    )?;
    Ok(base)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_is_correct() {
        assert_eq!(utc_timestamp(0), "1970-01-01T00:00:00Z");
        assert_eq!(utc_timestamp(1_791_331_200), "2026-10-07T00:00:00Z");
    }

    #[test]
    fn empty_session_is_unavailable_never_pass() {
        let (_, consumer) = rtrb::RingBuffer::new(4);
        let shared = Shared::new(consumer);
        let findings = build_findings(&shared);
        assert!(findings.iter().all(|f| f.severity != Severity::Pass));
        assert!(findings.iter().any(|f| f.id == "stream.no_data"));
        let json = render_full_json(&shared, Mode::Live, &[], "a \"note\"");
        assert!(json.starts_with('{') && json.ends_with('}'));
    }
}
