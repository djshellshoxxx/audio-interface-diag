# Requirements Traceability

Status key: SPEC = fully specified, CORE = implemented in portable core, NATIVE = requires platform backend/hardware qualification.

| ID | Requirement | Status |
|---|---|---|
| SAFE-001 | Live mode has no intrusive tests by default | CORE |
| SAFE-002 | DAW mode has no intrusive tests by default | CORE |
| SAFE-003 | Active tests require Engineer arming | CORE |
| SAFE-004 | Abort silences active output state | CORE |
| DSP-001 | RMS/peak dBFS | CORE |
| DSP-002 | DC offset | CORE |
| DSP-003 | stereo correlation/polarity basis | CORE |
| DSP-004 | delay correlation primitive | CORE |
| DSP-005 | clock ppm | CORE |
| DSP-006 | sine and sweep generation | CORE |
| DSP-007 | clipping, crest, gain mismatch | CORE |
| LIVE-001 | passive device/session monitoring | SPEC/NATIVE |
| LIVE-002 | glitch timeline and snapshots | SPEC/NATIVE |
| DAW-001 | DAW-safe ownership policy | SPEC |
| DAW-002 | Windows process loopback | SPEC/NATIVE |
| DAW-003 | recording compensation workflow | SPEC/NATIVE |
| ENG-001 | buffer sweep | SPEC/NATIVE |
| ENG-002 | physical RTL test | SPEC/NATIVE |
| ENG-003 | channel map | SPEC/NATIVE |
| ENG-004 | noise/frequency/THD/crosstalk | SPEC/NATIVE |
| ENG-005 | drift and burn-in | SPEC/NATIVE |
| HOST-001 | generic native device enumeration | CORE (optional CPAL layer) |
| HOST-002 | WASAPI-specific advanced telemetry | SPEC/NATIVE |
| HOST-003 | CoreAudio advanced telemetry | SPEC/NATIVE |
| HOST-004 | ALSA/JACK/PipeWire telemetry | SPEC/NATIVE |
| UI-001 | Live/DAW/Engineer screens | SPEC |
| UI-002 | plots/timeline/help/tooltips | SPEC |
| DATA-001 | contextual result/evidence model | SPEC |
| DATA-002 | JSON/CSV/human reports | SPEC |
| QA-001 | unit tests for portable measurement core | CORE |
| QA-002 | hardware matrix qualification | SPEC/NATIVE |

This matrix is maintained as implementation advances; a feature is not marked complete merely because a UI control exists.
