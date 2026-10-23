//! Optional component benchmark, not a whole-application CPU qualification.
//! Run: cargo run --release --example plot_benchmark
use audio_interface_diag::{analyze_spectrum, scope_envelope};

fn main() {
    const ITERATIONS: usize = 1000;
    let samples: Vec<f32> = (0..48_000)
        .map(|i| (std::f64::consts::TAU * 750.0 * i as f64 / 48_000.0).sin() as f32 * 0.5)
        .collect();
    let begin = std::time::Instant::now();
    for _ in 0..ITERATIONS {
        std::hint::black_box(
            analyze_spectrum(std::hint::black_box(&samples), 48_000.0, 4096).unwrap(),
        );
        std::hint::black_box(scope_envelope(std::hint::black_box(&samples), 256).unwrap());
    }
    let elapsed = begin.elapsed().as_secs_f64();
    println!(
        "{ITERATIONS} worker FFT+scope windows: {elapsed:.3} s; mean {:.3} ms/window",
        elapsed * 1000.0 / ITERATIONS as f64
    );
}
