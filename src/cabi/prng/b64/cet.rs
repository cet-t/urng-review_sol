//! Sequential C ABI adapters sharing the verified Rust generation core.
#[cfg(all(feature = "simd", target_arch = "x86_64"))]
use crate::cabi::_internal::BlockRng;
use crate::{Cet64, Cet256};
#[cfg(all(feature = "simd", target_arch = "x86_64"))]
use crate::{Cet64x8, Cet256x2};
use crate::{Rng, cabi::_internal::buffers};

/// Allocates a `Cet64`; free with [`cet64_free`]. Unsupported SIMD CPUs return null.
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub extern "C" fn cet64_new(seed: u64) -> *mut Cet64 {
    Box::into_raw(Box::new(unsafe { Cet64::new(seed) }))
}

/// Frees a matching allocation exactly once. Null is a no-op.
/// # Safety
/// See the pointer, allocation, aliasing and ownership contract in [`crate::cabi`].
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub unsafe extern "C" fn cet64_free(ptr: *mut Cet64) {
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
pub unsafe extern "C" fn cet64_next_u64s(ptr: *mut Cet64, out: *mut u64, count: usize) {
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
pub unsafe extern "C" fn cet64_next_f64s(ptr: *mut Cet64, out: *mut f64, count: usize) {
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
pub unsafe extern "C" fn cet64_rand_i64s(
    ptr: *mut Cet64,
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
pub unsafe extern "C" fn cet64_rand_f64s(
    ptr: *mut Cet64,
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

/// Allocates a `Cet256`; free with [`cet256_free`]. Unsupported SIMD CPUs return null.
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub extern "C" fn cet256_new(seed: u64) -> *mut Cet256 {
    Box::into_raw(Box::new(unsafe { Cet256::new(seed) }))
}

/// Frees a matching allocation exactly once. Null is a no-op.
/// # Safety
/// See the pointer, allocation, aliasing and ownership contract in [`crate::cabi`].
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub unsafe extern "C" fn cet256_free(ptr: *mut Cet256) {
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
pub unsafe extern "C" fn cet256_next_u64s(ptr: *mut Cet256, out: *mut u64, count: usize) {
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
pub unsafe extern "C" fn cet256_next_f64s(ptr: *mut Cet256, out: *mut f64, count: usize) {
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
pub unsafe extern "C" fn cet256_rand_i64s(
    ptr: *mut Cet256,
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
pub unsafe extern "C" fn cet256_rand_f64s(
    ptr: *mut Cet256,
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
/// Allocates a `Cet64x8`; free with [`cet64x8_free`]. Unsupported SIMD CPUs return null.
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub extern "C" fn cet64x8_new(seed: u64) -> *mut Cet64x8 {
    if !(std::arch::is_x86_feature_detected!("avx512f")
        && std::arch::is_x86_feature_detected!("avx512dq"))
    {
        return std::ptr::null_mut();
    }
    Box::into_raw(Box::new(unsafe { Cet64x8::new(seed) }))
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
/// Frees a matching allocation exactly once. Null is a no-op.
/// # Safety
/// See the pointer, allocation, aliasing and ownership contract in [`crate::cabi`].
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub unsafe extern "C" fn cet64x8_free(ptr: *mut Cet64x8) {
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
pub unsafe extern "C" fn cet64x8_next_u64s(ptr: *mut Cet64x8, out: *mut u64, count: usize) {
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
pub unsafe extern "C" fn cet64x8_next_f64s(ptr: *mut Cet64x8, out: *mut f64, count: usize) {
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
pub unsafe extern "C" fn cet64x8_rand_i64s(
    ptr: *mut Cet64x8,
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
pub unsafe extern "C" fn cet64x8_rand_f64s(
    ptr: *mut Cet64x8,
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

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
/// Allocates a `Cet256x2`; free with [`cet256x2_free`]. Unsupported SIMD CPUs return null.
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub extern "C" fn cet256x2_new(seed: u64) -> *mut Cet256x2 {
    if !(std::arch::is_x86_feature_detected!("avx512f")
        && std::arch::is_x86_feature_detected!("avx512dq"))
    {
        return std::ptr::null_mut();
    }
    Box::into_raw(Box::new(unsafe { Cet256x2::new(seed) }))
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
/// Frees a matching allocation exactly once. Null is a no-op.
/// # Safety
/// See the pointer, allocation, aliasing and ownership contract in [`crate::cabi`].
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub unsafe extern "C" fn cet256x2_free(ptr: *mut Cet256x2) {
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
pub unsafe extern "C" fn cet256x2_next_u64s(ptr: *mut Cet256x2, out: *mut u64, count: usize) {
    let Some((rng, buffer)) = (unsafe { buffers(ptr, out, count) }) else {
        return;
    };
    let mut stream = BlockRng::<_, u64, 2, _>::new(rng, |r| unsafe { r.nextu() });
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
pub unsafe extern "C" fn cet256x2_next_f64s(ptr: *mut Cet256x2, out: *mut f64, count: usize) {
    let Some((rng, buffer)) = (unsafe { buffers(ptr, out, count) }) else {
        return;
    };
    let mut stream = BlockRng::<_, u64, 2, _>::new(rng, |r| unsafe { r.nextu() });
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
pub unsafe extern "C" fn cet256x2_rand_i64s(
    ptr: *mut Cet256x2,
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
    let mut stream = BlockRng::<_, u64, 2, _>::new(rng, |r| unsafe { r.nextu() });
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
pub unsafe extern "C" fn cet256x2_rand_f64s(
    ptr: *mut Cet256x2,
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
    let mut stream = BlockRng::<_, u64, 2, _>::new(rng, |r| unsafe { r.nextu() });
    for dst in buffer {
        dst.write(stream.randf(min, max));
    }
}
