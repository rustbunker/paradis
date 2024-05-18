use crate::index::{IndexList, UniqueIndexList};
use paradis_core::Bounds;

/// A list of indices that are unique by assumption.
///
/// See [assume_unique](crate::index::IndexList::assume_unique) for more information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssumedUnique<Indices> {
    indices: Indices,
}

impl<Indices: IndexList> AssumedUnique<Indices> {
    pub(crate) unsafe fn assume_unique(indices: Indices) -> Self {
        Self { indices }
    }
}

unsafe impl<Indices: IndexList> IndexList for AssumedUnique<Indices> {
    type Index = ();
    const ALWAYS_BOUNDED: bool = false;

    unsafe fn get_index_unchecked(&self, loc: usize) -> Self::Index {
        self.indices.get_index_unchecked(loc)
    }

    fn num_indices(&self) -> usize {
        self.indices.num_indices()
    }

    fn bounds(&self) -> Option<Bounds<Self::Index>> {
        self.indices.bounds()
    }

    fn get_index(&self, loc: usize) -> Self::Index {
        self.indices.get_index(loc)
    }
}

/// This is sound because the only way to construct this type is to call an unsafe function
/// where the user promises that the indices are truly unique.
unsafe impl<Indices: IndexList> UniqueIndexList for AssumedUnique<Indices> {}
