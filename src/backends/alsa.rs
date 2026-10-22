//! Read-only Linux ALSA procfs evidence, separate from CPAL advertised formats.
//! Kernel-reported snapshots are not proof of stream stability or device quality.
use crate::{Edition, Finding, SessionSummary, Severity};
use std::io::Read;
use std::path::{Path, PathBuf};

const MAX_TEXT: u64 = 16_384;

fn text(path: &Path) -> Result<String, String> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(MAX_TEXT + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_TEXT {
        return Err("procfs field exceeded 16 KiB safety bound".into());
    }
    String::from_utf8(bytes).map_err(|_| "invalid UTF-8 procfs field".into())
}

fn numbered_dirs(root: &Path, prefix: &str, suffix: Option<char>) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        let Some(mut number) = name.strip_prefix(prefix) else {
            continue;
        };
        if let Some(suffix) = suffix {
            let Some(trimmed) = number.strip_suffix(suffix) else {
                continue;
            };
            number = trimmed;
        }
        if number.is_empty() || !number.bytes().all(|x| x.is_ascii_digit()) {
            continue;
        }
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        if paths.len() >= 256 {
            return Err("ALSA inventory exceeded bounded directory count".into());
        }
        paths.push(entry.path());
    }
    paths.sort();
    Ok(paths)
}

fn field<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines().find_map(|line| {
        line.strip_prefix(key)
            .and_then(|s| s.strip_prefix(':'))
            .map(str::trim)
    })
}

fn number(text: &str, key: &str) -> Result<u32, String> {
    let value = field(text, key)
        .and_then(|v| v.split_whitespace().next())
        .and_then(|v| v.parse::<u32>().ok())
        .filter(|v| *v > 0);
    value.ok_or_else(|| format!("missing or invalid ALSA {key}"))
}

fn unavailable(title: String, reason: String) -> Finding {
    Finding { id: "native.alsa_pcm", severity: Severity::Unavailable, title,
        evidence: reason, action: "No stream was opened or changed. Verify ALSA visibility and capture a running-stream snapshot; direct hardware and routed JACK/PipeWire evidence remain distinct.".into() }
}

fn pcm(root: &Path, path: &Path, card: &str, direction: &str) -> Finding {
    let title = format!(
        "ALSA {} {direction}",
        path.strip_prefix(root).unwrap_or(path).display()
    );
    let read = || -> Result<(String, String), String> {
        Ok((text(&path.join("hw_params"))?, text(&path.join("status"))?))
    };
    let (params, status) = match read() {
        Ok(v) => v,
        Err(error) => return unavailable(title, error),
    };
    if params.trim() == "closed" || status.trim() == "closed" {
        return unavailable(title, "Kernel reports a closed PCM substream; no current format or timing can be established.".into());
    }
    let fields = || -> Result<_, String> {
        let rate = number(&params, "rate")?;
        let channels = number(&params, "channels")?;
        let period = number(&params, "period_size")?;
        let buffer = number(&params, "buffer_size")?;
        if period > buffer {
            return Err("inconsistent reported period/buffer; stream may have reconfigured during observation".into());
        }
        let format = field(&params, "format")
            .filter(|v| !v.is_empty())
            .ok_or("missing ALSA sample format")?;
        let state = field(&status, "state")
            .filter(|v| !v.is_empty())
            .ok_or("missing ALSA stream state")?;
        Ok((rate, channels, period, buffer, format, state))
    };
    let (rate, channels, period, buffer, format, state) = match fields() {
        Ok(v) => v,
        Err(error) => return unavailable(title, error),
    };
    let delay = field(&status, "delay")
        .and_then(|v| v.parse::<i64>().ok())
        .map(|v| format!("{v} samples"))
        .unwrap_or_else(|| "unavailable".into());
    Finding { id: "native.alsa_pcm", severity: if matches!(state, "XRUN" | "DISCONNECTED" | "SUSPENDED") { Severity::Warn } else { Severity::Info }, title,
        evidence: format!("Kernel-reported ALSA PCM {direction} snapshot; card label {card}; state {state}; {channels} channels; {rate} Hz; format {format}; period {period} frames; buffer {buffer} frames; delay {delay}. Files are sampled separately and may change during observation. Path/card label is not a globally stable endpoint ID."),
        action: "Reported format/delay are not measured round-trip latency. No ownership, routing or settings changed. This does not observe a DAW's ASIO path or infer JACK/PipeWire capabilities.".into() }
}

/// Read a bounded procfs tree. The root parameter enables deterministic adapter
/// tests without devices; production callers use `/proc/asound` on Linux only.
pub fn alsa_pcm_report(root: &Path) -> SessionSummary {
    let mut findings = Vec::new();
    let collect = || -> Result<Vec<Finding>, String> {
        let mut result = Vec::new();
        for card in numbered_dirs(root, "card", None)? {
            let label = text(&card.join("id")).unwrap_or_else(|_| "unavailable".into());
            for (suffix, direction) in [('c', "input"), ('p', "output")] {
                for device in numbered_dirs(&card, "pcm", Some(suffix))? {
                    for stream in numbered_dirs(&device, "sub", None)? {
                        if result.len() >= 1024 {
                            return Err("ALSA inventory exceeded 1024 substreams".into());
                        }
                        result.push(pcm(root, &stream, label.trim(), direction));
                    }
                }
            }
        }
        Ok(result)
    };
    match collect() {
        Ok(result) if !result.is_empty() => findings = result,
        Ok(_) => findings.push(unavailable("ALSA PCM observation unavailable".into(), "No ALSA PCM substreams are exposed; this does not imply a healthy or faulty audio system.".into())),
        Err(error) => findings.push(unavailable("ALSA PCM observation unavailable".into(), error)),
    }
    SessionSummary {
        edition: Edition::Engineer,
        findings,
    }
}
