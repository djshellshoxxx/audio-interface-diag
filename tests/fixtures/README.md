# Legacy VST3 state fixture

`legacy-vst3-0.0.2.json` was saved from the actual 0.0.2 NIH VST3 binary at commit
586e313, after setting Mode normalized 0.5, Frequency 0.75 and Level 0.2. It is
synthetic parameter/editor state and contains no audio, credentials or device
identity. `aid_compatibility` verifies the new wrapper restores these values,
preserves all automation IDs and writes the same raw-JSON framing.
