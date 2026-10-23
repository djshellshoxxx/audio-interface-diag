# Background visual analysis and passive ALSA evidence

0.0.4 development validation in the Linux cloud:

- 68 workspace tests passed; formatting and all-target Clippy with warnings denied.
- Optimized CLAP/standalone/analyzer and self-contained MIT VST3 built; Steinberg
  validator passed 47/47, plus balanced entry lifecycle and legacy-state probes.
- Portable package extracted into a fresh temporary directory: embedded/direct
  CLAP hashes, Cargo.lock provenance, notices and all 47 SDK tests verified.
  Debian control version is `0.0.4~beta`.
- Cloud `alsa` CLI returned exit 1 with valid JSON unavailable evidence because
  `/proc/asound` is absent. Deterministic adapter fixtures cover reported format,
  delay, XRUN, closed/absent/oversized data and process-ID exclusion.
- Component benchmark: 1000 worker Hann-FFT/min-max windows took 0.191 seconds
  (0.191 ms/window) in this shared instance. Reproduce with
  `cargo run --release --example plot_benchmark`. This is not whole-app CPU or
  hardware qualification.

Visual analysis stays on the existing background capture task. The audio callback
only records capture metadata and swaps preallocated buffers. Invalid samples
invalidate a plot; old contexts and captures older than three seconds are hidden.
Capture-time ticks prevent queued analysis from making old evidence look fresh.
A regression reproduced retained partial capture after host reset; reset now
discards that partial window before new-context observations can be published.

Callback timing uses one relaxed atomic bin increment with fixed 64-bin storage.
The UI/report computes nearest-rank p95/p99 upper bounds outside the callback;
empty/overflow evidence is unavailable. These are measured AID execution bounds,
not driver xrun telemetry or exact unbinned percentiles.

The native release CI for this development branch and real desktop/hardware
qualification remain separate. See `docs/REMAINING-SPECS.md` for required missing
implementation, rather than treating hardware access as the only remaining work.
