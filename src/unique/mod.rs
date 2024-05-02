//! Parallel iteration of collections indexed by unique indices.
use paradis_core::{IntoParAccess, RecordIndex};

pub mod combinators;

mod checked_unique_indices;
mod repeat;
mod unique_indices;

pub use checked_unique_indices::CheckedIndexList;
pub use repeat::Repeat;
pub use unique_indices::{IndexList, IndexedAccess, UniqueIndexList};

/// An error indicating that indices were out of bounds.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct OutOfBounds;

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
