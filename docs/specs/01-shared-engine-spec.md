# Shared Diagnostic Engine Specification

## Real-time pipeline
Host backend -> callback ingress -> lock-free measurement tap -> bounded analysis queue -> analyzers -> event/result store -> UI/report.

The callback ingress records monotonic callback timestamps, frame count, stream time when supplied, overflow/underflow flags and discontinuity markers. It copies only the minimum data required by enabled analyzers into preallocated ring buffers.

## Signal analysis
Per channel:
- true sample peak approximation and sample peak
- RMS and moving RMS
- crest factor
- DC mean
- clipping runs and clipped sample count
- silence/stuck-sample detection
- drop/repeat/discontinuity signatures
- FFT/spectrum and spectrogram
- mains-family peaks (50/60 Hz plus harmonics) as observations, not definitive diagnoses

Stereo/pairs:
- Pearson correlation
- polarity inversion indication
- gain mismatch
- inter-channel delay estimate
- phase-versus-frequency when using a known stimulus

## Timing analysis
Measure callback interval, callback execution duration, deadline margin, worst/mean/p95/p99 timing, xrun/overflow/underflow counts, device reset/disconnects and effective sample delivery rate.

## Latency
Maintain three distinct concepts:
1. driver/API reported input/output latency
2. configured nominal buffer latency
3. measured analog/digital round-trip latency

Round-trip measurement uses a deterministic excitation (chirp/MLS/impulse) and correlation. Result includes samples and milliseconds plus uncertainty. Repeated runs yield median, spread and outliers.

## Clock
Estimate effective sample-rate error from long captures or dual-clock comparisons. Report ppm, accumulated sample drift and time drift. Never label normal asynchronous drift a hardware fault without a configured threshold and adequate duration.

## Active measurement generator
Generators: sine, dual tone, multitone, logarithmic sweep, linear sweep, impulse, chirp, MLS-like pseudorandom sequence, white noise, pink-noise approximation and silence.
Controls: channel routing, frequency, amplitude, duration, fade, repetition and emergency stop.

## Frequency response
Use known sweep and deconvolution/correlation path. Report magnitude by frequency and summary flatness over user-selected band. Measurements must state that interface input/output stages are measured as a combined loop when looped physically.

## Noise
Capture a configured silent path and report RMS/A-weighting only if implemented correctly, peak, spectrum and detected narrow-band components. Label results invalid if clipping, signal leakage or too-short capture invalidates the noise floor.

## THD / THD+N
Engineer mode only. Generate stable sine, window capture, identify fundamental and harmonics, compute ratios. Require sufficient FFT resolution and reject captures with clipping or clock instability beyond configured limits.

## Crosstalk
Drive one output/channel at a time while measuring all return channels. Build a matrix in dB relative to driven reference. Flag impossible routing or saturated captures as invalid.

## Diagnostics rules
Rules are deterministic and versioned. A finding cites the exact evidence. Thresholds are mode-specific and editable in Engineer mode. Example defaults:
- any xrun during a critical live window: fail event
- callback duration >80% of period: warn
- callback duration >=100% of period: fail
- repeated device reset: fail
- clipping: warn or fail depending on recurrence
No single CPU percentage alone may be blamed for an xrun.

## Raw evidence
Optional session trace stores timestamped metrics/events and compact analyzer summaries. Engineer mode can additionally retain WAV captures generated for a test.
