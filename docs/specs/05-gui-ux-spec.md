# GUI / UX Specification

## Visual family
Use the same technical Circuit Drift Labs family as DeckChek/MIDI Test: dark neutral canvas, restrained accent, high-contrast data, compact panels, minimal decorative animation. Diagnostic state uses text/icon/shape as well as color.

## Navigation
Left rail:
- Overview
- Live
- DAW
- Engineer
- Devices
- Sessions
- Compare
- Reports
- Settings
- Help

Mode switching never starts a test.

## Overview
Device card, driver/API card, current format, current session state, recent findings and explicit action buttons.

## Live screen
Large stage-safe health status; meters; xrun/glitch counter; callback margin; event timeline; “mark event”; “capture telemetry snapshot”; layout density toggle.

## DAW screen
DAW process selector, session visibility explanation, process/system loopback availability, routing assistant, production timeline and compensation-test launcher.

## Engineer screen
Test catalog on left, setup/cabling instructions in center, live measurements/plots on right, result tray at bottom. Tests have states: not ready, ready, armed, running, complete, invalid, aborted.

## Device screen
Tree by host API -> physical/virtual device -> input/output endpoints. Show reported capability versus verified capability.

## Plotting
Scope, spectrum, spectrogram, latency correlation, frequency response, phase, timing histogram, callback timeline, drift and crosstalk matrix. Plot updates are decimated independently of the measurement engine.

## Accessibility
Keyboard navigation, scalable UI, minimum contrast, color-independent statuses, reduced motion, screen-reader labels for controls, numeric alternatives for plots.

## Tooltips/help
Every engineering term has a concise tooltip. Help includes: what the test measures, required wiring, what can invalidate it, typical symptoms and how to interpret the result.

## Destructive/intrusive actions
Arming an output-generating test shows selected device/channel, level and routing. Persistent STOP button is visible during active tests.
