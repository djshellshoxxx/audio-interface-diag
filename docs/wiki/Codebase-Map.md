# Codebase Map: where to find what

| I want to... | Look in |
|---|---|
| Shared measurement and analysis | `src/analysis.rs`, `src/signal.rs` |
| Data model and reports | `src/model.rs`, `src/report.rs` |
| Test planner (engineer catalogue) | `src/planner.rs` |
| Session state | `src/session.rs` |
| Native audio backend (optional `native-audio` feature, cpal) | `src/native.rs` |
| CLI entry point | `src/main.rs` |
| Plugin (CLAP/VST3/standalone wrapper, GUI, report export) | `plugin/src/` (`lib.rs`, `main.rs`, `editor.rs`, `report.rs`, `shared.rs`) |
| Bundler | `xtask/src/main.rs` |
| Installers and packaging | `packaging/` |
| Tests | `tests/` (core, feature, session/signal, audit regressions) |
| Specs and audits | `docs/specs/`, `docs/audits/` |
| CI and release | `.github/workflows/` |
