use audio_interface_diag::{plan, Edition};

fn main() {
    let edition=match std::env::args().nth(1).as_deref() {
        Some("live")=>Edition::Live,
        Some("daw")=>Edition::Daw,
        Some("engineer")|Some("tech")=>Edition::Engineer,
        _=>{ eprintln!("usage: audio-interface-diag <live|daw|engineer>"); return; }
    };
    println!("Audio Interface Diag {:?}",edition);
    for t in plan(edition) {
        println!("- {:?}{}{}",t.kind,if t.intrusive{" [active]"}else{" [passive]"},if t.requires_loopback_cable{" [loopback cable]"}else{""});
    }
}
