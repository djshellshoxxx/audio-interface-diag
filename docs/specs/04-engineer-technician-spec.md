# Engineer / Technician Edition Specification

## Scope
Includes every Live and DAW feature plus active qualification and bench workflows.

## Test suites
### Device capability truth test
Enumerate advertised formats and attempt controlled opens while no production session is active. Matrix records advertised, open-success, stable-duration, observed rate, actual/report latency and errors.

### Buffer stability sweep
Test descending buffers/periods with a deterministic workload. At each point collect xruns, discontinuities, callback p95/p99/max, CPU load and successful duration. Recommend the smallest buffer meeting configured stability policy.

### Round-trip latency
Physical loopback recommended. Repeat deterministic correlation measurement. Report median, min/max/spread and driver-reported versus measured discrepancy.

### Channel map
Sequential unique stimulus identifies input/output path mappings, duplicates, dead paths and swapped pairs.

### Gain/channel tracking
With a stable stimulus, measure channel level mismatch through selected gain positions. Manual checkpoints allow technician to turn hardware controls.

### Polarity and phase
Detect full-band inversion and relative delay; sweep-based phase response is available for bench work.

### Noise/hum
Capture terminated/appropriate input states selected by technician. Analyze broadband level and 50/60 Hz families. Report configuration and gain so results are comparable.

### Frequency response
Log sweep, selectable band and level, result magnitude curve, flatness summary and per-channel comparison.

### Distortion
THD and THD+N with validity gates for clipping, insufficient level, inadequate duration/resolution and unstable reference.

### Crosstalk
Channel-by-channel matrix at configured frequencies/levels.

### Clock/drift
Long-duration effective-rate measurement, ppm and drift projection. Dual-device mode compares independently clocked devices.

### Burn-in
Configurable 10 min/1 h/8 h/custom. Tracks all timing errors, resets, disconnects, drift and signal-integrity failures. Can resume report generation after app restart but must not auto-resume output.

### Transport/port comparison
Repeat qualification with named environments (rear USB, front USB, hub, dock, battery, AC) and compare.

## Technician helpers
- cable loopback diagram based on selected channels
- test-level calculator
- expected routing checklist
- phantom-power workflow that instructs multimeter measurement but does not infer voltage without entered/measured data
- direct-monitoring comparison workflow
- headphone L/R and channel verification
- digital I/O lock/clock checklist for S/PDIF/ADAT where exposed
- MIDI handoff link to MIDI Test for interfaces with MIDI ports

## Reports
Bench report sections: DUT identity, host, driver, cabling, calibration notes, conditions, raw configuration, results, pass/warn/fail, plots/data references, technician notes and signature/date fields.

## Calibration
User may store loopback cable attenuation, external attenuator values and known reference-interface profiles. Uncalibrated amplitude-sensitive results are labeled relative rather than absolute.
