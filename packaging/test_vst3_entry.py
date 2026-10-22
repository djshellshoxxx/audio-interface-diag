#!/usr/bin/env python3
"""Exercise the private CLAP proxy's balanced initialization contract."""
import ctypes
from pathlib import Path
import sys


class Version(ctypes.Structure):
    _fields_ = [("major", ctypes.c_uint32), ("minor", ctypes.c_uint32), ("revision", ctypes.c_uint32)]


Init = ctypes.CFUNCTYPE(ctypes.c_bool, ctypes.c_char_p)
Deinit = ctypes.CFUNCTYPE(None)
Factory = ctypes.CFUNCTYPE(ctypes.c_void_p, ctypes.c_char_p)


class Entry(ctypes.Structure):
    _fields_ = [("version", Version), ("init", Init), ("deinit", Deinit), ("factory", Factory)]


def main():
    bundle = Path(sys.argv[1])
    paths = [p for p in (bundle / "Contents").rglob("Audio Interface Diag*")
             if p.is_file() and p.parent.name in ("MacOS", "x86_64-linux", "x86_64-win")]
    if len(paths) != 1:
        raise RuntimeError(f"Expected one wrapper binary: {paths}")
    binary = ctypes.CDLL(str(paths[0].resolve()))
    entry = Entry.in_dll(binary, "clap_entry")
    assert not entry.factory(b"clap.plugin-factory")
    assert entry.init(str(paths[0]).encode())
    assert entry.init(str(paths[0]).encode())
    assert entry.factory(b"clap.plugin-factory")
    entry.deinit()
    assert entry.factory(b"clap.plugin-factory"), "First deinit destroyed a still-referenced entry"
    entry.deinit()
    assert not entry.factory(b"clap.plugin-factory")
    entry.deinit()  # A defensive unmatched deinit must stay harmless.
    assert entry.init(str(paths[0]).encode())
    entry.deinit()
    print("Balanced CLAP proxy init/deinit and reinitialization verified")


if __name__ == "__main__":
    main()
