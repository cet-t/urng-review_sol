"""Run the same already-built benchmark before and after, serially.

See benchmarks/README.md for building the isolated 1.0.0 baseline. Do not run
other builds or benchmarks concurrently. Times include correctness fixes.
"""
import csv
import hashlib
from pathlib import Path
import subprocess

root = Path.cwd()
baseline = root / "target/fix-baseline"
source = (root / "examples/fix-bench.rs").read_bytes()
assert source == (baseline / "examples/fix-bench.rs").read_bytes(), "benchmark source differs"
dest = root / "benchmarks"
dest.mkdir(exist_ok=True)
for label, directory in [("before", baseline), ("after", root)]:
    exe = directory / "target/release/examples/fix-bench.exe"
    result = subprocess.check_output([str(exe)], cwd=directory).decode()
    (dest / f"fix-{label}.csv").write_text(result.replace("\r\n", "\n"), encoding="utf-8")
    print(f"{label}:\n{result}", flush=True)

def read(label):
    rows = (dest / f"fix-{label}.csv").read_text().splitlines()
    return {name: float(value) for name, value in csv.reader(row for row in rows if not row.startswith("#"))}

before, after = read("before"), read("after")
assert before.keys() == after.keys()
print("name,before_ns,after_ns,after/before")
for name in before:
    print(f"{name},{before[name]:.6f},{after[name]:.6f},{after[name]/before[name]:.3f}")
(dest / "source-sha256.txt").write_text(hashlib.sha256(source).hexdigest() + "\n", encoding="ascii")
