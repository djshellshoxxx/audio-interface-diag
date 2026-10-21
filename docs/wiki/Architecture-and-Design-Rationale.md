# Architecture and Design Rationale

Sources: README, `docs/specs/01-shared-engine-spec.md`, `docs/specs/07-architecture-testing-spec.md`, `docs/specs/10-requirements-traceability.md`.

- **One shared engine, three modes.** Live, DAW and Engineer modes share one measurement engine and one rule engine, so findings are consistent across modes.
- **Portable core vs. host wrapper.** `src/` (DSP, rules, planner, session state, reports) has no audio-thread or GUI code. `plugin/` is the nih-plug wrapper with the realtime engine, egui GUI and report export. *(README "Layout".)*
- **Never "healthy" without data.** Stage health is shown with text and colour, and it is not labelled healthy without measurements (README).
- **Passive by default.** Live and DAW modes never generate signal. Active tests are armed explicitly, and arming is not persisted, so a restart cannot resume generation.
- **Feedback safety.** In the standalone app, input is not routed to output unless the user enables it.
- **Confidence-gated latency.** Round-trip results report median, min, max and spread across 5 runs, and are gated by confidence (README).
- **Traceability.** Requirements are mapped to tests in `docs/specs/10-requirements-traceability.md`; audits are in `docs/audits/`.
