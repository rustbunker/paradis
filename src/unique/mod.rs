//! Parallel iteration of collections indexed by unique indices.
use paradis_core::IntoParAccess;

pub mod combinators;

mod checked_unique_indices;
mod unique_indices;

pub use checked_unique_indices::CheckedIndexList;
pub use unique_indices::{IndexList, UniqueIndexList, UniqueIndexListWithAccess};

pub fn compose_access_with_indices<IntoAccess, Indices>(
    access: IntoAccess,
    indices: &Indices,
) -> UniqueIndexListWithAccess<'_, Indices, IntoAccess::Access>
where
    // TODO: Is the Sized bound necessary? Do we want it? The alternative is to sprinkle
    // ?Sized around, but I'm not sure whether we want that either. Gotta figure out...
    Indices: UniqueIndexList + Sized,
    IntoAccess: IntoParAccess<Indices::Index>,
{
    UniqueIndexListWithAccess {
        indices,
        access: access.into_par_access(),
    }
}
