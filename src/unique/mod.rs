//! Parallel iteration of collections indexed by unique indices.
use paradis_core::IntoUnsyncAccess;

mod checked_unique_indices;
mod index_from;
mod unique_index;
mod unique_indices;

pub use checked_unique_indices::CheckedUniqueIndices;
pub use index_from::IndexFrom;
pub use unique_index::UniqueIndex;
pub use unique_indices::{UniqueIndices, UniqueIndicesConvertedType, UniqueIndicesWithAccess};

pub fn compose_access_with_indices<IntoAccess, Indices>(
    access: IntoAccess,
    indices: &Indices,
) -> UniqueIndicesWithAccess<'_, Indices, IntoAccess::Access>
where
    // TODO: Is the Sized bound necessary? Do we want it? The alternative is to sprinkle
    // ?Sized around, but I'm not sure whether we want that either. Gotta figure out...
    Indices: UniqueIndices + Sized,
    IntoAccess: IntoUnsyncAccess<Indices::Index>,
{
    UniqueIndicesWithAccess {
        indices,
        access: access.into_unsync_access(),
    }
}
