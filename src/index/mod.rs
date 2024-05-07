//! Construction of index lists, and facilities for access narrowing.
use paradis_core::{IntoParAccess, RecordIndex};

pub mod combinators;

mod checked_unique;
mod index_list;
mod index_list_impl_std;
mod narrowed_access;
mod repeat;

use crate::error::OutOfBounds;
pub use checked_unique::CheckedUnique;
pub use index_list::{IndexList, UniqueIndexList};
pub use narrowed_access::NarrowedAccess;
pub use repeat::Repeat;

/// Narrows an access object to a subset of its index set.
///
/// The indices must be unique, which is ensured through the [`UniqueIndexList`] trait.
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
) -> Result<NarrowedAccess<'_, Indices, IntoAccess::Access>, OutOfBounds>
where
    Indices: UniqueIndexList,
    Indices::Index: RecordIndex,
    IntoAccess: IntoParAccess<Indices::Index>,
{
    NarrowedAccess::try_new(indices, access.into_par_access())
}
