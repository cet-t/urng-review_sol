use crate::{Rng, Word};

/// Validates representable size, null/alignment, and state/output overlap before
/// creating references. It cannot validate pointer provenance or allocation size.
///
/// # Safety
/// Non-null pointers must refer to live, initialized state and writable output
/// allocations of the specified sizes, exclusively borrowed for this call.
pub(crate) unsafe fn buffers<'a, R, T>(
    ptr: *mut R,
    out: *mut T,
    count: usize,
) -> Option<(&'a mut R, &'a mut [std::mem::MaybeUninit<T>])> {
    if count == 0 || ptr.is_null() || out.is_null() || !ptr.is_aligned() || !out.is_aligned() {
        return None;
    }
    let bytes = count.checked_mul(size_of::<T>())?;
    if bytes > isize::MAX as usize {
        return None;
    }
    let begin = out as usize;
    let end = begin.checked_add(bytes)?;
    let state = ptr as usize;
    let state_end = state.checked_add(size_of::<R>())?;
    if begin < state_end && state < end {
        return None;
    }
    // C output buffers may be uninitialized. Never form &mut [T] before writes.
    Some(unsafe { (&mut *ptr, std::slice::from_raw_parts_mut(out.cast(), count)) })
}

/// Scalar adapter for a block-producing generator. A partial final block is
/// consumed in full; subsequent C calls start at the next block boundary.
pub(crate) struct BlockRng<'a, R, W: Word, const N: usize, F> {
    rng: &'a mut R,
    next: F,
    buf: [W; N],
    pos: usize,
}

impl<'a, R, W: Word + Default, const N: usize, F: FnMut(&mut R) -> [W; N]>
    BlockRng<'a, R, W, N, F>
{
    pub(crate) fn new(rng: &'a mut R, next: F) -> Self {
        assert!(N != 0);
        Self {
            rng,
            next,
            buf: [W::default(); N],
            pos: N,
        }
    }
}

impl<R, W: Word, const N: usize, F: FnMut(&mut R) -> [W; N]> Rng for BlockRng<'_, R, W, N, F> {
    type Word = W;
    fn nextu(&mut self) -> W {
        if self.pos == N {
            self.buf = (self.next)(self.rng);
            self.pos = 0;
        }
        let word = self.buf[self.pos];
        self.pos += 1;
        word
    }
}
