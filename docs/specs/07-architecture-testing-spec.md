# Architecture and Testing Specification

## Modules
- host: platform/audio API adapters
- realtime: callbacks, ring buffers and event stamps
- dsp: levels, correlation, FFT, sweep/deconvolution, distortion and drift
- tests: test orchestration/state machines
- rules: deterministic findings
- storage: profiles/sessions/reports
- ui: presentation only
- export: JSON/CSV/report generation

The UI must not contain measurement logic.

## State machine
Idle -> Preparing -> Ready -> Armed (active tests only) -> Running -> Finalizing -> Complete.
Any state -> Aborted on stop/device loss/fatal stream error. Active output is possible only in Armed/Running.

## TDD
Pure DSP/rules/planner behavior is unit-tested first. Host adapters use fakes for deterministic integration tests plus platform hardware smoke tests. Every bug gets a reproduction test.

## Required automated tests
- edition plans are non-intrusive for Live/DAW
- Engineer active tests require arming
- RMS/peak/DC/correlation known vectors
- latency estimator known delay
- clock ppm known ratios
- clipping/discontinuity detection
- report serialization preserves units/context
- device removal does not panic
- bounded queues drop/mark telemetry rather than blocking callback
- generator stops on abort
- invalid measurement conditions cannot produce PASS

## Hardware QA matrix
Windows: onboard audio, common USB class-compliant interface, vendor ASIO interface, virtual audio device.
macOS: built-in + USB interface.
Linux: ALSA + PipeWire/JACK where available.
Rates: 44.1/48/96 kHz minimum.
Buffers: stable range exposed by hardware.

## Performance tests
Measure callback cost, analysis throughput, UI update overhead, memory growth during 8-hour simulated session, report size and burn-in event volume.

## CI
Format, warnings-as-errors where practical, unit tests and release build. Platform-specific native-audio jobs are introduced once backends are implemented and runners have required system packages.
