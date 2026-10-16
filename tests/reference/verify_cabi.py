"""Build/run the C guard-page test against the release DLL on Windows.

Requires GCC (MinGW/Strawberry) and the DLL from
`cargo build --locked --offline --release --all-features --lib`.
The input manifest is derived from public declarations, so missing exports fail.
"""
from pathlib import Path
import re
import subprocess

target = Path("target/fix-validation").resolve()
target.mkdir(parents=True, exist_ok=True)
rows = []
for p in sorted(Path("src/cabi").rglob("*.rs")):
    source = p.read_text(encoding="utf-8")
    for name, seed_bits in re.findall(r'fn (\w+)_new\(seed: u(32|64)\)', source):
        word_bits = re.search(rf'fn {name}_next_u(32|64)s\(', source).group(1)
        rows.append(f'{{"{name}", {seed_bits}, {word_bits}}},\n')
assert len(rows) == 56, len(rows)
(target / "cabi_generators.inc").write_text("".join(rows), encoding="ascii")
exe = target / "cabi-guard.exe"
subprocess.run(["gcc", "-std=c11", "-O2", "-Wall", "-Wextra", "-Wno-cast-function-type",
                "-I", str(target), "tests/reference/cabi_guard.c", "-o", str(exe)], check=True)
subprocess.run([str(exe), str(Path("target/release/urng.dll").resolve())], check=True)
