//! Sequential C ABI adapters sharing the verified Rust generation core.
use crate::Philox32;
#[cfg(all(feature = "simd", target_arch = "x86_64"))]
use crate::Philox32x4x4;
#[cfg(all(feature = "simd", target_arch = "x86_64"))]
use crate::cabi::_internal::BlockRng;
use crate::{Rng, cabi::_internal::buffers};

/// Allocates a `Philox32`; free with [`philox32_free`]. Unsupported SIMD CPUs return null.
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub extern "C" fn philox32_new(seed: u32) -> *mut Philox32 {
    Box::into_raw(Box::new(unsafe { Philox32::new(seed) }))
}

/// Frees a matching allocation exactly once. Null is a no-op.
/// # Safety
/// See the pointer, allocation, aliasing and ownership contract in [`crate::cabi`].
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub unsafe extern "C" fn philox32_free(ptr: *mut Philox32) {
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
pub unsafe extern "C" fn philox32_next_u32s(ptr: *mut Philox32, out: *mut u32, count: usize) {
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
pub unsafe extern "C" fn philox32_next_f32s(ptr: *mut Philox32, out: *mut f32, count: usize) {
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
pub unsafe extern "C" fn philox32_rand_i32s(
    ptr: *mut Philox32,
    out: *mut i32,
    count: usize,
    min: i32,
    max: i32,
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
pub unsafe extern "C" fn philox32_rand_f32s(
    ptr: *mut Philox32,
    out: *mut f32,
    count: usize,
    min: f32,
    max: f32,
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
/// Allocates a `Philox32x4x4`; free with [`philox32x4x4_free`]. Unsupported SIMD CPUs return null.
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub extern "C" fn philox32x4x4_new(seed: u32) -> *mut Philox32x4x4 {
    if !(std::arch::is_x86_feature_detected!("avx512f")) {
        return std::ptr::null_mut();
    }
    Box::into_raw(Box::new(unsafe { Philox32x4x4::new(seed) }))
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
/// Frees a matching allocation exactly once. Null is a no-op.
/// # Safety
/// See the pointer, allocation, aliasing and ownership contract in [`crate::cabi`].
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub unsafe extern "C" fn philox32x4x4_free(ptr: *mut Philox32x4x4) {
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
pub unsafe extern "C" fn philox32x4x4_next_u32s(
    ptr: *mut Philox32x4x4,
    out: *mut u32,
    count: usize,
) {
    let Some((rng, buffer)) = (unsafe { buffers(ptr, out, count) }) else {
        return;
    };
    let mut stream = BlockRng::<_, u32, 16, _>::new(rng, |r| unsafe { r.nextu() });
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
pub unsafe extern "C" fn philox32x4x4_next_f32s(
    ptr: *mut Philox32x4x4,
    out: *mut f32,
    count: usize,
) {
    let Some((rng, buffer)) = (unsafe { buffers(ptr, out, count) }) else {
        return;
    };
    let mut stream = BlockRng::<_, u32, 16, _>::new(rng, |r| unsafe { r.nextu() });
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
pub unsafe extern "C" fn philox32x4x4_rand_i32s(
    ptr: *mut Philox32x4x4,
    out: *mut i32,
    count: usize,
    min: i32,
    max: i32,
) {
    if min > max {
        return;
    }
    let Some((rng, buffer)) = (unsafe { buffers(ptr, out, count) }) else {
        return;
    };
    let mut stream = BlockRng::<_, u32, 16, _>::new(rng, |r| unsafe { r.nextu() });
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
pub unsafe extern "C" fn philox32x4x4_rand_f32s(
    ptr: *mut Philox32x4x4,
    out: *mut f32,
    count: usize,
    min: f32,
    max: f32,
) {
    if !min.is_finite() || !max.is_finite() || min >= max {
        return;
    }
    let Some((rng, buffer)) = (unsafe { buffers(ptr, out, count) }) else {
        return;
    };
    let mut stream = BlockRng::<_, u32, 16, _>::new(rng, |r| unsafe { r.nextu() });
    for dst in buffer {
        dst.write(stream.randf(min, max));
    }
}
