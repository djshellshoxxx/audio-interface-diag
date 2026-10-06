# Full Repository Audit — 2026-10-06

Audited scope: repository structure, all Rust source, current automated tests, CI workflow, static UI shell, product/edition specifications, platform backend specification, I/O/wiring specification, data/report specification, research notes and requirements traceability.

## Executive result

The repository is a sound **foundation**, not yet a complete audio-interface diagnostic application. The portable diagnostic core contains useful tested primitives and safety state logic, while most hardware-facing diagnostics remain specifications awaiting native stream/backend implementation.

The audit found several correctness and QA defects in the foundation. They have been corrected on the audit branch and are protected with regression tests.

## Defects corrected

### A-001 — Invalid stream telemetry could produce PASS
Severity: High.

A zero/invalid sample rate, zero buffer size, non-finite callback timing or non-finite CPU load could fall through the rule engine and result in `stream.clean`.

Fix: invalid telemetry now produces `stream.invalid_stats` FAIL and exits assessment before any clean result. Zero callback count is also invalid because no stability claim can be supported without observations.

### A-002 — Callback deadline misses were only warnings
Severity: High.

A callback duration at or beyond the audio period indicates that the real-time deadline was missed. The previous rule used one >80% warning threshold.

Fix: >=100% period is now FAIL; >80% and <100% remains WARN.

### A-003 — Non-finite DSP inputs were accepted
Severity: High.

NaN/Infinity in level or generator inputs could propagate through measurements or generated samples.

Fix: RMS, peak, DC, correlation, ppm and signal-generation paths reject non-finite measurement parameters.

### A-004 — Silent/non-finite latency input could produce a delay
Severity: High.

The delay primitive could return offset zero for a zero-energy capture and could select a result after NaN contamination.

Fix: non-finite vectors are rejected and zero-correlation measurements return unavailable rather than a fabricated delay.

### A-005 — Active native feature was outside CI
Severity: Medium.

Default CI never compiled the optional CPAL `native-audio` code.

Fix: CI now has a native-audio job that installs ALSA development headers on Ubuntu and tests/clippies the feature.

### A-006 — Rust formatting gate did not match source state
Severity: Medium.

CI required `cargo fmt --check` while the initial source was not maintained in rustfmt-style formatting.

Fix: Rust source/tests were reformatted and CI explicitly installs rustfmt/clippy components.

### A-007 — UI relied on implicit element globals
Severity: Medium.

The static UI JavaScript relied on browser-created globals for DOM IDs, an avoidable portability/maintenance hazard.

Fix: all DOM elements are explicitly resolved.

### A-008 — Live monitor survived mode changes
Severity: Medium.

A simulated Live monitor timer could continue after switching to DAW/Engineer/other views, leaving inconsistent UI state.

Fix: changing away from Live stops the monitor, resets timing state and records an event.

### A-009 — Native device summary repeated default device names
Severity: Low/Medium.

Every enumerated device contained copies of the default input/output names, which did not express whether that row itself was the default.

Fix: summary now records `is_default_input` and `is_default_output`.

## Test coverage added or strengthened

Regression coverage now includes:

- invalid sample-rate/stream telemetry cannot PASS
- non-finite stream values cannot PASS
- zero callbacks cannot PASS
- callback deadline miss is FAIL
- non-finite level samples return unavailable
- non-finite/zero-length signal generation is rejected
- silent/non-finite delay measurements return unavailable
- passive sessions cannot be armed
- non-finite ppm inputs are rejected
- original arming/abort, RMS/DC/correlation, delay, gain mismatch and planner tests remain

CI also performs JavaScript syntax validation with `node --check ui/app.js`.

## Remaining implementation gaps

### Native acquisition and real-time engine
No end-to-end audio callback/ring-buffer/analysis queue exists yet. CPAL currently enumerates the default host only. Live meters, xruns, callback timing and device events are not driven from real audio telemetry.

### Windows
WASAPI endpoint/session events, process loopback, IAudioClient3 period queries, exclusive/shared behavior and optional ASIO support remain unimplemented.

### macOS
CoreAudio device/property/buffer/latency integration remains unimplemented.

### Linux
Direct ALSA, JACK and PipeWire-specific telemetry/routing remain unimplemented beyond generic CPAL discovery.

### Engineer measurements
Round-trip latency has only a basic delay primitive. Buffer sweep, real physical loopback orchestration, channel mapping, calibrated noise, frequency response/deconvolution, THD/THD+N, crosstalk, clock-duration measurement and burn-in execution are not end-to-end implemented.

### Generator safety
Signal math exists, but no hardware output path, fade/ramp state machine or actual emergency stream-silence path exists yet. The current `output_enabled` flag is a state guard, not proof that a native output callback is silenced.

### Reports/data
A basic human-readable text renderer exists. Canonical JSON schema, CSV export, persistent sessions, raw evidence references, retention and support-bundle generation remain unimplemented.

### GUI
The UI is a static prototype shell. It is not packaged as Tauri/JUCE/native desktop software and has no bridge to the Rust engine. Device, DAW, Engineer and Reports actions remain placeholders. Spec-required Overview, Sessions, Compare, Settings and Help/tooltips are not implemented.

### Hardware QA
No real-interface qualification has been performed. No claim about latency accuracy, THD, frequency response, clock ppm accuracy or long-duration stability should be made until hardware-backed validation exists.

## Repository/release hygiene findings

- `Cargo.lock` is not committed. For a shipping application this should be generated and committed once the dependency set is stabilized.
- Package metadata still uses `LicenseRef-Proprietary-TBD`; final licensing has not been selected/applied.
- There is no release packaging, signing or installer workflow.
- There is no platform build matrix yet.
- There is no benchmark/performance harness for callback budget, memory growth or long-duration event volume.

## Completion assessment

Specification coverage is high: the major product, edition, engine, UI, wiring, platform, data and QA areas are documented.

Working-product coverage is much lower. The current repository should be considered an **early engineering foundation / pre-alpha**, because hardware acquisition and most end-to-end diagnostic workflows are not implemented.

The next implementation priority should be the native stream abstraction plus a fake stream backend. That enables deterministic integration testing first, then real CPAL/WASAPI/CoreAudio/ALSA adapters without putting platform logic inside DSP or UI code.
