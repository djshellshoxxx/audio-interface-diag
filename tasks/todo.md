# Release tasks

- [x] Lifecycle safety: fresh arming required after all host/session lifecycle changes;
  pending latency work cancelled. Verify plugin regression tests and workspace lint.
- [x] Compensation: valid repeated measurements plus entered host compensation yield
  a signed residual in samples/ms; stale rates and invalid inputs are unavailable.
  Verify known offsets and invalid/stale-input tests, UI and report integration.
- [x] Passive burn-in: observe callback progress for the requested duration, retain
  fault deltas and mark resets/interruptions unavailable. Verify accelerated 8 h
  telemetry, reset/no-data/abort cases, UI and exported findings.
- [ ] Release checkpoint: workspace tests, Clippy, formatting, package inspection and
  updated requirement traceability; cross-platform CI must pass before integration.
- [ ] Hardware checkpoint: desktop GUI, physical loopback, real disconnects and
  device matrix. Remains unqualified until real measurements are available.
- [ ] Distribution metadata: resolve the core manifest's existing
  `LicenseRef-Proprietary-TBD` and final distribution notices before public release.
