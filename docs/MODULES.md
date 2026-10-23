# Measurement and adapter boundaries

- `analysis.rs`, `bench.rs`, `noise.rs`, `crosstalk.rs`: portable evidence analysis;
  no audio devices or GUI. Offline analyses allocate only on caller/worker threads.
- `spectrum.rs`: bounded Hann FFT and min/max envelope for worker-thread displays;
  explicit amplitude units and display floor, without a device-quality judgement.
- `timing.rs`: allocation-free timing bucketing and non-real-time percentile bounds;
  fixed storage, invalid/empty/overflow evidence is unavailable.
- `native.rs`: advertised default-host inventory; it does not open streams or
  establish advanced backend telemetry/stable device identity.
- `plugin/src/lib.rs`: host lifecycle, bounded callback work and background tasks.
  Arming is transient and context changes invalidate prior diagnostic results.
- `plugin/src/shared.rs`: atomic telemetry, bounded events and worker/UI snapshots;
  scope/spectrum are generation-tagged and expire on wall-clock time.
- `plugin/src/plots.rs`: read-only egui presentation. Plotting cannot start audio,
  change host settings or run measurement logic.
- `packaging/vst3/entry.cpp`: private-bundle CLAP loader and balanced entry lifecycle.
  No filesystem/loading work occurs in processing callbacks.
- `packaging/vst3/aid_state.h`: bounded main-thread framing between legacy VST3
  JSON and NIH CLAP state. IDs/state framing are regression-tested against a real
  synthetic legacy state fixture; malformed input is rejected before allocation.
- `backends/alsa.rs`: bounded read-only ALSA PCM evidence with a replaceable procfs
  root for deterministic tests. It retains kernel-reported format/state/delay,
  excludes process IDs, never opens streams and never claims a stable endpoint ID
  from a transient card index.
