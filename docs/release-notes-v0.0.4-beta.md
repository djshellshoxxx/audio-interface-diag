# Audio Interface Diag 0.0.4 development candidate

This follow-on development branch adds bounded background scope/spectrum and
callback-timing distribution evidence. It is separate from the 0.0.3 release
candidate and is not a claim that the full native diagnostic roadmap is complete.

- The existing preallocated one-second mono capture feeds a worker-thread Hann
  FFT and min/max envelope. No FFT, allocation or plot work runs in the audio
  callback. Plots expire after three seconds or a host/session context change;
  invalid captures remain unavailable.
- FFT magnitudes use coherent-gain correction and peak-amplitude dBFS. The
  logarithmic display is unweighted, with a labelled -150 dBFS display floor.
  It is not integrated spectral density, an analog noise floor or a THD test.
- A fixed 64-bin timing histogram adds p95/p99 callback execution upper bounds to
  the UI and reports. One relaxed atomic increment records each callback;
  percentile calculations run outside the audio callback. Empty/overflow evidence
  stays unavailable. Histogram timing describes AID work, not driver xruns.

Existing VST3 component and automation IDs, passive defaults and generator arming
rules are preserved. Spectrum/scope do not implement spectrogram, phase,
frequency-response, native clock or crosstalk acquisition plots.

- A passive Linux ALSA PCM adapter reads bounded kernel `hw_params`/`status`
  snapshots without opening streams or collecting owner process IDs. The `alsa`
  CLI exports JSON; the Devices scan displays the same reported format/state
  evidence separately from advertised CPAL capabilities. Closed, absent, malformed
  or oversized evidence is unavailable; reported XRUN/disconnect/suspension is
  a warning. This is not JACK/PipeWire routing or a globally stable endpoint ID.
