# DAW Edition Specification

## Goal
Diagnose production-system audio while the DAW remains authoritative for the interface.

## Ownership rule
AID must not attempt to open the DAW's exclusive/ASIO endpoint in a way that can interrupt the DAW. When direct observation is impossible, the UI says so and uses OS/session telemetry or an explicit routed measurement path.

## DAW session dashboard
Show:
- detected/selected DAW process
- active audio sessions where OS exposes them
- interface/API metadata
- project-facing sample rate and observed system endpoint rate where available
- xrun/glitch event timeline
- CPU callback margin for AID-owned diagnostic paths
- output levels from permitted loopback/process capture
- clipping, silence and discontinuities
- device/session disconnect events

## Windows process loopback
Where supported, provide opt-in capture restricted to the selected DAW process tree. This is a shared-mode Windows facility and must be labeled as such; it cannot be represented as direct ASIO bus capture.

## Routed diagnostic input
Provide a “DAW Send” workflow: user routes a dedicated DAW output/bus to AID or a loopback input. AID analyzes deterministic pulses/tones or normal program audio without controlling the DAW. The app shows routing instructions generically and stores the selected mapping.

## Recording compensation test
Offline/armed workflow:
1. Generate a calibrated transient/sequence.
2. User routes output to input physically or through a known digital loop.
3. DAW records it.
4. User loads/drops recorded file or sends returned path.
5. Compare expected and recorded event position.
6. Report offset in samples/ms and repeatability.
Do not call a DAW “wrong” unless the project/sample-rate/grid context is known.

## Production diagnostics
- sample-rate mismatch detector
- unexpected resampling warning where observable
- clipping and inter-sample-risk indicators
- stereo correlation/polarity
- noise/hum spectrum
- long-session glitches
- dropout markers suitable for matching to DAW log times
- A/B environment profiles for driver/buffer/interface/USB-port changes

## Export
Session report includes enough timestamps and configuration to attach to a DAW/interface support ticket.
