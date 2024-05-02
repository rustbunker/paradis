use crate::unique::unique_indices::IndexList;
use crate::unique::UniqueIndexList;
use crate::{IndexFrom, RecordIndex};
use std::marker::PhantomData;
use paradis_core::Bounds;

/// Cast indices in a source index list to the target index type.
///
/// See [IndexList::index_cast](crate::unique::IndexList::index_cast).
#[derive(Debug)]
pub struct IndexCast<Indices, TargetIndex> {
    pub(crate) source_indices: Indices,
    pub(crate) marker: PhantomData<TargetIndex>,
}

unsafe impl<Indices, TargetIndex> IndexList for IndexCast<Indices, TargetIndex>
where
    Indices: IndexList,
    TargetIndex: Copy + RecordIndex + IndexFrom<Indices::Index>,
{
    type Index = TargetIndex;
    const ALWAYS_BOUNDED: bool = Indices::ALWAYS_BOUNDED;

    unsafe fn get_unchecked(&self, i: usize) -> Self::Index {
        // TODO: Cannot use TryFrom since it's not guaranteed to be
        let source_idx = self.source_indices.get_unchecked(i);
        TargetIndex::index_from(source_idx)
    }

    fn num_indices(&self) -> usize {
        self.source_indices.num_indices()
    }

    fn bounds(&self) -> Option<Bounds<Self::Index>> {
        self.source_indices
            .bounds()
            .map(|bounds| Bounds {
                offset: TargetIndex::index_from(bounds.offset),
                extent: TargetIndex::index_from(bounds.extent),
            })

    }
}

unsafe impl<Indices, TargetIndex> UniqueIndexList for IndexCast<Indices, TargetIndex>
where
    Indices: UniqueIndexList,
    TargetIndex: Copy + RecordIndex + IndexFrom<Indices::Index>,
{
}
