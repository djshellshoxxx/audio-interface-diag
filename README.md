# Audio Interface Diag

Audio Interface Diag is a sister diagnostic tool to DeckChek and MIDI Test. It is designed around one shared measurement engine with three operating editions:

- **Live** — passive, low-overhead monitoring while performing.
- **DAW** — non-disruptive production diagnostics while a DAW remains authoritative for the interface.
- **Engineer / Technician** — active loopback, latency, channel, noise, response, crosstalk, clock and burn-in qualification.

The repository currently contains the complete product/edition architecture specifications and the first tested diagnostic-core implementation.

## Current implemented core

- edition-specific test planner with Live/DAW non-intrusive guarantees
- RMS and peak dBFS measurement
- DC offset
- stereo correlation/polarity basis
- deterministic delay estimator for round-trip measurement primitives
- clock ppm calculation
- stream-health rules for xruns, discontinuities, callback margin and load
- text report rendering
- CI and unit tests

## Run

```bash
cargo test
cargo run -- live
cargo run -- daw
cargo run -- engineer
```

## Specifications

Start with:
- `docs/specs/00-product-spec.md`
- `docs/specs/02-live-edition-spec.md`
- `docs/specs/03-daw-edition-spec.md`
- `docs/specs/04-engineer-technician-spec.md`
- `docs/specs/05-gui-ux-spec.md`
- `docs/specs/06-io-wiring-spec.md`

Research basis is in `docs/research/open-source-and-platform-notes.md`.

## Architecture direction

Rust is used for the diagnostic core. Native audio backends remain isolated from DSP and result logic so that host-specific limitations do not contaminate measurements. The optional `native-audio` feature is reserved for CPAL-backed discovery/stream work while platform-specific adapters are added where a generic backend cannot expose required telemetry.

## Status

Foundation implementation is in progress. Hardware-active tests and the native GUI are the next implementation layer on top of the tested core.
