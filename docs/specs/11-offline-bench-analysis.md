# Offline bench capture analysis

This workflow measures user-supplied evidence without opening an audio device or
generating output. It supplements the plugin's live measurements.

## Distortion

Source command: `audio-interface-diag tone <sample-rate> <frequency> <mono.f32le>`.
Downloaded packages name the CLI executable `aid-analyze` (`aid-analyze.exe` on Windows).
Input is headerless, mono IEEE float32 little-endian, at most 32 MiB. Export a
steady section from a recorded hardware loopback. WAV headers and interleaved
stereo must be removed by the recording/export application first.

The capture must contain at least ten whole cycles of the specified fundamental,
with at least 256 samples. Frequency must leave room for a second harmonic below
Nyquist. Captures containing clipping, NaN/Inf, a fundamental below -80 dBFS RMS,
or a non-dominant fundamental are unavailable. Integer-cycle validation uses the
declared frequency/rate; a mismatched physical clock or unstable oscillator can
still cause spectral leakage. Use a coherent source and capture clock.

The analyzer projects sine/cosine components, excludes DC, and reports RMS
fundamental level, THD over harmonics 2 through 10 below Nyquist, and full-band
THD+N from the residual after subtracting the fundamental. These are relative,
uncalibrated measurements without AES17 weighting/band filtering. They are not
an absolute device certification or an automatic pass/fail judgement.

Output is the shared JSON findings schema, with configuration and units in the
evidence. Exit codes: 0 measured, 1 unavailable measurement, 2 input/usage error.
Raw audio is read locally and never retained or uploaded by this command.

## Verification

Known second harmonic, DC rejection, level and polarity scaling, 44.1/48/96/192
kHz, different record lengths, malformed input, missing files, silence, weak or
wrong-frequency stimulus, clipping, non-finite values and non-coherence are
covered by automated tests. Physical calibration and native acquisition remain
separate qualification work.

## Noise and mains-family evidence

`aid-analyze noise <sample-rate> <mono.f32le>` accepts the same bounded mono
capture format. Supply at least one second with a rate above 360 Hz (the third
60 Hz harmonic must remain below Nyquist). The analyzer rejects clipped,
non-finite, short and zero-AC captures; digital silence is unavailable evidence.
It reports DC separately, DC-removed full-band unweighted RMS dBFS, and the
stronger 50/60 Hz family through its third harmonic relative to AC power.
This narrow-bin observation does not diagnose grounding or measure an absolute
analog noise floor. Document gain, input termination, routing and bandwidth
separately; compare only matched conditions. No output signal or native capture
is started. Exit codes match the distortion command. Synthetic 50/60 Hz evidence
at 44.1/48/96/192 kHz, DC rejection and invalid captures are automated tests.

## Pairwise crosstalk

`aid-analyze crosstalk <sample-rate> <frequency> <driven.f32le> <victim.f32le>`
compares the stimulus-frequency RMS in two simultaneous equal-length captures.
The driven channel must pass the coherent-tone validity gates above; both files
must be finite and unclipped. Victim DC is removed. The result is the victim tone
level minus the driven fundamental in dB. An absent victim tone is unavailable,
not infinite isolation. This measures one route/frequency, without automatic
channel selection, calibrated analog units or a quality judgement. External
coincident tones/noise can contaminate the result; report routing, gain and
termination and repeat all required channel pairs. Native acquisition and a full
matrix remain separate work.

## Headless native inventory

`aid-analyze devices` exports advertised device capabilities through the default
CPAL host. Source builds need `--features native-audio`; packaged analyzers enable
that feature. No stream is opened. Unknown buffer ranges and per-direction query
errors stay explicit; an absent backend or an empty device list is unavailable.
Endpoint identity, actual successful opens and advanced OS telemetry cannot be
inferred from names or advertised ranges.
