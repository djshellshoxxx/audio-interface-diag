#!/usr/bin/env bash
# Builds the Linux portable tarball and .deb installer from target/bundled.
# Usage: packaging/linux/package.sh <version>   (run after `cargo xtask bundle aid_plugin --release`)
set -euo pipefail
VERSION="${1:?version required}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
B="$ROOT/target/bundled"
OUT="$ROOT/dist"
NAME="Audio Interface Diag"
rm -rf "$OUT" && mkdir -p "$OUT"

# Portable: everything in one folder plus a per-user install helper.
P="$OUT/AudioInterfaceDiag-$VERSION-linux-x86_64-portable"
mkdir -p "$P"
cp "$B/$NAME" "$P/audio-interface-diag"
cp "$B/$NAME.clap" "$P/"
cp -r "$B/$NAME.vst3" "$P/"
cp "$ROOT/packaging/linux/install-user.sh" "$P/"
cp "$ROOT/README.md" "$P/README.md"
chmod +x "$P/audio-interface-diag" "$P/install-user.sh"
tar -C "$OUT" -czf "$P.tar.gz" "$(basename "$P")"

# Installer: system-wide .deb.
D="$OUT/deb"
mkdir -p "$D/DEBIAN" "$D/usr/bin" "$D/usr/lib/clap" "$D/usr/lib/vst3" "$D/usr/share/applications"
cp "$B/$NAME" "$D/usr/bin/audio-interface-diag"
cp "$B/$NAME.clap" "$D/usr/lib/clap/"
cp -r "$B/$NAME.vst3" "$D/usr/lib/vst3/"
cat > "$D/usr/share/applications/audio-interface-diag.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Audio Interface Diag
Comment=Audio interface diagnostics (Live / DAW / Engineer)
Exec=audio-interface-diag
Terminal=false
Categories=AudioVideo;Audio;
DESKTOP
cat > "$D/DEBIAN/control" <<CONTROL
Package: audio-interface-diag
Version: $(echo "${VERSION#v}" | tr - "~")
Section: sound
Priority: optional
Architecture: amd64
Depends: libasound2 | libasound2t64, libgl1, libx11-xcb1, libxcursor1
Recommends: libjack-jackd2-0 | libjack0
Maintainer: Circuit Drift Labs <noreply@example.com>
Description: Audio Interface Diag - CLAP, VST3 and standalone audio interface diagnostics
 Passive Live/DAW monitoring and armed Engineer loopback measurements.
CONTROL
chmod -R u+rwX,go+rX "$D"
dpkg-deb --root-owner-group --build "$D" "$OUT/audio-interface-diag_${VERSION#v}_amd64.deb"
rm -rf "$D" "$P"
ls -la "$OUT"
