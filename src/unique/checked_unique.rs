use crate::unique::unique_indices::IndexList;
use crate::unique::UniqueIndexList;
use crate::RecordIndex;
use paradis_core::Bounds;
use std::collections::HashSet;
use std::hash::Hash;

/// A list of indices that are checked to be unique.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedUnique<Indices: IndexList> {
    indices: Indices,
    bounds: Bounds<Indices::Index>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonUniqueIndex;

impl<Indices> CheckedUnique<Indices>
where
    Indices: IndexList,
    Indices::Index: RecordIndex,
{
    pub fn from_hashable_indices(indices: Indices) -> Result<Self, NonUniqueIndex>
    where
        Indices::Index: Hash,
    {
        let n = indices.num_indices();
        if n == 0 {
            return Ok(Self {
                indices,
                bounds: Indices::Index::empty_bounds(),
            });
        }

        let mut bounds = Bounds::bounds_for_index(indices.get(0));
        // TODO: Use faster hash? ahash?
        let mut set = HashSet::with_capacity(n);
        for loc in 0..n {
            let idx = indices.get(loc);
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

    unsafe fn get_unchecked(&self, i: usize) -> Self::Index {
        self.indices.get_unchecked(i)
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
