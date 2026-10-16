//! C-compatible ABI wrappers for the RNGs.
//!
//! # Safety and stream contract
//! Free/fill functions are unsafe for Rust callers. State must come from the
//! matching `_new`, be exclusively owned, and be freed exactly once. Non-empty
//! output must be a live writable allocation, disjoint from the state.
//! Output elements may be uninitialized; successful fills initialize them.
//! Zero-length fills are no-ops (null pointers allowed). Null/misaligned pointers,
//! unrepresentable lengths, overlapping state/output, and invalid ranges are
//! rejected without accessing memory. These checks cannot validate provenance,
//! allocation extent, use-after-free, concurrent access, or double-free.
//!
//! Scalar fills follow the Rust `Rng` stream, independently of call boundaries.
//! SIMD fills flatten blocks in lane order and consume a partial final block in
//! full. No parallel reseeding or non-temporal stores are used. SIMD constructors
//! return null on unsupported CPUs; symbols are available on x86_64 only.
//!
//! ```compile_fail
//! let p = urng::cabi::sfc32_new(1);
//! // Destruction carries an ownership obligation, even though the ABI is C.
//! urng::cabi::sfc32_free(p);
//! ```

#![allow(ambiguous_glob_reexports)]

mod _internal;

pub mod cbrng;
pub mod prng;

pub use cbrng::*;
pub use prng::*;
