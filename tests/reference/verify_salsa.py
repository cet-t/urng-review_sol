"""Reproduce libsodium's independent core3 known-answer hashes, offline.

Sources (key, nonce, counter order and SHA-256 expectations):
https://github.com/jedisct1/libsodium/blob/master/test/default/core3.c
https://github.com/jedisct1/libsodium/blob/master/test/default/core3.exp
Uses Python's standard hashlib rather than a second implementation in urng.
Run from the repository root: python tests/reference/verify_salsa.py
"""
import hashlib
import os
from pathlib import Path
import subprocess

subprocess.run(["cargo", "build", "--locked", "--offline", "--lib"], check=True)
target = Path(os.environ.get("CARGO_TARGET_DIR", "target"))
rlib = target / "debug" / "liburng.rlib"
if not rlib.exists():
    rlib = target / "debug" / "urng.rlib"
exe = target / ("reference-salsa.exe" if os.name == "nt" else "reference-salsa")
subprocess.run(["rustc", "--edition=2024", "tests/reference/salsa_stream.rs",
                "--extern", f"urng={rlib}", "-L", f"dependency={target / 'debug/deps'}",
                "-o", str(exe)], check=True)
expected = {
    "20": "662b9d0e3463029156069b12f918691a98f7dfb2ca0393c96bbfc6b1fbd630a2",
    "8": "017421baa9959cbe894bd003ec87938254f47c1e757eb66cf89c353d0c2b68de",
}
for rounds, digest in expected.items():
    stream = subprocess.check_output([str(exe.resolve()), rounds])
    assert len(stream) == 4 * 1024 * 1024
    actual = hashlib.sha256(stream).hexdigest()
    assert actual == digest, (rounds, actual, digest)
    print(f"Salsa{rounds}: {len(stream)} bytes, SHA256 {actual}: PASS")
