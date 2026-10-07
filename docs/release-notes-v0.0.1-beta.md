# Audio Interface Diag v0.0.1 beta

First public beta of Audio Interface Diag as a **CLAP plugin, VST3 plugin and standalone app** for Windows and Linux.

## Downloads

| Platform | Installer | Portable |
|---|---|---|
| Windows x64 | `AudioInterfaceDiag-0.0.1-beta-windows-x64-setup.exe` (installs CLAP to `Common Files\CLAP`, VST3 to `Common Files\VST3`, standalone to Program Files) | `AudioInterfaceDiag-v0.0.1-beta-windows-x64-portable.zip` |
| Linux x86_64 | `audio-interface-diag_0.0.1-beta_amd64.deb` (`/usr/lib/clap`, `/usr/lib/vst3`, `/usr/bin/audio-interface-diag`) | `AudioInterfaceDiag-v0.0.1-beta-linux-x86_64-portable.tar.gz` (run `install-user.sh` for a per-user install) |

Verify downloads with `SHA256SUMS.txt`. Builds are unsigned (beta): Windows SmartScreen may ask for confirmation.

## What it does

- **Live** (passive): stage-view health status, peak/RMS/peak-hold meters, suspected dropouts from callback gaps, clipping, signal loss, NaN/Inf, AID callback margin, event timeline and markers.
- **DAW** (passive): the same telemetry plus stereo correlation/polarity, DC offset, crest factor, mains-hum observation, routing assistant. Audio passes through untouched.
- **Engineer** (armed): sine/pink/white/impulse generator (-30 dBFS default, >-12 dBFS needs acknowledgement, ramped, persistent STOP) and **round-trip latency measurement** (5-run median/min/max/spread).

## New in this release

- Usability: one-click **report export** (TXT + JSON + CSV + events CSV) and copy-to-clipboard, with technician notes.
- Value: **measured round-trip latency** through a physical or sidechain loopback.
- Fun: **glitch-free streak and achievements**.
- Extra: **mains-hum (50/60 Hz family) detector**.

## Known limits (beta)

- Dropouts are *inferred* from callback timing; driver-reported xruns need native backends (not in this build).
- Buffer sweep, channel map, frequency response, THD, crosstalk, clock drift and burn-in still require the native backend layer.
- No hardware qualification has been performed yet; treat latency numbers as measurements of your specific host path.
- macOS is not built in this release.
