# Release Audit — 2026-10-07 (v0.0.1 beta)

Scope: full repository against `docs/specs/*`, the 2026-10-06 audit, CI, and release readiness for CLAP/VST3/standalone on Windows and Linux.

## Defects found and fixed

| ID | Severity | Defect | Fix |
|---|---|---|---|
| R-001 | High | Result model lacked `unavailable` and `not-run` statuses required by the product spec, so missing data could only be shown as FAIL/INFO. | `Severity::Unavailable` / `Severity::NotRun` + `label()`; reports render them; the plugin never shows PASS/HEALTHY without data. |
| R-002 | High | `estimate_delay_samples` returned a delay for any non-zero correlation, so an unconnected loopback produced a fabricated latency. | New `measure_delay` with peak-to-mean confidence gating (`MIN_DELAY_CONFIDENCE`); invalid runs are counted, not guessed. |
| R-003 | High | Signal-discipline rules (ramp in/out, -12 dBFS acknowledgement) had no implementation. | `apply_fade`, `validate_generator_level`, `DEFAULT_GENERATOR_DBFS`; the realtime generator ramps 20 ms in / 2 ms out and clamps to -12 dBFS until acknowledged. |
| R-004 | Medium | DATA-002 JSON/CSV reports missing. | `render_json`, `render_csv` (RFC 4180, JSON escaping) + plugin export of TXT/JSON/CSV/events CSV with context. |
| R-005 | Medium | `ui/` prototype simulated a live monitor with fake numbers and had placeholder (unwired) actions. Presenting non-measured data violates the spec. | Removed; replaced by the egui GUI wired to the realtime engine. |
| R-006 | Medium | No plugin/standalone build, no Windows CI, no release packaging; `Cargo.lock` uncommitted; version mismatch. | `plugin/` (nih-plug) + `xtask/`; CI workspace + Windows jobs; release workflow with installers/portables; lockfile committed; version 0.0.1. |
| R-007 | Low | Clippy `manual_is_multiple_of` on current stable. | Fixed. |

## Realtime design checks (plugin)

- `process()` performs no allocation, locking or I/O: atomics, a lock-free `rtrb` event ring and `try_lock` buffer swaps only. Buffers are preallocated in `initialize()`. Debug builds enable `assert_process_allocs`.
- Heavy analysis (latency cross-correlation, hum Goertzel) runs on the background task executor.
- Host re-activation (`reset`/`deactivate`) clears timing history so paused transport is not reported as a dropout; offline rendering disables timing faults; gaps >1 s are reported as "host paused", not dropouts.
- Live/DAW leave plugin audio untouched. Engineer output requires GUI arming (not a parameter, not persisted). Leaving Engineer mode or STOP disarms.
- Standalone does not monitor input to output unless enabled (feedback protection).

## Features added in this release

- Usability: report export (TXT/JSON/CSV/events) + clipboard + technician notes.
- Value: measured round-trip latency (5 runs, median/min/max/spread, confidence-gated).
- Fun: glitch-free streak and achievements.
- Extra: mains-hum (50/60 Hz family) observation.

## Remaining gaps (unchanged from 2026-10-06 unless noted)

Driver-reported xruns, OS device/session events, WASAPI/CoreAudio/ALSA-specific telemetry, buffer sweep, channel map, frequency response, THD, crosstalk, clock drift, burn-in, plots and hardware qualification still require the native backend layer. These appear in the Engineer catalog as "needs native backend" rather than as controls that do nothing.

## Validation performed

- `cargo test --workspace`, `cargo clippy --workspace --all-targets -D warnings`, `cargo fmt --check`: clean.
- `clap-validator 0.4.1` against the **debug** CLAP build (which aborts on any audio-thread allocation via `assert_process_allocs`): all process, parameter and descriptor tests pass. The three `state-reproducibility-*` tests fail identically on nih-plug's own unmodified `gain` example at the pinned revision, so this is a framework/validator-version mismatch rather than an AID defect. Parameters still save/restore in hosts through nih-plug's normal state path; revisit when nih-plug updates.
- GUI rendering could not be screenshot-verified in the headless build container (no GLX framebuffer config); verify on a desktop before promoting beyond beta.
