use crate::unique::unique_indices::IndexList;
use crate::unique::UniqueIndexList;
use crate::RecordIndex;
use std::collections::HashSet;
use std::hash::Hash;
use paradis_core::Bounds;

/// A list of indices that are checked to be unique.
pub struct CheckedIndexList<Idx> {
    // TODO: Generalize to something like IndexContainer that supports e.g. Vec<Idx>, &[Idx]
    indices: Vec<Idx>,
    bounds: Bounds<Idx>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonUniqueIndex;

impl<Idx: RecordIndex> CheckedIndexList<Idx> {
    pub fn from_hashable_indices(indices: Vec<Idx>) -> Result<Self, NonUniqueIndex>
    where
        Idx: Hash,
    {
        // TODO: Implement re-usable "checker" for re-using allocations

        if indices.is_empty() {
            return Ok(Self {
                indices,
                bounds: Idx::empty_bounds()
            });
        }

        let mut set = HashSet::with_capacity(indices.len());
        let (head, tail) = indices.split_first().unwrap();
        let mut bounds = Bounds::bounds_for_index(*head);

        for idx in tail.iter().copied() {
            bounds.enclose_index(idx);
            if !set.insert(idx) {
                return Err(NonUniqueIndex);
            }
        }

        Ok(Self {
            indices,
            bounds,
        })
    }
}

unsafe impl<Idx> IndexList for CheckedIndexList<Idx>
where
    Idx: RecordIndex + Send + Sync,
{
    type Index = Idx;

    const ALWAYS_BOUNDED: bool = true;

    unsafe fn get_unchecked(&self, i: usize) -> Self::Index {
        *self.indices.get_unchecked(i)
    }

    fn num_indices(&self) -> usize {
        self.indices.len()
    }

    fn bounds(&self) -> Option<Bounds<Self::Index>> {
        Some(self.bounds)
    }
}

unsafe impl<Idx> UniqueIndexList for CheckedIndexList<Idx> where Idx: RecordIndex + Send + Sync {}
