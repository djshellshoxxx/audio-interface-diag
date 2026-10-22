# Proprietary VST3 build and state migration

Owner direction: original application code is proprietary/all rights reserved.
The prior NIH VST3 path used GPLv3 `vst3-sys` bindings and cannot be retained in
that distribution. NIH's ISC CLAP/GUI/standalone implementations remain enabled
with `default-features = false`; Cargo.lock removes all four VST3 binding crates.

The replacement uses exact commits in `packaging/build_vst3.py`: MIT clap-wrapper,
CLAP SDK and Steinberg VST3 SDK 3.8.0. A private CLAP payload is loaded only from
this VST3 bundle's Resources. The entry proxy balances initialization and teardown
and never performs filesystem/loading work in audio processing. AAX/ASIO wrapper
features and automatic installation/download fallback are disabled.

State migration required an explicit adapter: despite sharing JSON serialization,
NIH CLAP prepends a little-endian u64 length while NIH VST3 stores raw JSON.
Direct legacy restore reproduced an enormous attempted allocation. The dedicated
`aid_state.h` bounds streams to 16 MiB, checks framing before calling NIH, adds the
CLAP length on input and strips it on output. The pinned MIT wrapper is adapted
in a generated translation unit; upstream dependency checkouts are unchanged.

Compatibility validation checks the exact legacy class ID, all eight parameter
IDs, non-default values from the actual legacy binary, raw/framed restore,
resave/restore and malformed/truncated/oversized rejection. The canonical old
component ID is `43444C417564696F4966616365446961`.

`packaging/validate_vst3.py` builds and runs Steinberg's native validator plus the
migration and entry-lifecycle probes. It is required in release CI for every
platform. The old NIH 0.0.2 binary fails the validator's sidechain arrangement
check; the replacement passed all 47 native validator tests in the Linux cloud.

`THIRD-PARTY-NOTICES.txt` collects actual package license/copyright texts,
including embedded-font licenses. Packages that omit license files use their
pinned upstream texts; old block/malloc_buf tags are pinned explicitly. The
notice inventory follows the native application dependency graph, excluding
unshipped xtask/dev-only tooling. Dynamic operating-system audio/display
libraries remain supplied by the user's system under their own licenses.

Build provenance records MIT dependency pins, Cargo.lock digest and the exact
embedded CLAP digest. Packaging replaces the known generated VST3 directory
before staging; the old NIH-generated VST3 cannot be reused as a release asset.
