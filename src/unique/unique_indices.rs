use crate::unique::combinators::{IndexCast, IndexFlatten, IndexProduct, IndexZip};
use crate::IndexFrom;
use paradis_core::{LinearParAccess, ParAccess};
use std::ops::{Range, RangeInclusive};

/// A finite list of indices.
pub unsafe trait IndexList: Sync + Send {
    type Index: Copy;

    unsafe fn get_unchecked(&self, loc: usize) -> Self::Index;

    fn num_indices(&self) -> usize;

    fn get(&self, i: usize) -> Self::Index {
        assert!(i < self.num_indices(), "Index must be in bounds");
        unsafe { self.get_unchecked(i) }
    }
}

/// A finite list of *unique* indices.
pub unsafe trait UniqueIndexList: IndexList {
    /// Casts indices in this collection to the desired type.
    ///
    /// This method is generally used to convert index types smaller than `usize` or `usize` tuples
    /// to `usize` tuples. For example, a collection of `(u32, u32)` might be used as
    /// indices into a matrix data structure that can be indexed by `(usize, usize)`.
    fn index_cast<TargetIndex>(self) -> IndexCast<Self, TargetIndex>
    where
        Self: Sized,
        TargetIndex: Copy + IndexFrom<Self::Index>,
    {
        IndexCast {
            source_indices: self,
            marker: Default::default(),
        }
    }

    /// Returns the Cartesian product of this index set with another set of (unique) indices.
    fn index_product<I: UniqueIndexList>(self, other: I) -> IndexProduct<Self, I>
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
    fn index_zip<I: UniqueIndexList>(self, other: I) -> IndexZip<Self, I>
    where
        Self: Sized,
    {
        IndexZip::new(self, other)
    }

    /// Flattens nested tuple indices.
    ///
    /// TODO: More docs, examples
    fn index_flatten(self) -> IndexFlatten<Self>
    where
        Self: Sized,
    {
        IndexFlatten(self)
    }
}

/// An access object that has been narrowed to a subset of its index set.
///
/// This is the result type for
/// [compose_access_with_indices](crate::unique::compose_access_with_indices).
#[derive(Debug)]
pub struct IndexedAccess<'a, Indices, Access> {
    pub(crate) indices: &'a Indices,
    pub(crate) access: Access,
}

unsafe impl<'a, Indices, Access> ParAccess<usize> for IndexedAccess<'a, Indices, Access>
where
    Indices: UniqueIndexList,
    Access: ParAccess<Indices::Index>,
{
    type Record = Access::Record;

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
        // Note: We can not use unchecked indexing here because
        // we can not know that the index we obtain for indexing into the access
        // is actually in bounds
        self.access.get_unsync(self.indices.get(index))
    }
}

unsafe impl<'a, Indices, Access> LinearParAccess for IndexedAccess<'a, Indices, Access>
where
    Indices: UniqueIndexList,
    Access: ParAccess<Indices::Index>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.indices.num_indices()
    }
}

unsafe impl IndexList for Range<usize> {
    type Index = usize;

    unsafe fn get_unchecked(&self, i: usize) -> usize {
        self.start + i
    }

    fn num_indices(&self) -> usize {
        self.end.saturating_sub(self.start)
    }
}

unsafe impl UniqueIndexList for Range<usize> {}

unsafe impl IndexList for RangeInclusive<usize> {
    type Index = usize;

    unsafe fn get_unchecked(&self, i: usize) -> usize {
        self.start() + i
    }

    fn num_indices(&self) -> usize {
        self.clone().count()
    }
}

unsafe impl UniqueIndexList for RangeInclusive<usize> {}
