//! Sequential C ABI adapters sharing the verified Rust generation core.
use crate::Sfc64;
#[cfg(all(feature = "simd", target_arch = "x86_64"))]
use crate::Sfc64x8;
#[cfg(all(feature = "simd", target_arch = "x86_64"))]
use crate::cabi::_internal::BlockRng;
use crate::{Rng, cabi::_internal::buffers};

/// Allocates a `Sfc64`; free with [`sfc64_free`]. Unsupported SIMD CPUs return null.
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub extern "C" fn sfc64_new(seed: u64) -> *mut Sfc64 {
    Box::into_raw(Box::new(unsafe { Sfc64::new(seed) }))
}

/// Frees a matching allocation exactly once. Null is a no-op.
/// # Safety
/// See the pointer, allocation, aliasing and ownership contract in [`crate::cabi`].
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub unsafe extern "C" fn sfc64_free(ptr: *mut Sfc64) {
    if !ptr.is_null() && ptr.is_aligned() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
}

/// Fills the output sequentially. Invalid sizes/pointers/ranges are no-ops.
/// # Safety
/// See the pointer, allocation, aliasing and ownership contract in [`crate::cabi`].
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub unsafe extern "C" fn sfc64_next_u64s(ptr: *mut Sfc64, out: *mut u64, count: usize) {
    let Some((rng, buffer)) = (unsafe { buffers(ptr, out, count) }) else {
        return;
    };
    for dst in buffer {
        dst.write(rng.nextu());
    }
}

/// Fills the output sequentially. Invalid sizes/pointers/ranges are no-ops.
/// # Safety
/// See the pointer, allocation, aliasing and ownership contract in [`crate::cabi`].
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub unsafe extern "C" fn sfc64_next_f64s(ptr: *mut Sfc64, out: *mut f64, count: usize) {
    let Some((rng, buffer)) = (unsafe { buffers(ptr, out, count) }) else {
        return;
    };
    for dst in buffer {
        dst.write(rng.nextf());
    }
}

/// Fills the output sequentially. Invalid sizes/pointers/ranges are no-ops.
/// # Safety
/// See the pointer, allocation, aliasing and ownership contract in [`crate::cabi`].
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub unsafe extern "C" fn sfc64_rand_i64s(
    ptr: *mut Sfc64,
    out: *mut i64,
    count: usize,
    min: i64,
    max: i64,
) {
    if min > max {
        return;
    }
    let Some((rng, buffer)) = (unsafe { buffers(ptr, out, count) }) else {
        return;
    };
    for dst in buffer {
        dst.write(rng.randi(min, max));
    }
}

/// Fills the output sequentially. Invalid sizes/pointers/ranges are no-ops.
/// # Safety
/// See the pointer, allocation, aliasing and ownership contract in [`crate::cabi`].
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub unsafe extern "C" fn sfc64_rand_f64s(
    ptr: *mut Sfc64,
    out: *mut f64,
    count: usize,
    min: f64,
    max: f64,
) {
    if !min.is_finite() || !max.is_finite() || min >= max {
        return;
    }
    let Some((rng, buffer)) = (unsafe { buffers(ptr, out, count) }) else {
        return;
    };
    for dst in buffer {
        dst.write(rng.randf(min, max));
    }
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
/// Allocates a `Sfc64x8`; free with [`sfc64x8_free`]. Unsupported SIMD CPUs return null.
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub extern "C" fn sfc64x8_new(seed: u64) -> *mut Sfc64x8 {
    if !(std::arch::is_x86_feature_detected!("avx512f")) {
        return std::ptr::null_mut();
    }
    Box::into_raw(Box::new(unsafe { Sfc64x8::new(seed) }))
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
/// Frees a matching allocation exactly once. Null is a no-op.
/// # Safety
/// See the pointer, allocation, aliasing and ownership contract in [`crate::cabi`].
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub unsafe extern "C" fn sfc64x8_free(ptr: *mut Sfc64x8) {
    if !ptr.is_null() && ptr.is_aligned() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
/// Fills the output sequentially. Invalid sizes/pointers/ranges are no-ops.
/// # Safety
/// See the pointer, allocation, aliasing and ownership contract in [`crate::cabi`].
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub unsafe extern "C" fn sfc64x8_next_u64s(ptr: *mut Sfc64x8, out: *mut u64, count: usize) {
    let Some((rng, buffer)) = (unsafe { buffers(ptr, out, count) }) else {
        return;
    };
    let mut stream = BlockRng::<_, u64, 8, _>::new(rng, |r| unsafe { r.nextu() });
    for dst in buffer {
        dst.write(stream.nextu());
    }
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
/// Fills the output sequentially. Invalid sizes/pointers/ranges are no-ops.
/// # Safety
/// See the pointer, allocation, aliasing and ownership contract in [`crate::cabi`].
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub unsafe extern "C" fn sfc64x8_next_f64s(ptr: *mut Sfc64x8, out: *mut f64, count: usize) {
    let Some((rng, buffer)) = (unsafe { buffers(ptr, out, count) }) else {
        return;
    };
    let mut stream = BlockRng::<_, u64, 8, _>::new(rng, |r| unsafe { r.nextu() });
    for dst in buffer {
        dst.write(stream.nextf());
    }
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
/// Fills the output sequentially. Invalid sizes/pointers/ranges are no-ops.
/// # Safety
/// See the pointer, allocation, aliasing and ownership contract in [`crate::cabi`].
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub unsafe extern "C" fn sfc64x8_rand_i64s(
    ptr: *mut Sfc64x8,
    out: *mut i64,
    count: usize,
    min: i64,
    max: i64,
) {
    if min > max {
        return;
    }
    let Some((rng, buffer)) = (unsafe { buffers(ptr, out, count) }) else {
        return;
    };
    let mut stream = BlockRng::<_, u64, 8, _>::new(rng, |r| unsafe { r.nextu() });
    for dst in buffer {
        dst.write(stream.randi(min, max));
    }
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
/// Fills the output sequentially. Invalid sizes/pointers/ranges are no-ops.
/// # Safety
/// See the pointer, allocation, aliasing and ownership contract in [`crate::cabi`].
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub unsafe extern "C" fn sfc64x8_rand_f64s(
    ptr: *mut Sfc64x8,
    out: *mut f64,
    count: usize,
    min: f64,
    max: f64,
) {
    if !min.is_finite() || !max.is_finite() || min >= max {
        return;
    }
    let Some((rng, buffer)) = (unsafe { buffers(ptr, out, count) }) else {
        return;
    };
    let mut stream = BlockRng::<_, u64, 8, _>::new(rng, |r| unsafe { r.nextu() });
    for dst in buffer {
        dst.write(stream.randf(min, max));
    }
}
