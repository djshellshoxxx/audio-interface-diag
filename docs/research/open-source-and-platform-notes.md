# Research Notes — Open Source and Platform References

Research date: 2026-10-06.

## PortAudio
PortAudio is a cross-platform open-source audio I/O library with callback and blocking APIs. Its repository includes device enumeration, minimum-latency, device QA, CPU-load and wiring/record/playback tests. Those patterns validate separating capability discovery, stability testing and workload testing in AID.

References:
- https://github.com/PortAudio/portaudio
- https://github.com/PortAudio/portaudio/wiki/BufferingLatencyAndTimingImplementationGuidelines

Design changes adopted:
- distinguish suggested/default latency from actual reported stream latency
- build a separate minimum-stable-buffer qualification test
- maintain host-API abstraction instead of assuming one latency model

## Friture
Friture is GPL-3.0 and provides real-time scope, spectrum and spectrogram analysis. AID adopts the product pattern of multiple simultaneous live analysis views, but no Friture source is copied because AID is independently implemented and licensing may differ.

Reference:
- https://github.com/tlecomte/friture

## JACK latency methodology
JACK documentation and PortAudio's latency notes reference physical loopback measurement via jack_iodelay. AID therefore treats physical round-trip measurement as distinct from software-reported latency and stores both.

Reference:
- https://jackaudio.org/
- PortAudio latency wiki above

## JUCE device-management concepts
JUCE AudioDeviceManager exposes current devices, setup state and CPU usage and provides a selector component. AID adopts the UX idea of a centralized device/setup model but uses its own engine/implementation.

Reference:
- https://docs.juce.com/master/classjuce_1_1AudioDeviceManager.html

## Windows WASAPI
Microsoft documents loopback capture of the system render mix, audio sessions, low-latency buffer/period discovery, and process-specific loopback capture on supported Windows versions. WASAPI loopback is shared-mode only; it cannot be treated as a transparent observer of an exclusive stream.

References:
- https://learn.microsoft.com/windows/win32/coreaudio/loopback-recording
- https://learn.microsoft.com/windows/win32/coreaudio/audio-sessions
- https://learn.microsoft.com/windows-hardware/drivers/audio/low-latency-audio
- https://learn.microsoft.com/samples/microsoft/windows-classic-samples/applicationloopbackaudio-sample/

Design changes adopted:
- DAW mode is passive-first and must not contend for exclusive/ASIO ownership
- Windows process-loopback is optional and explicitly labeled shared-mode observation
- endpoint/session disconnect events belong in the diagnostic timeline
- Windows backend should query modern periodicity capabilities where supported

## Licensing rule
Research notes may describe algorithms, concepts and observed product behavior. Source code from third-party projects is not copied unless its license is reviewed and the repository license strategy explicitly permits it.
