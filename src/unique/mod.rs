//! Parallel iteration of collections indexed by unique indices.
use paradis_core::{IntoParAccess, RecordIndex};

pub mod combinators;

mod checked_unique;
mod index_list;
mod repeat;

use crate::error::OutOfBounds;
pub use checked_unique::CheckedUnique;
pub use index_list::{IndexList, IndexedAccess, UniqueIndexList};
pub use repeat::Repeat;

/// Narrows an access object to a subset of its index set.
///
/// # Errors
///
/// Returns an [`OutOfBounds`] error if the index bounds are not
/// contained in the bounds of the collection.
///
/// # Panics
///
/// If indices are not bounded, later accesses made through the
/// returned access object may panic.
pub fn narrow_access_to_indices<IntoAccess, Indices>(
    access: IntoAccess,
    indices: &Indices,
) -> Result<IndexedAccess<'_, Indices, IntoAccess::Access>, OutOfBounds>
where
    // TODO: Is the Sized bound necessary? Do we want it? The alternative is to sprinkle
    // ?Sized around, but I'm not sure whether we want that either. Gotta figure out...
    Indices: UniqueIndexList + Sized,
    Indices::Index: RecordIndex,
    IntoAccess: IntoParAccess<Indices::Index>,
{
    IndexedAccess::try_new(indices, access.into_par_access())
}
