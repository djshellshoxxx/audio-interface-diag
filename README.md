# Audio Interface Diag

Audio Interface Diag (AID) is a sister diagnostic tool to DeckChek and MIDI Test. One shared measurement engine powers three modes:

- **Live** — passive, low-overhead monitoring while performing.
- **DAW** — non-disruptive production diagnostics while the DAW remains authoritative for the interface.
- **Engineer / Technician** — armed active tests such as loopback round-trip latency.

AID ships as a **CLAP plugin, VST3 plugin and standalone application** (Windows and Linux). Downloads are on the GitHub Releases page.

## Features

| Area | Feature |
|---|---|
| Live | Stage view health (text + colour, never "healthy" without data), meters with peak hold, suspected dropouts, clipping, signal loss, NaN/Inf, AID callback margin, event timeline, markers, snapshot export |
| DAW | Correlation/polarity, DC offset, crest factor, mains hum, routing assistant, dropout markers for matching DAW logs |
| Engineer | Test catalog from the shared planner, armed generator (sine/pink/white/impulse, -30 dBFS default, >-12 dBFS acknowledgement, ramps, STOP), round-trip latency (5 runs, median/min/max/spread, confidence-gated), passive host-path burn-in |
| Devices | Host-provided format context and background native inventory with advertised input/output channel, rate, sample-format and buffer ranges |
| Report | Findings via the shared rule engine, technician notes, copy to clipboard, export TXT/JSON/CSV + events CSV to `Documents/AudioInterfaceDiag` |
| Badges | Glitch-free streak and achievements |

Safety: Live/DAW never generate signal or alter audio; host reset/deactivation/session clear require fresh arming; leaving Engineer mode disarms instantly; arming is never saved, so generation never resumes after a restart. In the standalone app, input is not monitored to the output unless you enable it (prevents feedback).

## Using the loopback latency test

1. Engineer screen → connect interface **Output 1 → Input 1** with a cable (or route a hardware return to AID's sidechain and set *Return source = Sidechain*).
2. Keep the level at -30 dBFS to start. Click **Arm generator…**, check the summary, click **ARM**.
3. Click **Measure (5 runs)**. Results show in samples and ms; invalid runs (no clear return) are counted, never guessed.

## Build

Offline sine-capture distortion measurement is available through
`aid-analyze tone 48000 1000 capture.f32le` (or
`cargo run -- tone 48000 1000 capture.f32le` from source). It exports JSON with relative
THD/THD+N and rejects invalid captures. See
[capture format and measurement limits](docs/specs/11-offline-bench-analysis.md).

```bash
cargo test --workspace
cargo run -- live                          # CLI planner
cargo xtask bundle aid_plugin --release    # CLAP, VST3 and standalone in target/bundled
packaging/linux/package.sh v0.0.2-beta     # Linux .deb + portable tarball in dist/
```

Linux build dependencies: `libasound2-dev libjack-jackd2-dev libgl-dev libx11-xcb-dev libxcursor-dev libxcb-icccm4-dev libxcb-dri2-0-dev libxkbcommon-dev pkg-config`.

Pushing a `v*` tag runs `.github/workflows/release.yml`, which builds and publishes installers and portable builds for Windows and Linux.

## Layout

- `src/` — portable diagnostic core (DSP, rules, planner, session state, reports). No audio-thread or GUI code.
- `plugin/` — nih-plug CLAP/VST3/standalone wrapper: realtime engine, egui GUI, report export.
- `xtask/` — bundler. `packaging/` — installers.
- `docs/specs/` — product specifications; `docs/audits/` — audit records.

## Candidate qualification

The 0.0.2 beta candidate has automated software QA; hardware/desktop qualification
and the remaining native diagnostic roadmap are outstanding. See
[release notes](docs/release-notes-v0.0.2-beta.md) for the exact supported scope.
