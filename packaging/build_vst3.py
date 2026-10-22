#!/usr/bin/env python3
"""Build the proprietary-compatible VST3 facade from pinned MIT dependencies."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
PINS = {
    "clap-wrapper": ("free-audio/clap-wrapper", "806bba0a5f792e3966b99c1561ab9952d3316d7c"),
    "clap": ("free-audio/clap", "195b42a004144fab0b3cf95e9c067187d15365b7"),
    "vst3sdk": ("steinbergmedia/vst3sdk", "9fad9770f2ae8542ab1a548a68c1ad1ac690abe0"),
}


def run(*args, **kwargs):
    subprocess.run(args, check=True, **kwargs)


def dependency(base, name, repository, commit):
    path = base / name
    if not path.exists():
        run("git", "init", str(path))
        run("git", "-C", str(path), "remote", "add", "origin", f"https://github.com/{repository}.git")
        run("git", "-C", str(path), "fetch", "--depth", "1", "origin", commit)
        run("git", "-C", str(path), "checkout", "--detach", "FETCH_HEAD")
    actual = subprocess.check_output(["git", "-C", str(path), "rev-parse", "HEAD"], text=True).strip()
    if actual != commit:
        raise RuntimeError(f"{path}: expected pinned {commit}, found {actual}; use a fresh dependency directory")
    dirty = subprocess.check_output(["git", "-C", str(path), "status", "--porcelain", "--untracked-files=all"], text=True).strip()
    if dirty:
        raise RuntimeError(f"Pinned dependency has local modifications: {path}; use a fresh dependency directory")
    if name == "vst3sdk":
        run("git", "-C", str(path), "submodule", "update", "--init", "--depth", "1", "base", "public.sdk", "pluginterfaces", "cmake")
    if subprocess.check_output(["git", "-C", str(path), "status", "--porcelain", "--untracked-files=all"], text=True).strip():
        raise RuntimeError(f"Dependency/submodule checkout is not pristine: {path}")
    return path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jobs", default="2")
    parser.add_argument("--dependencies", type=Path, default=ROOT / "target/vst3-dependencies")
    args = parser.parse_args()
    paths = {name: dependency(args.dependencies, name, *pin) for name, pin in PINS.items()}
    build = ROOT / "target/vst3-wrapper"
    run("cmake", "-S", str(ROOT / "packaging/vst3"), "-B", str(build),
        "-DCMAKE_BUILD_TYPE=Release", f"-DAID_WRAPPER_ROOT={paths['clap-wrapper']}",
        f"-DCLAP_SDK_ROOT={paths['clap']}", f"-DVST3_SDK_ROOT={paths['vst3sdk']}")
    run("cmake", "--build", str(build), "--config", "Release", "--target", "aid_vst3", "--parallel", args.jobs)
    candidates = [p for p in build.rglob("Audio Interface Diag.vst3") if p.is_dir()]
    if len(candidates) != 1:
        raise RuntimeError(f"Expected exactly one VST3 bundle, found {candidates}")
    source = candidates[0]
    clap = ROOT / "target/bundled/Audio Interface Diag.clap"
    if not clap.exists():
        raise RuntimeError("Build the CLAP first: cargo xtask bundle aid_plugin --release")
    resources = source / "Contents/Resources"
    resources.mkdir(parents=True, exist_ok=True)
    destination = resources / clap.name
    if destination.is_dir():
        shutil.rmtree(destination)
    elif destination.exists():
        destination.unlink()
    if clap.is_dir():
        shutil.copytree(clap, destination)
    else:
        shutil.copy2(clap, destination)
    def digest(path):
        result = hashlib.sha256()
        files = sorted(path.rglob("*")) if path.is_dir() else [path]
        for file in files:
            if file.is_file():
                if path.is_dir():
                    result.update(str(file.relative_to(path)).encode("utf-8"))
                result.update(file.read_bytes())
        return result.hexdigest()
    if digest(clap) != digest(destination):
        raise RuntimeError("Embedded CLAP does not match the current candidate")
    provenance = {"format": "aid-mit-vst3-v1", "sdk_pins": PINS,
                  "component_id": "43444C417564696F4966616365446961",
                  "clap_sha256": digest(destination),
                  "cargo_lock_sha256": hashlib.sha256((ROOT / "Cargo.lock").read_bytes()).hexdigest()}
    (resources / "BUILD-PROVENANCE.json").write_text(json.dumps(provenance, indent=2) + "\n")
    final = ROOT / "target/bundled" / source.name
    if final.is_dir():
        shutil.rmtree(final)
    shutil.copytree(source, final)
    if sys.platform == "darwin":
        run("codesign", "--force", "--sign", "-", str(final))
    print(f"Created self-contained MIT-wrapper VST3: {final}")


if __name__ == "__main__":
    main()
