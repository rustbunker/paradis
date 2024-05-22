//! Construction of index lists, and facilities for access narrowing.
use crate::error::OutOfBounds;
use paradis_core::{IntoParAccess, RecordIndex};

pub mod combinators;
pub mod patterns;

mod assumed_unique;
mod checked_unique;
mod index_list;
mod index_list_impl_std;
mod narrowed_access;

pub use assumed_unique::AssumedUnique;
pub use checked_unique::CheckedUnique;
pub use index_list::{IndexList, UniqueIndexList};
pub use narrowed_access::NarrowedAccess;

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

/// Collects an index list into the desired collection.
///
/// This is a convenience feature intended mainly for debugging and tests.
///
/// # Examples
///
/// ```
/// use paradis::index::collect_indices;
///
/// let indices: Vec<_> = collect_indices(1 .. 5);
/// assert_eq!(indices, vec![1, 2, 3, 4]);
/// ```
pub fn collect_indices<Collection, Indices>(indices: Indices) -> Collection
where
    Collection: FromIterator<Indices::Index>,
    Indices: IndexList,
{
    (0..indices.num_indices())
        .map(|loc| indices.get_index(loc))
        .collect()
}
