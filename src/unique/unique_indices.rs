use crate::unique::combinators::{IndexProduct, IndexZip};
use crate::{IndexFrom, RecordIndex};
use paradis_core::{LinearParAccess, ParAccess};
use std::marker::PhantomData;
use std::ops::{Range, RangeInclusive};

pub unsafe trait UniqueIndices: Sync + Send {
    type Index: Copy;

    unsafe fn get_unchecked(&self, i: usize) -> Self::Index;
    fn num_indices(&self) -> usize;

    fn get(&self, i: usize) -> Self::Index {
        assert!(i < self.num_indices(), "Index must be in bounds");
        unsafe { self.get_unchecked(i) }
    }

    /// Casts indices in this collection to the desired type.
    ///
    /// This method is generally used to convert index types smaller than `usize` or `usize` tuples
    /// to `usize` tuples. For example, a collection of `(u32, u32)` might be used as
    /// indices into a matrix data structure that can be indexed by `(usize, usize)`.
    fn cast_index_type<TargetIndex>(self) -> UniqueIndicesConvertedType<Self, TargetIndex>
    where
        Self: Sized,
        TargetIndex: Copy + IndexFrom<Self::Index>,
    {
        UniqueIndicesConvertedType {
            source_indices: self,
            marker: Default::default(),
        }
    }

    /// Returns the Cartesian product of this index set with another set of (unique) indices.
    fn index_product<I: UniqueIndices>(self, other: I) -> IndexProduct<Self, I>
    where
        Self: Sized,
    {
        IndexProduct(self, other)
    }

    /// Zips this index set with another.
    ///
    /// # Panics
    ///
    /// Panics if the other index set does not have the same number of elements as this index set.
    /// This behavior is distinct from Iterator::zip, which instead ignores elements in the longer
    /// collection.
    ///
    /// TODO: Better docs
    fn index_zip<I: UniqueIndices>(self, other: I) -> IndexZip<Self, I>
    where
        Self: Sized,
    {
        IndexZip::new(self, other)
    }
}

pub struct UniqueIndicesConvertedType<Indices, TargetIndex> {
    source_indices: Indices,
    marker: PhantomData<TargetIndex>,
}

unsafe impl<Indices, TargetIndex> UniqueIndices for UniqueIndicesConvertedType<Indices, TargetIndex>
where
    Indices: UniqueIndices,
    TargetIndex: Copy + RecordIndex + IndexFrom<Indices::Index>,
{
    type Index = TargetIndex;

    unsafe fn get_unchecked(&self, i: usize) -> Self::Index {
        // TODO: Cannot use TryFrom since it's not guaranteed to be
        let source_idx = self.source_indices.get_unchecked(i);
        TargetIndex::index_from(source_idx)
    }

    fn num_indices(&self) -> usize {
        self.source_indices.num_indices()
    }
}

#[derive(Debug)]
pub struct UniqueIndicesWithAccess<'a, Indices, Access> {
    pub(crate) indices: &'a Indices,
    pub(crate) access: Access,
}

unsafe impl<'a, Indices, Access> ParAccess<usize>
    for UniqueIndicesWithAccess<'a, Indices, Access>
where
    Indices: UniqueIndices,
    Access: ParAccess<Indices::Index>,
{
    type Record = Access::Record;
    type RecordMut = Access::RecordMut;

    #[inline(always)]
    unsafe fn clone_access(&self) -> Self {
        Self {
            indices: self.indices,
            access: unsafe { self.access.clone_access() },
        }
    }

    #[inline(always)]
    fn in_bounds(&self, index: usize) -> bool {
        let in_bounds_in_index_list = index < self.indices.num_indices();
        if in_bounds_in_index_list {
            let index = unsafe { self.indices.get_unchecked(index) };
            self.access.in_bounds(index)
        } else {
            false
        }
    }

    #[inline(always)]
    unsafe fn get_unsync_unchecked(&self, index: usize) -> Self::Record {
        // Cannot use unchecked indexing here, see note in _mut
        self.access.get_unsync(self.indices.get(index))
    }

    #[inline(always)]
    unsafe fn get_unsync_unchecked_mut(&self, index: usize) -> Self::RecordMut {
        // Note: We can not use unchecked indexing here because
        // we can not know that the index we obtain for indexing into the access
        // is actually in bounds
        unsafe {
            self.access
                .get_unsync_mut(self.indices.get_unchecked(index))
        }
    }
}

unsafe impl<'a, Indices, Access> LinearParAccess for UniqueIndicesWithAccess<'a, Indices, Access>
where
    Indices: UniqueIndices,
    Access: ParAccess<Indices::Index>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.indices.num_indices()
    }
}

unsafe impl UniqueIndices for Range<usize> {
    type Index = usize;

    unsafe fn get_unchecked(&self, i: usize) -> usize {
        self.start + i
    }

    fn num_indices(&self) -> usize {
        self.end.saturating_sub(self.start)
    }
}

unsafe impl UniqueIndices for RangeInclusive<usize> {
    type Index = usize;

    unsafe fn get_unchecked(&self, i: usize) -> usize {
        self.start() + i
    }

    fn num_indices(&self) -> usize {
        self.clone().count()
    }
}
