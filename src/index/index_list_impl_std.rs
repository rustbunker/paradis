use crate::index::{IndexList, UniqueIndexList};
use paradis_core::Bounds;
use std::ops::{Range, RangeInclusive};

unsafe impl IndexList for Range<usize> {
    type Index = usize;
    const ALWAYS_BOUNDED: bool = true;

    #[inline(always)]
    unsafe fn get_index_unchecked(&self, i: usize) -> usize {
        self.start + i
    }

    #[inline(always)]
    fn num_indices(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    #[inline]
    fn bounds(&self) -> Option<Bounds<Self::Index>> {
        Some(Bounds {
            offset: self.start,
            extent: self.num_indices(),
        })
    }
}

unsafe impl UniqueIndexList for Range<usize> {}

unsafe impl IndexList for RangeInclusive<usize> {
    type Index = usize;
    const ALWAYS_BOUNDED: bool = true;

    #[inline(always)]
    unsafe fn get_index_unchecked(&self, i: usize) -> usize {
        self.start() + i
    }

    #[inline(always)]
    fn num_indices(&self) -> usize {
        self.clone().count()
    }

    #[inline]
    fn bounds(&self) -> Option<Bounds<Self::Index>> {
        Some(Bounds {
            offset: *self.start(),
            extent: self.num_indices(),
        })
    }
}

unsafe impl UniqueIndexList for RangeInclusive<usize> {}

unsafe impl<I: Copy + Send + Sync> IndexList for Vec<I> {
    type Index = I;
    const ALWAYS_BOUNDED: bool = false;

    unsafe fn get_index_unchecked(&self, loc: usize) -> Self::Index {
        unsafe { *<[I]>::get_unchecked(self.as_slice(), loc) }
    }

    fn num_indices(&self) -> usize {
        self.len()
    }

    fn bounds(&self) -> Option<Bounds<Self::Index>> {
        None
    }
}
