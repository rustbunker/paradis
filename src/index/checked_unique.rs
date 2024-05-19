use crate::error::NonUniqueIndex;
use crate::index::{IndexList, UniqueIndexList};
use crate::RecordIndex;
use paradis_core::Bounds;
use std::collections::HashSet;

/// A list of indices that are checked to be unique.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedUnique<Indices: IndexList> {
    indices: Indices,
    bounds: Bounds<Indices::Index>,
}

impl<Indices: IndexList> CheckedUnique<Indices> {
    /// Obtain a reference to the underlying index list.
    pub fn get_inner(&self) -> &Indices {
        &self.indices
    }

    /// Recover the underlying index list.
    pub fn into_inner(self) -> Indices {
        self.indices
    }
}

impl<Indices> CheckedUnique<Indices>
where
    Indices: IndexList,
    Indices::Index: RecordIndex,
{
    /// Check that the provided indices are unique.
    ///
    /// On success, wrap this object in [`CheckedUnique`]. The bounds of the index list
    /// are computed at the same time.
    ///
    /// # Errors
    ///
    /// An error is returned if the indices are not unique.
    pub fn from_hashable_indices(indices: Indices) -> Result<Self, NonUniqueIndex> {
        let n = indices.num_indices();
        if n == 0 {
            return Ok(Self {
                indices,
                bounds: Indices::Index::empty_bounds(),
            });
        }

        let mut bounds = Bounds::bounds_for_index(indices.get_index(0));
        // TODO: Use faster hash? ahash?
        let mut set = HashSet::with_capacity(n);
        for loc in 0..n {
            let idx = indices.get_index(loc);
            bounds.enclose_index(idx);
            if !set.insert(idx) {
                return Err(NonUniqueIndex);
            }
        }

        Ok(Self { indices, bounds })
    }
}

unsafe impl<Indices> IndexList for CheckedUnique<Indices>
where
    Indices: IndexList,
    Indices::Index: RecordIndex,
{
    type Index = Indices::Index;

    const ALWAYS_BOUNDED: bool = true;

    unsafe fn get_index_unchecked(&self, i: usize) -> Self::Index {
        unsafe { self.indices.get_index_unchecked(i) }
    }

    fn num_indices(&self) -> usize {
        self.indices.num_indices()
    }

    fn bounds(&self) -> Option<Bounds<Self::Index>> {
        Some(self.bounds)
    }
}

unsafe impl<Indices> UniqueIndexList for CheckedUnique<Indices>
where
    Indices: IndexList,
    Indices::Index: RecordIndex,
{
}
