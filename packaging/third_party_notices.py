#!/usr/bin/env python3
"""Collect retained license texts from the locked Cargo graph and pinned SDKs."""
import json
import shutil
from pathlib import Path
import subprocess
import sys
import re
import urllib.request
import urllib.error
from concurrent.futures import ThreadPoolExecutor

ROOT = Path(__file__).resolve().parents[1]


def license_files(directory):
    def named(path):
        return any(word in path.name.upper() for word in ("LICENSE", "LICENCE", "COPYING", "COPYRIGHT", "NOTICE", "OFL")) or ("fonts" in path.parts and path.suffix == ".txt")
    files = sorted(p for p in directory.rglob("*") if p.is_file() and named(p) and ".git" not in p.parts and p.stat().st_size < 1_000_000)
    if not files:
        # Git workspace packages such as nih_plug_egui share the repository license.
        files = sorted(p for p in directory.parent.iterdir() if p.is_file() and named(p))
    return files


def upstream_licenses(package):
    directory = Path(package["manifest_path"]).parent
    vcs_file = directory / ".cargo_vcs_info.json"
    vcs = json.loads(vcs_file.read_text()) if vcs_file.exists() else {}
    commit = {"block": "47178790cfc9d4a8b092051d8b413b78bd31254a", "malloc_buf": "a7811e5f4c6f9685dd57ad9bfe34e9bf6b0ba4f9"}.get(package["name"]) or vcs.get("git", {}).get("sha1") or package["source"].split("#")[-1]
    repository = package.get("repository") or ""
    match = re.match(r"https?://github[.]com/([^/]+/[^/#]+)", repository)
    if not match or not re.fullmatch(r"[a-f0-9]{40}", commit):
        raise RuntimeError(f"Cannot identify pinned upstream license for {package['name']}")
    slug = match[1].removesuffix(".git")
    cache = ROOT / "target/license-cache" / f"{package['name']}-{package['version']}"
    cache.mkdir(parents=True, exist_ok=True)
    if list(cache.iterdir()):
        return sorted(cache.iterdir())
    # Prefer the upstream root's complete grant, then its MIT/Apache alternatives.
    candidates = ["LICENSE", "LICENSE-MIT", "LICENSE-APACHE", "LICENSE.txt", "LICENSE.md", "LICENSE-MIT.txt", "LICENSE-APACHE.txt", "UNLICENSE"]
    for name in candidates:
        url = f"https://raw.githubusercontent.com/{slug}/{commit}/{name}"
        try:
            with urllib.request.urlopen(url, timeout=30) as response:
                data = response.read(1_000_001)
            if len(data) > 1_000_000:
                raise RuntimeError(f"Oversized upstream license: {url}")
            (cache / name).write_bytes(data)
        except urllib.error.HTTPError as error:
            if error.code != 404:
                raise
    if not list(cache.iterdir()):
        raise RuntimeError(f"Missing pinned upstream license: {slug}@{commit}")
    return sorted(cache.iterdir())


def main():
    rust_info = subprocess.check_output(["rustc", "-vV"], text=True)
    host = next(line.split(": ", 1)[1] for line in rust_info.splitlines() if line.startswith("host: "))
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--format-version", "1", "--filter-platform", host], cwd=ROOT))
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    relevant = {p["id"] for p in metadata["packages"] if p["name"] in ("aid_plugin", "audio-interface-diag")}
    pending = list(relevant)
    while pending:
        for dependency in nodes[pending.pop()]["deps"]:
            if dependency["pkg"] not in relevant and any(kind["kind"] != "dev" for kind in dependency["dep_kinds"]):
                relevant.add(dependency["pkg"])
                pending.append(dependency["pkg"])
    metadata["packages"] = [p for p in metadata["packages"] if p["id"] in relevant]
    packages = [p for p in metadata["packages"] if p["source"] is not None]
    missing = [p for p in packages if not license_files(Path(p["manifest_path"]).parent)]
    with ThreadPoolExecutor(max_workers=6) as pool:
        fetched = dict(zip((p["id"] for p in missing), pool.map(upstream_licenses, missing)))
    text = ["Audio Interface Diag — Third-party notices\n", "Owned application code: see LICENSE. Dependency licenses below remain in force.\n", "This inventory includes application dependencies and their build dependencies for this platform.\n"]
    for package in sorted(metadata["packages"], key=lambda p: (p["name"], p["version"])):
        if package["source"] is None:
            continue
        directory = Path(package["manifest_path"]).parent
        files = license_files(directory) or fetched.get(package["id"], [])
        text.append(f"\n{'=' * 72}\n{package['name']} {package['version']}\nLicense expression: {package.get('license')}\nSource: {package.get('repository') or package['source']}\n")
        if not files:
            raise RuntimeError(f"Missing retained license text: {package['name']}")
        for path in files:
            text.append(f"\n--- {path.name} ---\n{path.read_text(encoding='utf-8', errors='replace')}\n")
    dependencies = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "target/vst3-dependencies"
    for name in ["clap-wrapper", "clap", "vst3sdk"]:
        directory = dependencies / name
        if not directory.exists():
            raise RuntimeError(f"Missing pinned SDK {directory}; run build_vst3.py first")
        text.append(f"\n{'=' * 72}\n{name} (pinned VST3 build dependency)\n")
        for path in license_files(directory):
            text.append(f"\n--- {path.relative_to(directory)} ---\n{path.read_text(encoding='utf-8', errors='replace')}\n")
    output = ROOT / "target/bundled/THIRD-PARTY-NOTICES.txt"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text("".join(text), encoding="utf-8")
    resources = ROOT / "target/bundled/Audio Interface Diag.vst3/Contents/Resources"
    resources.mkdir(parents=True, exist_ok=True)
    shutil.copy2(output, resources / output.name)
    shutil.copy2(ROOT / "LICENSE", resources / "LICENSE")
    if sys.platform == "darwin":
        subprocess.run(["codesign", "--force", "--sign", "-", str(resources.parent.parent)], check=True)
    print(output)


if __name__ == "__main__":
    main()
