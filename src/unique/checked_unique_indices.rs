use crate::unique::unique_indices::IndexList;
use crate::unique::UniqueIndexList;
use crate::RecordIndex;
use std::collections::HashSet;
use std::hash::Hash;

/// A list of indices that are checked to be unique.
pub struct CheckedIndexList<Idx> {
    // TODO: Generalize to something like IndexContainer that supports e.g. Vec<Idx>, &[Idx]
    indices: Vec<Idx>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonUniqueIndex;

impl<Idx: RecordIndex> CheckedIndexList<Idx> {
    pub fn from_hashable_indices(indices: Vec<Idx>) -> Result<Self, NonUniqueIndex>
    where
        Idx: Hash,
    {
        // TODO: Implement re-usable "checker" for re-using allocations
        let hashed: HashSet<Idx> = indices.iter().copied().collect();
        if hashed.len() == indices.len() {
            Ok(Self { indices })
        } else {
            Err(NonUniqueIndex)
        }
    }
}

unsafe impl<Idx> IndexList for CheckedIndexList<Idx>
where
    Idx: RecordIndex + Send + Sync,
{
    type Index = Idx;

    unsafe fn get_unchecked(&self, i: usize) -> Self::Index {
        *self.indices.get_unchecked(i)
    }

    fn num_indices(&self) -> usize {
        self.indices.len()
    }
}

unsafe impl<Idx> UniqueIndexList for CheckedIndexList<Idx> where Idx: RecordIndex + Send + Sync {}
