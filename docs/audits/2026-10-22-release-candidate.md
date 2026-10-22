# 0.0.2 beta candidate review

The release review found and reproduced two generator lifecycle failures:
host reset/deactivation and session clear retained the arming flag. The fixes
require fresh arming/level acknowledgement and cancel queued latency runs.
Context generations invalidate old results, including background captures that
finish after reconfiguration. Entered host compensation is also discarded.

Recording compensation now uses five stable same-rate loopback runs and an
explicitly entered host compensation value. Residual samples/ms are informational,
signed and exported. The app does not modify a DAW's settings.

Passive burn-in uses fixed-size state and monotonically increasing callback/fault
counters. An accelerated eight-hour scenario verifies 5,414,400 callbacks and
28 injected fault increments. A 50,000-event flood verifies bounded logs and
explicit reporting of dropped ring-buffer events. Counter reset, context change, no progress,
monitoring gaps and user cancellation cannot produce a completed clean result.
The existing one-second analysis tasks keep observation active with the editor
closed. If scheduling prevents continuity from being verified, the result is
unavailable. No raw audio or burn-in state is persisted after application exit.

Validation: full workspace tests (53), warnings-as-errors Clippy, formatting and
shell syntax passed during candidate development. Linux package smoke verified
the portable/.deb layout, inclusion of aid-analyze and Debian prerelease version.
Smoke package fixtures were isolated and discarded; they are not release assets.
Release workflow publication depends on Windows/Linux/macOS build jobs and
creates a draft beta candidate from the exact release-branch commit.

Review covered correctness, boundary validation, race/stale-data handling,
fixed-size monitoring state, audio-thread separation and packaging. No dependency
versions changed; Cargo.lock changes only the two application package versions.

This is a beta candidate, not full roadmap completion. Native buffer sweep,
channel-map/frequency-response/crosstalk acquisition, process loopback, platform
telemetry and clock drift remain incomplete. Desktop rendering, real disconnects,
physical loopback, real-time soak and the device qualification matrix remain
unverified. A generally qualified release is blocked on those hardware checks.
