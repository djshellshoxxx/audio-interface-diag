# Requirements Traceability

Status key:

- **SPEC**: specified, not implemented.
- **CORE**: implemented in the portable core and covered by automated tests.
- **PARTIAL**: some implementation exists, but the requirement is not end-to-end complete.
- **NATIVE**: requires a platform audio backend and/or hardware qualification.
- **UI-SHELL**: presentation structure exists but is not wired to the native engine.

| ID | Requirement | Status |
|---|---|---|
| SAFE-001 | Live mode has no intrusive tests by default | CORE |
| SAFE-002 | DAW mode has no intrusive tests by default | CORE |
| SAFE-003 | Active tests require Engineer arming | CORE |
| SAFE-004 | Abort silences active output state | CORE |
| DSP-001 | RMS/peak dBFS | CORE |
| DSP-002 | DC offset | CORE |
| DSP-003 | stereo correlation/polarity basis | CORE |
| DSP-004 | delay correlation primitive | PARTIAL |
| DSP-005 | clock ppm | CORE |
| DSP-006 | sine and sweep generation | PARTIAL |
| DSP-007 | clipping, crest, gain mismatch | CORE |
| LIVE-001 | passive device/session monitoring | SPEC / NATIVE |
| LIVE-002 | glitch timeline and snapshots | UI-SHELL / NATIVE |
| DAW-001 | DAW-safe ownership policy | SPEC |
| DAW-002 | Windows process loopback | SPEC / NATIVE |
| DAW-003 | recording compensation workflow | SPEC / NATIVE |
| ENG-001 | buffer sweep | SPEC / NATIVE |
| ENG-002 | physical RTL test | PARTIAL / NATIVE |
| ENG-003 | channel map | SPEC / NATIVE |
| ENG-004 | noise/frequency/THD/crosstalk | SPEC / NATIVE |
| ENG-005 | drift and burn-in | PARTIAL / NATIVE |
| HOST-001 | generic native device enumeration | PARTIAL |
| HOST-002 | WASAPI-specific advanced telemetry | SPEC / NATIVE |
| HOST-003 | CoreAudio advanced telemetry | SPEC / NATIVE |
| HOST-004 | ALSA/JACK/PipeWire telemetry | SPEC / NATIVE |
| UI-001 | Live/DAW/Engineer screens | UI-SHELL |
| UI-002 | plots/timeline/help/tooltips | PARTIAL / UI-SHELL |
| DATA-001 | contextual result/evidence model | PARTIAL |
| DATA-002 | JSON/CSV/human reports | PARTIAL |
| QA-001 | unit tests for portable measurement core | CORE |
| QA-002 | hardware matrix qualification | SPEC / NATIVE |

## Audit interpretation

A UI control, enum value or planner entry does not count as feature completion. A diagnostic feature is complete only when its acquisition path, measurement logic, validity checks, result model, UI/reporting path and automated or hardware-backed verification are all present.

The 2026-10-06 full audit corrected earlier overstatement of native device enumeration and round-trip/clock/burn-in readiness. See `docs/audits/2026-10-06-full-audit.md`.
