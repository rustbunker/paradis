use crate::unique::UniqueIndices;
use crate::{IndexFrom, RecordIndex};
use std::marker::PhantomData;

/// Cast indices in a source index list to the target index type.
///
/// See [UniqueIndices::index_cast](crate::unique::UniqueIndices::index_cast).
#[derive(Debug)]
pub struct IndexCast<Indices, TargetIndex> {
    pub(crate) source_indices: Indices,
    pub(crate) marker: PhantomData<TargetIndex>,
}

unsafe impl<Indices, TargetIndex> UniqueIndices for IndexCast<Indices, TargetIndex>
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
