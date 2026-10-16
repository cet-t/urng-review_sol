"""Check every distinct combination, including implicit `thiserror`.

Run from the repository root with Python 3. `full` is an alias for the six
functional features and is checked separately. Logs survive interruptions.
"""
import itertools
import json
from pathlib import Path
import subprocess

features = ["rand", "cabi", "wide", "simd", "sampler", "seedgen", "thiserror"]
log = Path("target/fix-validation")
log.mkdir(parents=True, exist_ok=True)
results = []
configs = [list(itertools.compress(features, bits))
           for bits in itertools.product([False, True], repeat=len(features))]
configs.append(["full"])
for index, selected in enumerate(configs, 1):
    cmd = ["cargo", "check", "--locked", "--offline", "--all-targets", "--no-default-features"]
    if selected:
        cmd += ["--features", ",".join(selected)]
    result = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    (log / f"features-{index:03}.log").write_bytes(result.stdout)
    results.append({"features": selected, "exit_code": result.returncode})
    (log / "features.json").write_text(json.dumps(results, indent=2), encoding="utf-8")
    print(f"{index}/{len(configs)} {','.join(selected) or '(none)'}: "
          f"{'PASS' if result.returncode == 0 else 'FAIL'}", flush=True)
    if result.returncode:
        print(result.stdout.decode("utf-8", errors="replace"), flush=True)
        raise SystemExit(result.returncode)
print("Every feature configuration passed.")
