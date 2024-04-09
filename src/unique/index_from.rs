use crate::internal;
use crate::unique::unique_index::UniqueIndex;

pub trait IndexFrom<SourceIndex>: internal::Sealed {
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
    I0: UniqueIndex,
    usize: IndexFrom<I0>,
{
    fn index_from((i0,): (I0,)) -> Self {
        (usize::index_from(i0),)
    }
}

impl<I0, I1> IndexFrom<(I0, I1)> for (usize, usize)
where
    I0: UniqueIndex,
    I1: UniqueIndex,
    usize: IndexFrom<I0> + IndexFrom<I1>,
{
    fn index_from((i0, i1): (I0, I1)) -> Self {
        (usize::index_from(i0), usize::index_from(i1))
    }
}
