# Live Edition Specification

## Goal
Run during a performance with negligible interference and surface only information that helps a performer or technician prevent/understand an audible failure.

## Default behavior
Live mode is passive. It must not seize an exclusive device, renegotiate the active stream, emit tones, alter routing or probe unsupported formats while a performance is underway.

## Dashboard
At-a-glance top row: Device, API, sample rate, buffer/period if observable, session duration and overall health.
Primary meters:
- input/output levels when accessible
- headroom and clipping
- xruns/discontinuities
- callback/period margin
- device/driver state
- clock/sample-rate consistency
Timeline: glitches, reconnects, format changes, device changes, warning transitions.
A compact “stage view” uses large typography readable at distance.

## Live health score
Do not hide raw counts behind a score. Score is secondary and decomposes into transport, timing, clipping, device state and clock stability.

## Alerts
Default alerts:
- device disconnected/reconnected
- active endpoint changed
- sample rate changed
- repeated clipping
- xrun/overflow/underflow
- discontinuity
- callback margin below threshold
- CPU/system pressure correlated with failures where observable
Alerts are rate-limited and may be visual-only to avoid injecting audio into the performance chain.

## Snapshot
One click captures the previous and next bounded interval of telemetry around a fault marker. Raw audio is not captured unless separately enabled before the show.

## Pre-show check
A non-performance workflow may validate device visibility, channels, chosen sample rate, safe buffer, clock source metadata and a short silent stability run. Any signal-emitting loopback test is clearly separated.

## Comparison
Compare tonight’s current profile against a known-good venue/profile: driver version, port/device, sample rate, buffer, endpoint IDs and recent stability.

## Performance requirements
Monitoring overhead target: <1% typical CPU on a modern desktop for basic telemetry, bounded memory, no unbounded logs, no blocking from UI rendering, and automatic degradation of expensive visualizations before core monitoring is impacted.

## Failure behavior
If AID itself falls behind, it marks its telemetry gap and disables optional visualization work. It must not continue presenting stale data as current.
