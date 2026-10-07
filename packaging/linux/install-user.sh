#!/usr/bin/env bash
# Installs the portable build for the current user only (no root needed).
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$HOME/.clap" "$HOME/.vst3" "$HOME/.local/bin"
cp "$HERE/Audio Interface Diag.clap" "$HOME/.clap/"
rm -rf "$HOME/.vst3/Audio Interface Diag.vst3"
cp -r "$HERE/Audio Interface Diag.vst3" "$HOME/.vst3/"
cp "$HERE/audio-interface-diag" "$HOME/.local/bin/"
echo "Installed CLAP to ~/.clap, VST3 to ~/.vst3, standalone to ~/.local/bin/audio-interface-diag"
