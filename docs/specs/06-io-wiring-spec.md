# Input / Output and Wiring Specification

## Logical routing
A test route is explicit:
Host API -> output device -> output channel(s) -> physical/digital path -> input device -> input channel(s).

Never infer that similarly named endpoints are physically connected.

## Physical loopback
For line-level tests, the wizard asks connector type and routes selected output to selected line input. Mic inputs require caution about gain/phantom state. Instrument/Hi-Z paths are treated as different signal chains and labeled accordingly.

## Output level
Default -30 dBFS. Wizard begins muted, opens stream, verifies state, fades signal up, runs measurement, fades down, then stops. Emergency stop writes silence and tears down generator state.

## Inputs
Capture gain is hardware-dependent. User records gain/control positions in report metadata. Software must detect clipping and reject invalid measurements.

## Stereo/channel map
Channel IDs are preserved exactly as host backend reports them. UI aliases (L/R, 1/2) do not replace raw IDs.

## Digital I/O
When digital ports are exposed, support channel/clock-lock metadata. Digital loopback results must be labeled separately from analog loopback because DAC/ADC latency and analog performance are bypassed.

## DAW routed mode
AID never assumes access to ASIO buffers owned by another process. The user can route a DAW bus to a free hardware/virtual input or export a recorded diagnostic file.

## Virtual devices
Virtual cable/mixer devices are allowed, but reports identify them as virtual and avoid claims about analog hardware.

## Multi-interface
Two-device measurement must identify clock relationship: shared clock, externally synchronized, or free-running/unknown.

## File I/O
Engineer mode can ingest WAV/AIFF/FLAC when decoder support is present for offline response/latency/compensation analysis. Exports: JSON, CSV and human-readable report; WAV raw capture is opt-in.
