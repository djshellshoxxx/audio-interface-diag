# Development and QA — 2026-10-22

## Implemented in this pass

- Offline coherent-sine capture analysis with relative THD, full-band THD+N,
  fundamental RMS and DC; shared JSON report output and explicit unavailable
  results. No audio acquisition or output is started by this command.
- Bounded float32 capture import; command-line tests cover malformed, missing,
  silent and oversized files.
- Default-host native inventory now includes advertised input/output formats,
  channel counts, sample-rate and buffer ranges, with per-direction errors.
  Unknown buffer ranges remain unknown. Advertised does not mean qualified.
- Device scanning runs on a worker, preventing a slow driver query from blocking
  the GUI. Only one scan per editor is allowed at a time; a closed editor drops
  the result receiver without blocking on the driver.

## Verification

- Portable core/CLI tests: 41 passed.
- Core/CLI with `native-audio`: 42 passed, including capability conversion.
- Full workspace/all targets: 44 tests passed, including plugin report tests.
- DSP stress matrix: 44.1/48/96/192 kHz, three durations and four signed gain
  factors (48 configurations), with known harmonic and DC components.
- Invalid-signal cases: non-finite samples, clipping, silence, wrong frequency,
  non-coherent capture, insufficient length, invalid rates/frequencies.
- Full workspace/all targets Clippy with warnings denied and workspace formatting:
  passed.
- Independent JSON parsing of the CLI output: valid schema; a synthetic 1% second
  harmonic measured 0.999999% THD and THD+N.
- Review corrected the harmonic-bandwidth gate: reject a fundamental at or above
  one quarter of the sample rate, where no second harmonic is below Nyquist.

These checks do not qualify hardware, GUI rendering or real driver behavior.
Workspace plugin compilation and native lint passed. This document is not a
release certification; no desktop rendering, physical loopback or driver soak
test was performed.

## Remaining work

Native controlled stream opens and buffer sweep; process loopback and platform
telemetry; channel-map acquisition; frequency response/crosstalk workflows;
drift and persistent burn-in; compensation and calibration profiles; plots;
hardware/device-disconnect qualification and desktop GUI stress tests.
The offline distortion command does not complete those native workflows.
