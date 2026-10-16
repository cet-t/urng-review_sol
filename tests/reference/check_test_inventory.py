"""Check that no pre-existing unit-test name disappeared (never run baseline UB).

Requires the isolated 1.0.0 snapshot described in benchmarks/README.md.
`--list` compiles/enumerates baseline tests but does not execute generators.
"""
from pathlib import Path
import subprocess

for selected in [[], ["--all-features"]]:
    inventories = []
    for cwd in [Path("target/fix-baseline"), Path(".")]:
        result = subprocess.run(["cargo", "test", "--locked", "--offline", *selected,
                                 "--lib", "--", "--list"], cwd=cwd,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
        inventories.append({line[:-6] for line in result.stdout.decode().splitlines()
                            if line.endswith(": test")})
    original, fixed = inventories
    missing = original - fixed
    assert not missing, sorted(missing)
    print(f"{','.join(selected) or 'default'}: original {len(original)}, "
          f"fixed {len(fixed)}, missing 0")
