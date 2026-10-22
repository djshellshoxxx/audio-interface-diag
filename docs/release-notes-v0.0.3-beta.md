# Audio Interface Diag v0.0.3 beta

This beta adds offline bench evidence and a proprietary-compatible VST3 build.
It does not complete the advanced native diagnostic roadmap or certify hardware.

## Added

- Offline `noise` analysis: DC-removed broadband RMS and 50/60 Hz family evidence,
  with explicit unavailable results for silence, short, clipped or invalid captures.
- Offline pairwise `crosstalk` analysis of matched coherent-tone captures. Reports
  relative leakage without claiming calibrated absolute isolation or a full matrix.
- Headless `devices` inventory exports advertised default-host capabilities as JSON,
  retaining unknown ranges/query errors. It does not open streams.
- Native macOS arm64 and Intel x64 release builds alongside Windows and Linux.

## Distribution and compatibility

Owned code is proprietary, all rights reserved, as selected by the owner.
NIH-plug's GPL VST3 bindings are disabled and removed from the locked dependency
graph. VST3 is built with pinned MIT CLAP wrapper, CLAP SDK and Steinberg VST3 SDK.
The CLAP implementation is private inside the VST3 bundle; it does not search for
or depend on a separate installed CLAP. Third-party notices, font licenses and
build provenance accompany the distribution.

The VST3 component ID and all eight automation IDs are retained. A bounded,
main-thread state adapter accepts legacy 0.0.2 raw JSON and CLAP framing and saves
raw JSON, preserving existing project-state framing. Invalid, truncated and
oversized states cannot reach the underlying allocation path. The migration
fixture was captured from the actual 0.0.2 VST3 binary with non-default values.

## Qualification limits

Advanced native buffer sweep, channel-map acquisition, frequency-response and
crosstalk acquisition, process loopback, platform telemetry, clock drift,
calibration profiles and persistent sessions still require implementation.
Physical loopback, real disconnects, desktop interaction, long real-time soak and
the hardware/device matrix remain unqualified. Offline analysis does not complete
those acquisition paths. macOS bundles are ad-hoc signed, without Apple
notarization; Windows installers are unsigned.
