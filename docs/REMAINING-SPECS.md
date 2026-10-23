# Required implementation still outstanding

The 0.0.3 beta and 0.0.4 development work do not complete specs 00–09. This list
keeps implementation work separate from hardware qualification. A planner entry
or unavailable UI label is not an implemented measurement path.

## Software implementation

| Area | Required work still missing or incomplete |
| --- | --- |
| Windows host adapter | Stable endpoint/session inventory, current/shared/exclusive capabilities, IAudioClient3 periods, default/format/device events and supported process-tree loopback. Direct ASIO observation must never be inferred from shared-mode capture. |
| macOS host adapter | CoreAudio device/stream UIDs, rate/buffer/latency/safety-offset/clock metadata and property/device listeners. |
| Linux host adapter | Stable endpoint identity, ALSA stream/event integration and distinct JACK/PipeWire routing/telemetry. The 0.0.4 PCM procfs snapshot is read-only reported evidence, not these complete backends. |
| Active orchestration | Controlled capability opens, descending buffer stability sweep, channel-map acquisition, gain checkpoints, frequency-response/deconvolution, full crosstalk matrix and clock/drift acquisition with arming, validity gates and abort/device-loss handling. |
| Signal analysis | Inter-sample/true-peak approximation, stuck/repeat/discontinuity signatures, spectrogram, inter-channel/phase response and integrated timing/drift plots. Scope/Hann FFT and timing percentiles cover only part of this area. |
| Stimulus families | Dual/multitone, logarithmic sweep/chirp/MLS and configurable test duration/repetition. Existing sine/pink/white/impulse remain supported. |
| Storage | Named environment/calibration profiles, session identity and retention, compatible comparisons, crash report recovery and explicitly opt-in raw evidence. Restart must never restore output arming. |
| Result model | Fully structured per-test values/units, quality/threshold/method/context records, wall-clock/monotonic event correlation and evidence references. Current findings/report context is partial. |
| Snapshot | Bounded previous-and-next telemetry intervals around a marker; the immediate exported report alone is not this workflow. |
| UI | Overview/Sessions/Compare/Settings, DAW process/session selection, saved routing mappings, remaining test-state/result-tray flows and plot families. |
| Accessibility | Full keyboard/screen-reader coverage, scalable controls, reduced-motion settings and numeric alternatives for all remaining plots. Scope/spectrum now have numeric alternatives. |

ASIO is optional in spec 08 and has separate redistribution requirements.
WDM/KS and ETW/correlation are conditional on dependable backend/API support and
permissions. Unsupported features must stay explicitly unavailable.

## Qualification rather than missing source code

- Physical loopback and calibrated reference measurements; real disconnect/reset
  and device switching; native process/driver behavior on supported platforms.
- Windows onboard/USB/vendor-ASIO/virtual-device, macOS built-in/USB and Linux
  ALSA/JACK/PipeWire hardware matrix at 44.1/48/96 kHz and exposed buffer ranges.
- Desktop/DAW interaction and accessibility checks, real-time long-session soak
  and full application CPU/latency overhead. The worker FFT/scope component
  benchmark and accelerated fault tests do not certify these conditions.
- Apple notarization and Windows code signing if those distribution trust
  guarantees are required; current beta binaries make no such claim.
