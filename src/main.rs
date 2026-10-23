use audio_interface_diag::{plan, Edition};

fn main() {
    if std::env::args().nth(1).as_deref() == Some("alsa") {
        let report = audio_interface_diag::alsa_pcm_report(std::path::Path::new("/proc/asound"));
        println!("{}", audio_interface_diag::render_json(&report));
        if report
            .findings
            .iter()
            .all(|f| f.severity == audio_interface_diag::Severity::Unavailable)
        {
            std::process::exit(1);
        }
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("devices") {
        let report = audio_interface_diag::native_inventory_report(
            audio_interface_diag::enumerate_native_devices(),
        );
        println!("{}", audio_interface_diag::render_json(&report));
        if report
            .findings
            .iter()
            .any(|f| f.severity == audio_interface_diag::Severity::Unavailable)
        {
            std::process::exit(1);
        }
        return;
    }
    if matches!(
        std::env::args().nth(1).as_deref(),
        Some("tone" | "noise" | "crosstalk")
    ) {
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
            eprintln!("usage: audio-interface-diag <live|daw|engineer>\n       audio-interface-diag tone <sample-rate> <frequency> <mono.f32le>\n       audio-interface-diag noise <sample-rate> <mono.f32le>\n       audio-interface-diag crosstalk <sample-rate> <frequency> <driven.f32le> <victim.f32le>\n       audio-interface-diag devices\n       audio-interface-diag alsa");
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
    let command = std::env::args().nth(1).unwrap();
    let args: Vec<_> = std::env::args().skip(2).collect();
    let expected = match command.as_str() {
        "noise" => 2,
        "crosstalk" => 4,
        _ => 3,
    };
    if args.len() != expected {
        return Err("usage: audio-interface-diag tone <rate> <frequency> <mono.f32le> OR noise <rate> <mono.f32le> OR crosstalk <rate> <frequency> <driven.f32le> <victim.f32le>".into());
    }
    let rate: f64 = args[0].parse()?;
    let report = if command == "noise" {
        audio_interface_diag::noise_report(&read_capture(&args[1])?, rate)
    } else {
        let frequency: f64 = args[1].parse()?;
        let samples = read_capture(&args[2])?;
        if command == "crosstalk" {
            audio_interface_diag::crosstalk_report(
                &samples,
                &read_capture(&args[3])?,
                rate,
                frequency,
            )
        } else {
            audio_interface_diag::tone_report(&samples, rate, frequency)
        }
    };
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

fn read_capture(path: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    use std::io::Read;
    // Bound reads even when the input is a pipe or grows after metadata is read.
    const MAX_BYTES: u64 = 32 * 1024 * 1024;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_BYTES || bytes.len() % 4 != 0 || bytes.is_empty() {
        return Err("capture must be nonempty mono little-endian float32, at most 32 MiB".into());
    }
    Ok(bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b))
        .collect())
}
