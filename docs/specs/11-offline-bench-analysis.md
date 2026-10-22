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
