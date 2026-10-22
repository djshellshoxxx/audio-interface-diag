# Audio Interface Diag v0.0.2 beta candidate

This candidate extends the existing Live/DAW/Engineer application and
CLAP/VST3 plugins. It is not the completed advanced diagnostic roadmap.

## Added

- Offline coherent-sine distortion analysis: relative THD/THD+N, RMS and DC,
  JSON evidence and invalid-capture rejection. Downloads include `aid-analyze`
  (`aid-analyze.exe` on Windows). See the offline bench specification.
- Background device inventory with advertised input/output channel, rate,
  sample-format and buffer ranges. Enumeration does not certify stream opens.
- Recording-offset aid: enter host compensation for the measured loopback path;
  five stable runs produce the signed residual in samples/ms. No DAW setting is
  changed. Recheck your DAW's sign convention with a test recording.
- Passive host-path burn-in with 10 min/1 h/8 h/custom durations, fault counters,
  interruption/reset detection and exported findings. It generates no signal,
  retains no raw audio and does not qualify driver xruns or clock drift.

## Fixed

- Reset, deactivation, reinitialization and session clear now require fresh
  generator arming and high-level acknowledgement.
- Host context changes invalidate old loopback results and queued capture analysis.
- Frequencies without measurable harmonic bandwidth cannot report zero THD.
- Device scans no longer block the editor; duplicate device names have separate
  capability disclosure controls.

## Qualification limits

Desktop GUI rendering, physical loopback, unplug/replug, long real-time soak and
the device matrix require real hardware qualification. Automated simulated
stress does not certify those conditions. Native buffer sweep, channel mapping,
process loopback, platform-specific telemetry, frequency-response/crosstalk
acquisition and clock-drift workflows remain future work, labelled unavailable.

Do not promote this beta to a generally qualified release until the hardware
checkpoint in `tasks/todo.md` is complete.

The core manifest still declares `LicenseRef-Proprietary-TBD`; final distribution
licensing/notice metadata remains unresolved. Release-branch builds are drafts.
