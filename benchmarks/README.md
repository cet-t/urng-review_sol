# Correctness-fix benchmark

The CSV files contain an actual before/after run on the same Windows x86_64
machine (2026-10-16). `after/before` is elapsed time per output, so a larger ratio
means slower generation. These measurements compare the original algorithms with
their corrected implementations; they do not claim identical output sequences.

- Baseline: urng 1.0.0, `9f8518e05b4c7580d8a291ba679e900fb8dba0af`.
- Compiler: rustc 1.98.1 (`48a229cea`), LLVM 22.1.8, `x86_64-pc-windows-msvc`.
- CPU identifier: `AMD64 Family 26 Model 68 Stepping 0, AuthenticAMD`;
  16 logical processors. Both programs ran on this machine.
- Identical `examples/fix-bench.rs`, checked byte-for-byte; SHA-256 is saved in
  `source-sha256.txt`. Dependencies and lockfile are unchanged.
- `--release --all-features`, opt-level 3, fat LTO, codegen-units 1, panic abort.
  `RUSTFLAGS`, `RAYON_NUM_THREADS` unset; no extra target CPU/features requested.
- Seed 123; 2,000,000 outputs per sample; one warmup, nine samples, median
  nanoseconds/output. Wide output counts every lane. C ABI writes 16,000,000
  bytes per sample. Black-box observations prevent output elimination.
- Executed the baseline first, then the fixed program, with no concurrent builds
  or test jobs. CPU affinity and frequency were not pinned. Small differences
  should be treated as measurement/code-layout noise. No confidence intervals
  were calculated. The original unsafe `Xoshiro256Ssx2` path was never run.

Reproduce on Windows/PowerShell from the repository root, using a fresh
`target/fix-baseline` directory:

```powershell
git archive --format=zip 9f8518e05b4c7580d8a291ba679e900fb8dba0af --output=target/fix-baseline.zip
Expand-Archive -LiteralPath target/fix-baseline.zip -DestinationPath target/fix-baseline
Copy-Item -LiteralPath examples/fix-bench.rs -Destination target/fix-baseline/examples/fix-bench.rs
Push-Location target/fix-baseline
cargo build --locked --offline --release --all-features --example fix-bench
Pop-Location
cargo build --locked --offline --release --all-features --example fix-bench
python tests/reference/compare_bench.py
```

Clear task-specific compiler/thread flags before reproducing these conditions.
The isolated baseline only receives the identical benchmark example; its library
sources, Cargo profile, and lockfile remain from the original commit.

The main costs are intentional: sequential C filling replaces parallel reseeding
and NT kernels; uniform integer draws use rejection; SplitMix wide lanes now use
the verified scalar finalizer and disjoint counter blocks. Cipher timings also
reflect fixing the number of rounds, state layout, and feed-forward. See
`../FIX_REPORT.md` for the measured table and limitations.
