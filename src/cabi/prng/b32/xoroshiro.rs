//! Sequential C ABI adapters sharing the verified Rust generation core.
use crate::Xoroshiro64Ss;
#[cfg(all(feature = "simd", target_arch = "x86_64"))]
use crate::cabi::_internal::BlockRng;
use crate::{Rng, cabi::_internal::buffers};
#[cfg(all(feature = "simd", target_arch = "x86_64"))]
use crate::{RngV, Xoroshiro64Ssx8, Xoroshiro64Ssx16};

/// Allocates a `Xoroshiro64Ss`; free with [`xoroshiro64ss_free`]. Unsupported SIMD CPUs return null.
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub extern "C" fn xoroshiro64ss_new(seed: u32) -> *mut Xoroshiro64Ss {
    Box::into_raw(Box::new(unsafe { Xoroshiro64Ss::new(seed) }))
}

/// Frees a matching allocation exactly once. Null is a no-op.
/// # Safety
/// See the pointer, allocation, aliasing and ownership contract in [`crate::cabi`].
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub unsafe extern "C" fn xoroshiro64ss_free(ptr: *mut Xoroshiro64Ss) {
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
pub unsafe extern "C" fn xoroshiro64ss_next_u32s(
    ptr: *mut Xoroshiro64Ss,
    out: *mut u32,
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
pub unsafe extern "C" fn xoroshiro64ss_next_f32s(
    ptr: *mut Xoroshiro64Ss,
    out: *mut f32,
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
pub unsafe extern "C" fn xoroshiro64ss_rand_i32s(
    ptr: *mut Xoroshiro64Ss,
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
pub unsafe extern "C" fn xoroshiro64ss_rand_f32s(
    ptr: *mut Xoroshiro64Ss,
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
/// Allocates a `Xoroshiro64Ssx8`; free with [`xoroshiro64ssx8_free`]. Unsupported SIMD CPUs return null.
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub extern "C" fn xoroshiro64ssx8_new(seed: u32) -> *mut Xoroshiro64Ssx8 {
    if !(std::arch::is_x86_feature_detected!("avx2")) {
        return std::ptr::null_mut();
    }
    Box::into_raw(Box::new(unsafe { Xoroshiro64Ssx8::new(seed) }))
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
/// Frees a matching allocation exactly once. Null is a no-op.
/// # Safety
/// See the pointer, allocation, aliasing and ownership contract in [`crate::cabi`].
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub unsafe extern "C" fn xoroshiro64ssx8_free(ptr: *mut Xoroshiro64Ssx8) {
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
pub unsafe extern "C" fn xoroshiro64ssx8_next_u32s(
    ptr: *mut Xoroshiro64Ssx8,
    out: *mut u32,
    count: usize,
) {
    let Some((rng, buffer)) = (unsafe { buffers(ptr, out, count) }) else {
        return;
    };
    let mut stream =
        BlockRng::<_, u32, 8, _>::new(rng, |r| unsafe { std::mem::transmute(r.nextuv()) });
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
pub unsafe extern "C" fn xoroshiro64ssx8_next_f32s(
    ptr: *mut Xoroshiro64Ssx8,
    out: *mut f32,
    count: usize,
) {
    let Some((rng, buffer)) = (unsafe { buffers(ptr, out, count) }) else {
        return;
    };
    let mut stream =
        BlockRng::<_, u32, 8, _>::new(rng, |r| unsafe { std::mem::transmute(r.nextuv()) });
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
pub unsafe extern "C" fn xoroshiro64ssx8_rand_i32s(
    ptr: *mut Xoroshiro64Ssx8,
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
    let mut stream =
        BlockRng::<_, u32, 8, _>::new(rng, |r| unsafe { std::mem::transmute(r.nextuv()) });
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
pub unsafe extern "C" fn xoroshiro64ssx8_rand_f32s(
    ptr: *mut Xoroshiro64Ssx8,
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
    let mut stream =
        BlockRng::<_, u32, 8, _>::new(rng, |r| unsafe { std::mem::transmute(r.nextuv()) });
    for dst in buffer {
        dst.write(stream.randf(min, max));
    }
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
/// Allocates a `Xoroshiro64Ssx16`; free with [`xoroshiro64ssx16_free`]. Unsupported SIMD CPUs return null.
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub extern "C" fn xoroshiro64ssx16_new(seed: u32) -> *mut Xoroshiro64Ssx16 {
    if !(std::arch::is_x86_feature_detected!("avx512f")) {
        return std::ptr::null_mut();
    }
    Box::into_raw(Box::new(unsafe { Xoroshiro64Ssx16::new(seed) }))
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
/// Frees a matching allocation exactly once. Null is a no-op.
/// # Safety
/// See the pointer, allocation, aliasing and ownership contract in [`crate::cabi`].
#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub unsafe extern "C" fn xoroshiro64ssx16_free(ptr: *mut Xoroshiro64Ssx16) {
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
pub unsafe extern "C" fn xoroshiro64ssx16_next_u32s(
    ptr: *mut Xoroshiro64Ssx16,
    out: *mut u32,
    count: usize,
) {
    let Some((rng, buffer)) = (unsafe { buffers(ptr, out, count) }) else {
        return;
    };
    let mut stream =
        BlockRng::<_, u32, 16, _>::new(rng, |r| unsafe { std::mem::transmute(r.nextuv()) });
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
pub unsafe extern "C" fn xoroshiro64ssx16_next_f32s(
    ptr: *mut Xoroshiro64Ssx16,
    out: *mut f32,
    count: usize,
) {
    let Some((rng, buffer)) = (unsafe { buffers(ptr, out, count) }) else {
        return;
    };
    let mut stream =
        BlockRng::<_, u32, 16, _>::new(rng, |r| unsafe { std::mem::transmute(r.nextuv()) });
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
pub unsafe extern "C" fn xoroshiro64ssx16_rand_i32s(
    ptr: *mut Xoroshiro64Ssx16,
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
    let mut stream =
        BlockRng::<_, u32, 16, _>::new(rng, |r| unsafe { std::mem::transmute(r.nextuv()) });
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
pub unsafe extern "C" fn xoroshiro64ssx16_rand_f32s(
    ptr: *mut Xoroshiro64Ssx16,
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
    let mut stream =
        BlockRng::<_, u32, 16, _>::new(rng, |r| unsafe { std::mem::transmute(r.nextuv()) });
    for dst in buffer {
        dst.write(stream.randf(min, max));
    }
}
