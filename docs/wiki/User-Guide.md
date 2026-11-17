# User Guide

- **Install:** download the Windows installer or Linux package from the GitHub Releases page. Or build (see the Developer Guide).
- **Live:** passive monitoring while performing: stage health, meters with peak hold, dropouts, clipping, signal loss, NaN/Inf, event timeline, markers, snapshot export.
- **DAW:** checks while the DAW controls the interface: correlation and polarity, DC offset, crest factor, mains hum, routing assistant, and dropout markers to match DAW logs.
- **Engineer / Technician:** armed generator (sine, pink, white, impulse; default -30 dBFS) and round-trip loopback latency (5 runs; median, min, max, spread). Follow "Using the loopback latency test" in the README.
- **Reports:** findings, technician notes, copy to clipboard, and export TXT, JSON or CSV to `Documents/AudioInterfaceDiag`.

**Safety:** Live and DAW modes never generate signal or alter audio. Leaving Engineer mode disarms immediately. Arming is never saved, so generation does not resume after a restart. In the standalone app, input is not monitored to the output unless you enable it, which prevents feedback.
