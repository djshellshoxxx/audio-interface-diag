# Audio Interface Diag — Product Specification

## Product purpose
Audio Interface Diag (AID) is a sister application to DeckChek, MIDI Test and other Circuit Drift Labs diagnostic tools. It diagnoses generic computer audio paths without assuming a manufacturer, model, DAW or driver API.

One measurement engine powers three editions/modes:
1. **Live**: safe, passive monitoring during a performance.
2. **DAW**: non-disruptive monitoring and production diagnostics while a DAW owns the interface.
3. **Engineer / Technician**: active qualification, loopback measurement, stress testing and detailed reports.

## Non-negotiable design rules
- Live and DAW modes must never change sample rate, buffer size, clock source, channel routing, gain, mute, default device, driver mode or exclusive ownership without an explicit user action.
- Active test signals are forbidden in Live/DAW mode unless the user enters a clearly labeled offline diagnostic workflow.
- Engineer mode may open devices, emit test signals and change supported stream settings only after showing what will happen.
- Audio callbacks must be allocation-free, lock-free where practical, non-blocking, and must never perform file/network/UI work.
- Every reported measurement must include context: API, device, direction, channel(s), sample rate, buffer/period, timestamp and measurement method.
- Distinguish **driver-reported** values from **measured** values.
- A failure must preserve raw evidence sufficient to reproduce or dispute the conclusion.
- Unknown/unavailable must never be rendered as pass.

## Supported host APIs
Target architecture:
- Windows: WASAPI shared/exclusive, WDM/KS where backend support is dependable, optional ASIO integration.
- macOS: CoreAudio.
- Linux: ALSA, JACK and PipeWire exposure where backend support permits.
- Virtual devices are treated as first-class devices and clearly labeled.

## Shared capability model
For each endpoint/backend retain:
- stable internal ID and OS ID where available
- friendly/vendor name
- host API and driver version
- input/output channel counts
- supported/default/current sample rates
- supported/default/current buffer or period sizes
- sample format
- shared/exclusive capability
- loopback capability
- hardware clock/source information where exposed
- active/inactive/disconnected state
- transport/topology metadata when OS exposes it

## Shared diagnostics
Device inventory, format capability validation, channel presence, RMS/peak/crest, clipping, DC offset, polarity/correlation, discontinuity detection, xruns, callback timing, CPU/load margin, sample-rate consistency, latency metadata, measured round-trip latency, clock drift, noise floor, frequency response, THD/THD+N where measurement conditions support it, crosstalk, channel mismatch, burn-in reliability, device-reset tracking and report generation.

## Result model
Every test result has:
- test ID/version
- status: pass/info/warn/fail/unavailable/not-run
- measurement values and units
- confidence/quality flags
- thresholds used
- environmental context
- evidence
- corrective actions
- references to raw capture/event data when retained

## Profiles
Users may save named environment profiles such as Direct USB, Hub, AC power, Battery, Driver A/B, Venue, Studio and Laptop. Comparison view highlights statistically meaningful changes and configuration changes.

## Safety and signal discipline
Signal generators default to -30 dBFS. Any test above -12 dBFS requires explicit acknowledgement inside Engineer mode. Output is ramped in/out. Abort, device disconnect, stream error or UI emergency stop immediately silences generated output.

## Persistence
Store preferences and reports locally. Raw audio capture is opt-in and off by default. Crash recovery must restore settings but never automatically resume signal generation.

## Acceptance
A build is releasable only when unit tests pass, live and DAW plans contain no intrusive tests by default, test generation cannot occur without an armed Engineer session, reports distinguish measured from reported values, and the application handles device disappearance without crashing.
