# Release readiness plan

Continue from PR #12. Preserve the specification's release acceptance criteria
and track incomplete advanced diagnostics honestly.

1. Generator lifecycle safety: disarm on initialize/reset/deactivate/session clear;
   cancel pending latency work; verify lifecycle regression tests.
2. Recording compensation: calculate residual recording offset from repeated RTL
   and entered host compensation, expose results with units and validity gates.
3. Passive host-path burn-in: timed monitoring, callback progress, telemetry error
   deltas, interruption detection, UI and exported findings; no automatic output.
4. Release audit: test the workspace, lint, formatting, repeat stress scenarios,
   validate packaging and publish changes for cross-platform CI.

Native platform telemetry, controlled device-opening suites, physical bench
calibration and desktop/hardware qualification must not be marked complete from
synthetic tests. Releasability is governed by docs/specs/00-product-spec.md;
the full advanced-feature roadmap is larger than the current beta release.

Tests precede release claims. Each slice has its own commit. Keep changes on the
existing PR branch and preserve local/remote tree equality when publishing.
