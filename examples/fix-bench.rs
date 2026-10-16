//! Comparable before/after benchmark. Run with --release --all-features.
//! Safe baseline paths only: deliberately excludes the broken 1.0.0 Ssx2 store.
use std::{hint::black_box, time::Instant};
use urng::{Rng, wide::RngW};

const N: usize = 2_000_000;
const RUNS: usize = 9;

fn measure(name: &str, mut run: impl FnMut()) {
    run();
    let mut ns = Vec::new();
    for _ in 0..RUNS {
        let t = Instant::now();
        run();
        ns.push(t.elapsed().as_nanos() as f64 / N as f64);
    }
    ns.sort_by(f64::total_cmp);
    println!("{name},{:.6}", ns[RUNS / 2]);
}

fn main() {
    println!("# N={N}, runs={RUNS}, median ns/output, seed=123");
    macro_rules! scalar {
        ($($ty:ident),+ $(,)?) => {$( {
            let mut rng = urng::$ty::new(black_box(123));
            measure(stringify!($ty), || {
                for _ in 0..N { black_box(rng.nextu()); }
                black_box(&mut rng);
            });
        } )+};
    }
    scalar!(
        SplitMix64,
        Xoshiro256Pp,
        Sfc32,
        Sfc64,
        Philox32,
        Pcg32,
        ChaCha20,
        Salsa20
    );
    let mut rng = urng::Sfc32::new(123);
    measure("Sfc32_randi_0_1e9", || {
        for _ in 0..N {
            black_box(rng.randi(0, 1_000_000_000));
        }
    });
    let mut rng = urng::wide::SplitMix32x16::new(123);
    measure("wide_SplitMix32x16", || {
        for _ in 0..N / 16 {
            black_box(rng.nextu());
        }
        black_box(&mut rng);
    });
    let mut out = vec![0u64; N];
    // Both versions have the same C ABI. The fixed version requires unsafe.
    #[allow(unused_unsafe)]
    unsafe {
        let rng = urng::cabi::splitmix64_new(123);
        measure("cabi_SplitMix64_16MB", || {
            urng::cabi::splitmix64_next_u64s(rng, out.as_mut_ptr(), out.len());
            black_box(&out);
        });
        urng::cabi::splitmix64_free(rng);
    }
}
