use crate::error::{NonUniqueIndex, OutOfBounds};
use crate::unique::combinators::{
    IndexAZip, IndexCast, IndexFlatten, IndexProduct, IndexTranspose, IndexZip,
};
use crate::unique::CheckedUnique;
use crate::IndexFrom;
use paradis_core::{Bounds, LinearParAccess, ParAccess, RecordIndex};
use std::any::type_name;
use std::hash::Hash;
use std::ops::{Range, RangeInclusive};

/// A finite list of indices.
pub unsafe trait IndexList: Sync + Send {
    /// The index type contained in this index list.
    type Index: Copy;

    /// Signals whether this index list is *always* bounded.
    ///
    /// This means that [`bounds`](Self::bounds) never returns `None`. This can be used to statically eliminate
    /// bounds checks in some circumstances.
    const ALWAYS_BOUNDED: bool;

    /// Obtain the index at the given location.
    ///
    /// No bounds checks are performed.
    unsafe fn get_index_unchecked(&self, loc: usize) -> Self::Index;

    /// The number of indices in this index list.
    fn num_indices(&self) -> usize;

    /// Return the tightest bounds that contain all indices in this index list, if possible.
    fn bounds(&self) -> Option<Bounds<Self::Index>>;

    /// Returns the index at the given location.
    ///
    /// # Panics
    ///
    /// Panics if the location is out of bounds.
    fn get_index(&self, loc: usize) -> Self::Index {
        assert!(loc < self.num_indices(), "Index must be in bounds");
        unsafe { self.get_index_unchecked(loc) }
    }

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
    fn index_product<I: IndexList>(self, other: I) -> IndexProduct<Self, I>
    where
        Self: Sized,
    {
        IndexProduct(self, other)
    }

    /// Zips this index list with another.
    ///
    /// Specifically, if `a` and `b` are lists, then the elements of `a.index_zip(b)`
    /// are `(a[0], b[0]), (a[1], b[1]), ...`.
    ///
    /// # Uniqueness
    ///
    /// The resulting indices are unique if *either* of the two index lists have unique indices.
    /// However, this cannot be expressed in the type system. Therefore, the resulting indices
    /// are unique only if the first index list has unique indices.
    /// Use [`index_azip`](Self::index_azip) if only the second list has unique indices.
    ///
    /// # Panics
    ///
    /// Panics if the other index set does not have the same number of elements as this index set.
    /// This behavior is distinct from [Iterator::zip], which instead ignores elements in the longer
    /// collection.
    fn index_zip<I: IndexList>(self, other: I) -> IndexZip<Self, I>
    where
        Self: Sized,
    {
        IndexZip::new(self, other)
    }

    /// Zips this index list with another, but uniqueness is determined by the second list.
    ///
    /// This is identical to [`index_zip`](Self::index_zip), except that indices are considered
    /// unique if the *second* list is unique.
    ///
    /// # Panics
    ///
    /// Panics if the other index set does not have the same number of elements as this index set.
    /// This behavior is distinct from [`Iterator::zip`], which instead ignores elements in the longer
    /// collection.
    fn index_azip<I: IndexList>(self, other: I) -> IndexAZip<Self, I>
    where
        Self: Sized,
    {
        IndexAZip::new(self, other)
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

    /// Transposes the indices in this index list.
    ///
    /// TODO: Examples
    fn index_transpose(self) -> IndexTranspose<Self>
    where
        Self: Sized,
    {
        IndexTranspose(self)
    }

    /// Turns an index list into a list of unique indices, if possible.
    ///
    /// Checks that all indices are unique, and also determines their bounds.
    ///
    /// # Errors
    ///
    /// Returns an error if the indices are not unique.
    fn check_unique(self) -> Result<CheckedUnique<Self>, NonUniqueIndex>
    where
        Self: Sized,
        Self::Index: RecordIndex + Hash,
    {
        CheckedUnique::from_hashable_indices(self)
    }
}

unsafe impl<'a, I: IndexList> IndexList for &'a I {
    type Index = I::Index;

    const ALWAYS_BOUNDED: bool = I::ALWAYS_BOUNDED;

    unsafe fn get_index_unchecked(&self, loc: usize) -> Self::Index {
        I::get_index_unchecked(self, loc)
    }

    fn num_indices(&self) -> usize {
        I::num_indices(self)
    }

    fn bounds(&self) -> Option<Bounds<Self::Index>> {
        I::bounds(self)
    }
}

/// A finite list of *unique* indices.
///
/// TODO: Document requirements
pub unsafe trait UniqueIndexList: IndexList {}

unsafe impl<'a, I: UniqueIndexList> UniqueIndexList for &'a I {}

/// An access object that has been narrowed to a subset of its indices.
///
/// This is the result type for
/// [`narrow_access_to_indices`](crate::unique::narrow_access_to_indices).
///
/// TODO: Move this struct into its own file to ensure more control over its field
/// (incorrect access from other pieces of the code could lead to soundness issues)
///
/// TODO: Provide method like `.ensure_in_bounds()` that ensures that
/// all bounds checks are statically eliminated (currently we rely on
/// compiler optimizations to eliminate those)
#[derive(Debug)]
pub struct NarrowedAccess<'a, Indices, Access> {
    indices: &'a Indices,
    access: Access,
    verified_in_bounds: bool,
}

impl<'a, Indices, Access> NarrowedAccess<'a, Indices, Access>
where
    Indices: IndexList,
    Indices::Index: RecordIndex,
    Access: ParAccess<Indices::Index>,
{
    pub(crate) fn try_new(indices: &'a Indices, access: Access) -> Result<Self, OutOfBounds> {
        if let Some(index_bounds) = indices.bounds() {
            if access.bounds().contains_bounds(&index_bounds) {
                Ok(Self {
                    indices,
                    access,
                    verified_in_bounds: true,
                })
            } else {
                Err(OutOfBounds)
            }
        } else {
            assert!(
                !Indices::ALWAYS_BOUNDED,
                "IndexList {} claims that it is ALWAYS_BOUNDED, but no bounds were returned",
                type_name::<Indices>()
            );

            // In this case, bounds are not available, so we can not say
            // whether all indices in bounds. This means that we might panic
            // upon access instead
            Ok(Self {
                indices,
                access,
                verified_in_bounds: false,
            })
        }
    }
}

unsafe impl<'a, Indices, Access> ParAccess<usize> for NarrowedAccess<'a, Indices, Access>
where
    Indices: UniqueIndexList,
    Indices::Index: RecordIndex,
    Access: ParAccess<Indices::Index>,
{
    type Record = Access::Record;

    #[inline(always)]
    unsafe fn clone_access(&self) -> Self {
        Self {
            indices: self.indices,
            access: unsafe { self.access.clone_access() },
            verified_in_bounds: self.verified_in_bounds,
        }
    }

    fn bounds(&self) -> Bounds<usize> {
        Bounds {
            offset: 0,
            extent: self.indices.num_indices(),
        }
    }

    #[inline(always)]
    unsafe fn get_unsync_unchecked(&self, loc: usize) -> Self::Record {
        // SAFETY: Since this is an unchecked method, we can always directly try to obtain
        // the index at the requested location in the index list
        let index = self.indices.get_index_unchecked(loc);
        if Indices::ALWAYS_BOUNDED {
            // This branch hopefully allows us to completely eliminate all branches
            // whenever we're able to statically prove that bounds checking is unnecessary
            debug_assert!(self.verified_in_bounds);
            // SAFETY: This is sound due to the fact that
            // we've checked that all indices are in bounds when constructing Self
            self.access.get_unsync_unchecked(index)
        } else if self.verified_in_bounds {
            // SAFETY: This is sound due to the fact that
            // we've checked that all indices are in bounds when constructing Self
            self.access.get_unsync_unchecked(index)
        } else {
            // We cannot prove that all indices are in bounds, so we need
            // to use bounds checking to avoid possible unsoundness
            self.access.get_unsync(index)
        }
    }
}

unsafe impl<'a, Indices, Access> LinearParAccess for NarrowedAccess<'a, Indices, Access>
where
    Indices: UniqueIndexList,
    Indices::Index: RecordIndex,
    Access: ParAccess<Indices::Index>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.indices.num_indices()
    }
}

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
        *<[I]>::get_unchecked(self.as_slice(), loc)
    }

    fn num_indices(&self) -> usize {
        self.len()
    }

    fn bounds(&self) -> Option<Bounds<Self::Index>> {
        None
    }
}
