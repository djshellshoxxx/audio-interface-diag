#!/usr/bin/env python3
"""Run the MIT SDK validator and legacy-state/automation compatibility fixture."""
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def run(*args):
    subprocess.run(args, check=True)


def main():
    dependencies = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "target/vst3-dependencies"
    build = ROOT / "target/vst3-validation"
    run("cmake", "-S", str(ROOT / "packaging/vst3-validation"), "-B", str(build),
        "-DCMAKE_BUILD_TYPE=Release", f"-DVST3_SDK_ROOT={dependencies / 'vst3sdk'}")
    run("cmake", "--build", str(build), "--config", "Release", "--target", "aid_compatibility", "validator", "--parallel", "2")
    suffix = ".exe" if sys.platform == "win32" else ""
    validator = build / f"bin/Release/validator{suffix}"
    compatibility = build / ("Release" if sys.platform == "win32" else "") / f"aid_compatibility{suffix}"
    bundle = ROOT / "target/bundled/Audio Interface Diag.vst3"
    run(str(validator), str(bundle))
    run(sys.executable, str(ROOT / "packaging/test_vst3_entry.py"), str(bundle))
    run(str(compatibility), str(bundle), str(ROOT / "tests/fixtures/legacy-vst3-0.0.2.json"), "load")


if __name__ == "__main__":
    main()
