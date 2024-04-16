//! Parallel iteration of collections indexed by unique indices.
use paradis_core::IntoParAccess;

pub mod combinators;

mod checked_unique_indices;
mod repeat;
mod unique_indices;

pub use checked_unique_indices::CheckedIndexList;
pub use repeat::Repeat;
pub use unique_indices::{IndexList, IndexedAccess, UniqueIndexList};

/// Narrows an access object to a subset of its index set.
pub fn narrow_access_to_indices<IntoAccess, Indices>(
    access: IntoAccess,
    indices: &Indices,
) -> IndexedAccess<'_, Indices, IntoAccess::Access>
where
    // TODO: Is the Sized bound necessary? Do we want it? The alternative is to sprinkle
    // ?Sized around, but I'm not sure whether we want that either. Gotta figure out...
    Indices: UniqueIndexList + Sized,
    IntoAccess: IntoParAccess<Indices::Index>,
{
    IndexedAccess {
        indices,
        access: access.into_par_access(),
    }
}
