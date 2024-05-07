use crate::error::OutOfBounds;
use crate::index::{IndexList, UniqueIndexList};
use crate::{BoundedParAccess, Bounds, LinearParAccess, RecordIndex};
use paradis_core::ParAccess;
use std::any::type_name;

/// An access object that has been narrowed to a subset of its indices.
///
/// This is the result type for
/// [`narrow_access_to_indices`](crate::index::narrow_access_to_indices).
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
    Access: BoundedParAccess<Indices::Index>,
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
    Access: BoundedParAccess<Indices::Index>,
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

    #[inline(always)]
    unsafe fn get_unsync_unchecked(&self, loc: usize) -> Self::Record {
        // SAFETY: Since this is an unchecked method, we can always directly try to obtain
        // the index at the requested location in the index list
        let index = unsafe { self.indices.get_index_unchecked(loc) };
        if Indices::ALWAYS_BOUNDED {
            // This branch hopefully allows us to completely eliminate all branches
            // whenever we're able to statically prove that bounds checking is unnecessary
            debug_assert!(self.verified_in_bounds);
            // SAFETY: This is sound due to the fact that
            // we've checked that all indices are in bounds when constructing Self
            unsafe { self.access.get_unsync_unchecked(index) }
        } else if self.verified_in_bounds {
            // SAFETY: This is sound due to the fact that
            // we've checked that all indices are in bounds when constructing Self
            unsafe { self.access.get_unsync_unchecked(index) }
        } else {
            // We cannot prove that all indices are in bounds, so we need
            // to use bounds checking to avoid possible unsoundness
            unsafe { self.access.get_unsync(index) }
        }
    }
}

unsafe impl<'a, Indices, Access> BoundedParAccess<usize> for NarrowedAccess<'a, Indices, Access>
where
    Indices: UniqueIndexList,
    Indices::Index: RecordIndex,
    Access: BoundedParAccess<Indices::Index>,
{
    fn bounds(&self) -> Bounds<usize> {
        Bounds {
            offset: 0,
            extent: self.indices.num_indices(),
        }
    }
}

unsafe impl<'a, Indices, Access> LinearParAccess for NarrowedAccess<'a, Indices, Access>
where
    Indices: UniqueIndexList,
    Indices::Index: RecordIndex,
    Access: BoundedParAccess<Indices::Index>,
{
    #[inline(always)]
    fn collection_len(&self) -> usize {
        self.indices.num_indices()
    }
}
