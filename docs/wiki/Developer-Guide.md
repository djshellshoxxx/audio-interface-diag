# Developer Guide

## Build and test

```bash
cargo test -p audio-interface-diag     # root crate: 37 integration tests pass (checked on this snapshot)
cargo test --workspace                 # includes the plugin crate
cargo run -- live                      # CLI planner
cargo xtask bundle aid_plugin --release   # CLAP, VST3 and standalone in target/bundled
```

On Linux, `cargo test --workspace` needs the ALSA and other development packages listed in the README
(`libasound2-dev` and the rest). Without them the `alsa-sys` build fails. The workspace test was not
completed in this pass for that reason; the root-crate tests were run.

Packaging: `packaging/linux/package.sh <tag>`; Windows installer script: `packaging/windows/installer.iss`.
Release: pushing a `v*` tag runs `.github/workflows/release.yml`.

## Notes

- `Cargo.toml` declares `license = "LicenseRef-Proprietary-TBD"`, which is a placeholder. Decide the licence before release.
- The `plugin/` crate uses nih-plug and egui; `src/` must stay free of audio-thread and GUI code.
