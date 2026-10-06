# Platform Backend Specification

## Contract
Every backend implements discovery, capability query, passive observation where the OS permits it, stream open/close, event reporting and stable endpoint IDs. Unsupported operations return explicit unavailable reasons.

## Windows
### WASAPI
Use modern Core Audio interfaces. Support endpoint enumeration, shared/exclusive capability inspection, mix format, current/default endpoint changes, session events and shared-mode loopback. Query IAudioClient3 periodicity/buffer capabilities where available.

### Process loopback
Optional DAW helper uses the supported process-tree loopback API. UI must identify this as Windows shared-mode process capture and not direct ASIO observation.

### ASIO
ASIO support is an optional backend because redistribution/build requirements differ from WASAPI. It must never probe or seize an ASIO device already owned by a DAW in DAW mode. Engineer mode may open it only after user confirmation.

### ETW/event correlation
Where permissions and APIs allow it, correlate audio/driver glitch and device events to the AID timeline. Correlation is evidence, not automatic causation.

## macOS
CoreAudio backend: devices, streams, nominal/actual rates where exposed, buffer frame size/range, latency/safety offsets, default-device and property-change listeners. Aggregate devices are labeled.

## Linux
ALSA backend for direct hardware capabilities; JACK/PipeWire-aware discovery for routed production systems. Never infer ALSA hardware capability from a PipeWire virtual endpoint.

## Backend capability flags
Each backend advertises:
- passive_monitor
- input_capture
- output_render
- system_loopback
- process_loopback
- exclusive
- buffer_query
- buffer_change
- sample_rate_query
- sample_rate_change
- clock_metadata
- xrun_events
- device_events

The test planner uses flags to disable impossible workflows before execution.
