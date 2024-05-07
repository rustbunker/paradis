use crate::index::{IndexList, UniqueIndexList};
use paradis_core::Bounds;

/// The result of zipping two *equal-length* index sets.
///
/// See [IndexList::index_zip](crate::index::IndexList::index_zip) for more
/// information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexZip<A, B>(A, B);

impl<A, B> IndexZip<A, B>
where
    A: IndexList,
    B: IndexList,
{
    /// Zip two index lists.
    ///
    /// # Panics
    ///
    /// Panics if the two index lists do not have the same number of indices.
    pub fn new(a: A, b: B) -> Self {
        assert_eq!(
            a.num_indices(),
            b.num_indices(),
            "IndexZip requires the number of indices to be equal in the zipped index sets"
        );
        Self(a, b)
    }
}

// TODO: Test this impl
unsafe impl<A, B> IndexList for IndexZip<A, B>
where
    A: IndexList,
    B: IndexList,
{
    type Index = (A::Index, B::Index);

    const ALWAYS_BOUNDED: bool = A::ALWAYS_BOUNDED && B::ALWAYS_BOUNDED;

    unsafe fn get_index_unchecked(&self, loc: usize) -> Self::Index {
        unsafe {
            (
                self.0.get_index_unchecked(loc),
                self.1.get_index_unchecked(loc),
            )
        }
    }

    fn num_indices(&self) -> usize {
        debug_assert_eq!(self.0.num_indices(), self.1.num_indices());
        self.0.num_indices()
    }

    fn bounds(&self) -> Option<Bounds<Self::Index>> {
        self.0.bounds().zip(self.1.bounds()).map(|(a, b)| a.zip(b))
    }
}

unsafe impl<A, B> UniqueIndexList for IndexZip<A, B>
where
    // TODO: IndexZip would satisfy UniqueIndexList if *either* A or B
    // has unique indices. However, we are unable to
    // express this in Rust's type system as this would require specialization
    // or at least lattice impls
    // We still need a way for users to work around this though
    A: UniqueIndexList,
    B: IndexList,
{
}

/// [`IndexZip`] where uniqueness is determined by the *second* argument.
///
/// See [IndexList::index_azip](crate::index::IndexList::index_azip) for more
/// information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexAZip<A, B>(A, B);

impl<A, B> IndexAZip<A, B>
where
    A: IndexList,
    B: IndexList,
{
    /// Zip two index lists.
    ///
    /// # Panics
    ///
    /// Panics if the two index lists do not have the same number of indices.
    pub fn new(a: A, b: B) -> Self {
        assert_eq!(
            a.num_indices(),
            b.num_indices(),
            "IndexAZip requires the number of indices to be equal in the zipped index sets"
        );
        Self(a, b)
    }
}

// TODO: Test this impl
unsafe impl<A, B> IndexList for IndexAZip<A, B>
where
    A: IndexList,
    B: IndexList,
{
    type Index = (A::Index, B::Index);

    const ALWAYS_BOUNDED: bool = A::ALWAYS_BOUNDED && B::ALWAYS_BOUNDED;

    unsafe fn get_index_unchecked(&self, loc: usize) -> Self::Index {
        unsafe {
            (
                self.0.get_index_unchecked(loc),
                self.1.get_index_unchecked(loc),
            )
        }
    }

    fn num_indices(&self) -> usize {
        debug_assert_eq!(self.0.num_indices(), self.1.num_indices());
        self.0.num_indices()
    }

    fn bounds(&self) -> Option<Bounds<Self::Index>> {
        self.0.bounds().zip(self.1.bounds()).map(|(a, b)| a.zip(b))
    }
}

unsafe impl<A, B> UniqueIndexList for IndexAZip<A, B>
where
    // TODO: IndexZip would satisfy UniqueIndexList if *either* A or B
    // has unique indices. However, we are unable to
    // express this in Rust's type system as this would require specialization
    // or at least lattice impls
    // We still need a way for users to work around this though
    A: IndexList,
    B: UniqueIndexList,
{
}
