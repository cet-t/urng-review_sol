//! Sequential C ABI adapters sharing the verified Rust generation core.
use crate::Threefish256;
use crate::{Rng, cabi::_internal::buffers};

/// Allocates a `Threefish256`; free with [`threefish256_free`]. Unsupported SIMD CPUs return null.
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub extern "C" fn threefish256_new(seed: u64) -> *mut Threefish256 {
    Box::into_raw(Box::new(unsafe { Threefish256::new(seed) }))
}

/// Frees a matching allocation exactly once. Null is a no-op.
/// # Safety
/// See the pointer, allocation, aliasing and ownership contract in [`crate::cabi`].
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub unsafe extern "C" fn threefish256_free(ptr: *mut Threefish256) {
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
pub unsafe extern "C" fn threefish256_next_u64s(
    ptr: *mut Threefish256,
    out: *mut u64,
    count: usize,
) {
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
pub unsafe extern "C" fn threefish256_next_f64s(
    ptr: *mut Threefish256,
    out: *mut f64,
    count: usize,
) {
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
pub unsafe extern "C" fn threefish256_rand_i64s(
    ptr: *mut Threefish256,
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
pub unsafe extern "C" fn threefish256_rand_f64s(
    ptr: *mut Threefish256,
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
