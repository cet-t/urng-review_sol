#![cfg(feature = "cabi")]
use urng::{Rng, cabi::*};

macro_rules! scalar_streams {
    ($bits:tt; $(($ty:ident, $prefix:ident)),+ $(,)?) => {$(pastey::paste! {
        #[test]
        fn [<$prefix _matches_scalar_at_call_and_chunk_boundaries>]() {
            let _: unsafe extern "C" fn(*mut urng::$ty, *mut [<u $bits>], usize)
                = [<$prefix _next_u $bits s>];
            for seed in [0, 1] {
                let mut scalar = urng::$ty::new(seed);
                let p = [<$prefix _new>](seed as _);
                assert!(!p.is_null());
                for n in [0,1,7,8,15,16,17,131071,131072,131073] {
                    let mut buffer = vec![[<u $bits>]::MAX; n + 2];
                    unsafe { [<$prefix _next_u $bits s>](p, buffer.as_mut_ptr().add(1), n); }
                    assert_eq!(buffer[0], [<u $bits>]::MAX);
                    assert_eq!(buffer[n+1], [<u $bits>]::MAX);
                    for &actual in &buffer[1..n+1] { assert_eq!(actual, scalar.nextu()); }
                }
                let mut ints=[0;35];
                unsafe { [<$prefix _rand_i $bits s>](p,ints.as_mut_ptr(),ints.len(),[<i $bits>]::MIN,[<i $bits>]::MAX); }
                for actual in ints {assert_eq!(actual,scalar.randi([<i $bits>]::MIN,[<i $bits>]::MAX));}
                let mut floats=[0.0;35];
                unsafe { [<$prefix _next_f $bits s>](p,floats.as_mut_ptr(),floats.len()); }
                for actual in floats {assert_eq!(actual,scalar.nextf());}
                let min=1.0 as [<f $bits>];let max=min.next_up();
                unsafe { [<$prefix _rand_f $bits s>](p,floats.as_mut_ptr(),floats.len(),min,max); }
                for actual in floats {assert_eq!(actual,scalar.randf(min,max));}
                // Invalid numeric ranges are rejected before touching state/output.
                unsafe {
                    [<$prefix _rand_i $bits s>](p,ints.as_mut_ptr(),ints.len(),1,0);
                    [<$prefix _rand_f $bits s>](p,floats.as_mut_ptr(),floats.len(),[<f $bits>]::NAN,max);
                }
                let mut last=0;
                unsafe { [<$prefix _next_u $bits s>](p,&mut last,1); }
                assert_eq!(last,scalar.nextu());
                unsafe { [<$prefix _free>](p); }
            }
        }
    })+};
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
#[allow(unused_unsafe)] // Constructors differ in whether they require unsafe.
mod vector_buffers {
    use super::*;
    use urng::{RngV, Word};
    macro_rules! draw {
        ($r:expr, array, $n:expr, $bits:tt) => {
            unsafe { $r.nextu() }
        };
        ($r:expr, vector, $n:expr, $bits:tt) => {
            unsafe { std::mem::transmute::<_, [pastey::paste! {[<u $bits>]}; $n]>($r.nextuv()) }
        };
    }
    macro_rules! vector_stream {
        ($ty:ident,$prefix:ident,$bits:tt,$n:expr,$kind:ident; $($feature:literal),*)=>{pastey::paste!{
            #[test]
            fn [<$prefix _all_lanes_tails_numeric_ranges_and_chunk_boundaries>]() {
                let supported=true $(&& std::arch::is_x86_feature_detected!($feature))*;
                for seed in [0,1] {
                    let p=[<$prefix _new>](seed);
                    if !supported { assert!(p.is_null());continue; }
                    assert!(!p.is_null());
                    let mut native=unsafe{urng::$ty::new(seed)};
                    for len in [0,1,$n-1,$n,$n+1,63,64,65,131071,131072,131073] {
                        // Element-aligned, deliberately not aligned to a SIMD block.
                        let mut out=vec![[<u $bits>]::MAX;len+2];
                        unsafe{[<$prefix _next_u $bits s>](p,out.as_mut_ptr().add(1),len);}
                        assert_eq!(out[0],[<u $bits>]::MAX);assert_eq!(out[len+1],[<u $bits>]::MAX);
                        for chunk in out[1..len+1].chunks($n) {
                            let expected=draw!(native,$kind,$n,$bits);
                            assert_eq!(chunk,&expected[..chunk.len()]);
                        }
                    }
                    let mut ints=vec![0;35];
                    unsafe{[<$prefix _rand_i $bits s>](p,ints.as_mut_ptr(),ints.len(),[<i $bits>]::MIN,[<i $bits>]::MAX);}
                    for chunk in ints.chunks($n) {
                        let expected=draw!(native,$kind,$n,$bits).map(|x|x.to_randi([<i $bits>]::MIN,[<i $bits>]::MAX));
                        assert_eq!(chunk,&expected[..chunk.len()]);
                    }
                    let mut floats=vec![0.0;35];
                    unsafe{[<$prefix _next_f $bits s>](p,floats.as_mut_ptr(),floats.len());}
                    for chunk in floats.chunks($n) {
                        let expected=draw!(native,$kind,$n,$bits).map(Word::to_f01);
                        assert_eq!(chunk,&expected[..chunk.len()]);
                    }
                    let min=1.0 as [<f $bits>];let max=min.next_up();
                    unsafe{[<$prefix _rand_f $bits s>](p,floats.as_mut_ptr(),floats.len(),min,max);}
                    assert!(floats.iter().all(|&x|x==min));
                    unsafe{[<$prefix _free>](p);}
                }
            }
        }};
    }
    vector_stream!(Jsf32x16,jsf32x16,32,16,vector;"avx512f");
    vector_stream!(Pcg32x8,pcg32x8,32,8,array;"avx512f");
    vector_stream!(Sfc32x4,sfc32x4,32,4,array;);
    vector_stream!(Sfc32x8,sfc32x8,32,8,array;"avx2");
    vector_stream!(Sfc32x16,sfc32x16,32,16,array;"avx512f");
    vector_stream!(SplitMix32x16,splitmix32x16,32,16,vector;"avx512f");
    vector_stream!(Xoroshiro64Ssx8,xoroshiro64ssx8,32,8,vector;"avx2");
    vector_stream!(Xoroshiro64Ssx16,xoroshiro64ssx16,32,16,vector;"avx512f");
    vector_stream!(Xoshiro128Ppx16,xoshiro128ppx16,32,16,vector;"avx512f");
    vector_stream!(Xoshiro128Ssx16,xoshiro128ssx16,32,16,vector;"avx512f");
    vector_stream!(Biski64x8,biski64x8,64,8,array;"avx512f");
    vector_stream!(Cet64x8,cet64x8,64,8,array;"avx512f","avx512dq");
    vector_stream!(Cet256x2,cet256x2,64,2,array;"avx512f","avx512dq");
    vector_stream!(Sfc64x8,sfc64x8,64,8,array;"avx512f");
    vector_stream!(Xoshiro256Ssx2,xoshiro256ssx2,64,2,array;);
    vector_stream!(Philox32x4x4,philox32x4x4,32,16,array;"avx512f");
    vector_stream!(Squares32x8,squares32x8,32,8,array;"avx512f","avx512dq");
}

scalar_streams!(32;
 (Jsf32,jsf32),(Mt19937,mt19937),(Sfmt19937,sfmt19937),
 (Sfmt607,sfmt607),(Sfmt1279,sfmt1279),(Sfmt2281,sfmt2281),
 (Sfmt4253,sfmt4253),(Sfmt11213,sfmt11213),(Sfmt44497,sfmt44497),
 (Sfmt86243,sfmt86243),(Sfmt132049,sfmt132049),(Sfmt216091,sfmt216091),
 (Pcg32,pcg32),(Sfc32,sfc32),(SplitMix32,splitmix32),
 (Xoroshiro64Ss,xoroshiro64ss),(Xorshift32,xorshift32),(Xorshift128,xorshift128),
 (Xoshiro128Pp,xoshiro128pp),(Xoshiro128Ss,xoshiro128ss),
 (Philox32,philox32),(Squares32,squares32),
 (Threefry32x2,threefry32x2),(Threefry32x4,threefry32x4));
scalar_streams!(64;
 (Biski64,biski64),(Cet64,cet64),(Cet256,cet256),
 (Mt1993764,mt1993764),(Sfmt1993764,sfmt1993764),(Sfc64,sfc64),
 (SplitMix64,splitmix64),(TwistedGFSR,twisted_gfsr),(Xorshift64,xorshift64),
 (Xoroshiro128Pp,xoroshiro128pp),(Xoroshiro128Ss,xoroshiro128ss),
 (Xoshiro256Pp,xoshiro256pp),(Xoshiro256Ss,xoshiro256ss),
 (Philox64,philox64),(Threefish256,threefish256));

#[test]
fn null_zero_misalignment_overlap_and_invalid_lengths_do_not_access_memory() {
    unsafe {
        sfc32_next_u32s(std::ptr::null_mut(), std::ptr::null_mut(), 0);
        sfc32_next_u32s(std::ptr::null_mut(), std::ptr::null_mut(), 1);
        sfc32_free(std::ptr::null_mut());
        let p = sfc32_new(1);
        let mut out = [0xdeadbeefu32; 4];
        sfc32_next_u32s(p, out.as_mut_ptr().cast::<u8>().add(1).cast(), 1);
        sfc32_next_u32s(p, out.as_mut_ptr(), usize::MAX);
        sfc32_next_u32s(p, p.cast(), 1);
        sfc32_next_u32s(p, std::ptr::null_mut(), 4);
        assert_eq!(out, [0xdeadbeef; 4]);
        sfc32_next_u32s(p, out.as_mut_ptr(), 4);
        let mut scalar = urng::Sfc32::new(1);
        assert_eq!(out, std::array::from_fn(|_| scalar.nextu()));
        sfc32_free(p);
    }
}

#[test]
fn c_output_can_be_uninitialized_and_need_not_be_simd_aligned() {
    let mut raw = [std::mem::MaybeUninit::<u32>::uninit(); 19];
    let p = sfc32_new(17);
    unsafe {
        sfc32_next_u32s(p, raw.as_mut_ptr().cast(), raw.len());
        sfc32_free(p);
        let mut reference = urng::Sfc32::new(17);
        for value in raw {
            assert_eq!(value.assume_init(), reference.nextu());
        }
    }
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
#[test]
fn former_parallel_chunk_streams_are_not_reused() {
    if !std::arch::is_x86_feature_detected!("avx512f")
        || !std::arch::is_x86_feature_detected!("avx512dq")
    {
        eprintln!("AVX-512 hardware unavailable: this SIMD test requires AVX-512F/DQ");
        return;
    }
    unsafe {
        let mut out = vec![0u64; 131072 + 64];
        let p = xoshiro256ssx2_new(1);
        xoshiro256ssx2_next_u64s(p, out.as_mut_ptr(), out.len());
        xoshiro256ssx2_free(p);
        assert_ne!(&out[1..32], &out[131072..131072 + 31]);
        let p = cet64x8_new(1);
        cet64x8_next_u64s(p, out.as_mut_ptr(), out.len());
        cet64x8_free(p);
        assert_ne!(&out[1..64], &out[65536..65536 + 63]);
        let p = biski64x8_new(1);
        biski64x8_next_u64s(p, out.as_mut_ptr(), out.len());
        biski64x8_free(p);
        assert_ne!(&out[1..48], &out[16384..16384 + 47]);
    }
}
