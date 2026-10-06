use audio_interface_diag::{plan, Edition};

fn main() {
    let edition = match std::env::args().nth(1).as_deref() {
        Some("live") => Edition::Live,
        Some("daw") => Edition::Daw,
        Some("engineer") | Some("tech") => Edition::Engineer,
        _ => {
            eprintln!("usage: audio-interface-diag <live|daw|engineer>");
            return;
        }
    };

    println!("Audio Interface Diag {:?}", edition);

    for test in plan(edition) {
        println!(
            "- {:?}{}{}",
            test.kind,
            if test.intrusive { " [active]" } else { " [passive]" },
            if test.requires_loopback_cable {
                " [loopback cable]"
            } else {
                ""
            }
        );
    }
}
