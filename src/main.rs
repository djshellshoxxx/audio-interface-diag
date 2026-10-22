use audio_interface_diag::{plan, Edition};

fn main() {
    if std::env::args().nth(1).as_deref() == Some("tone") {
        if let Err(error) = analyze_capture() {
            eprintln!("{error}");
            std::process::exit(2);
        }
        return;
    }
    let edition = match std::env::args().nth(1).as_deref() {
        Some("live") => Edition::Live,
        Some("daw") => Edition::Daw,
        Some("engineer") | Some("tech") => Edition::Engineer,
        _ => {
            eprintln!("usage: audio-interface-diag <live|daw|engineer>\n       audio-interface-diag tone <sample-rate> <frequency> <mono.f32le>");
            return;
        }
    };

    println!("Audio Interface Diag {:?}", edition);

    for test in plan(edition) {
        println!(
            "- {:?}{}{}",
            test.kind,
            if test.intrusive {
                " [active]"
            } else {
                " [passive]"
            },
            if test.requires_loopback_cable {
                " [loopback cable]"
            } else {
                ""
            }
        );
    }
}

fn analyze_capture() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Read;
    let args: Vec<_> = std::env::args().skip(2).collect();
    if args.len() != 3 {
        return Err(
            "usage: audio-interface-diag tone <sample-rate> <frequency> <mono.f32le>".into(),
        );
    }
    let rate: f64 = args[0].parse()?;
    let frequency: f64 = args[1].parse()?;
    // Bound reads even when the input is a pipe or grows after metadata is read.
    const MAX_BYTES: u64 = 32 * 1024 * 1024;
    let mut bytes = Vec::new();
    std::fs::File::open(&args[2])?
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_BYTES || bytes.len() % 4 != 0 || bytes.is_empty() {
        return Err("capture must be nonempty mono little-endian float32, at most 32 MiB".into());
    }
    let samples: Vec<_> = bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b))
        .collect();
    let report = audio_interface_diag::tone_report(&samples, rate, frequency);
    println!("{}", audio_interface_diag::render_json(&report));
    if report
        .findings
        .iter()
        .any(|f| f.severity == audio_interface_diag::Severity::Unavailable)
    {
        std::process::exit(1);
    }
    Ok(())
}
