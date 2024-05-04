use crate::internal;
use crate::RecordIndex;

/// Enables conversion of indices.
///
/// This trait is *sealed*, meaning that it cannot be implemented for external types.
/// The intention behind this trait is to permit converting e.g. `u16` or `u32` type indices
/// (or tuples thereof) to `usize`-based indices. This is primarily useful for saving space
/// when the indices need to be explicitly stored.
///
/// TODO: Extend impls to higher-arity tuples
pub trait IndexFrom<SourceIndex>: internal::Sealed {
    /// Convert the source index to `Self`, the target index type.
    fn index_from(source: SourceIndex) -> Self;
}

impl IndexFrom<usize> for usize {
    fn index_from(source: usize) -> Self {
        source
    }
}

#[cfg(any(
    target_pointer_width = "32",
    target_pointer_width = "64",
    target_pointer_width = "128",
))]
impl IndexFrom<u32> for usize {
    fn index_from(source: u32) -> Self {
        source
            .try_into()
            .expect("Can always convert u32 to usize since we assume usize is at least 32 bits")
    }
}

#[cfg(any(target_pointer_width = "64", target_pointer_width = "128",))]
impl IndexFrom<u64> for usize {
    fn index_from(source: u64) -> Self {
        source
            .try_into()
            .expect("Can always convert u64 to usize since we assume usize is at least 64 bits")
    }
}

impl<I0> IndexFrom<(I0,)> for (usize,)
where
    I0: RecordIndex,
    usize: IndexFrom<I0>,
{
    fn index_from((i0,): (I0,)) -> Self {
        (usize::index_from(i0),)
    }
}

impl<I0, I1> IndexFrom<(I0, I1)> for (usize, usize)
where
    I0: RecordIndex,
    I1: RecordIndex,
    usize: IndexFrom<I0> + IndexFrom<I1>,
{
    fn index_from((i0, i1): (I0, I1)) -> Self {
        (usize::index_from(i0), usize::index_from(i1))
    }
}
