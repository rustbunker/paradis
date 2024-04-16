use crate::unique::unique_indices::IndexList;
use crate::unique::UniqueIndexList;

/// The result of zipping two *equal-length* index sets.
///
/// See [UniqueIndexList::index_zip](crate::unique::UniqueIndexList::index_zip) for more
/// information.
///
/// TODO: Currently we require that A: UniqueIndexList and B: IndexList, but it should ideally
/// also be possible to use it the other way around. One option could be to provide
/// an associated const ALL_UNIQUE in IndexList, lower the constriant to A, B: IndexList
/// and that check at "runtime" that A and/or B.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexZip<A, B>(A, B);

impl<A, B> IndexZip<A, B>
where
    A: IndexList,
    B: IndexList,
{
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

    unsafe fn get_unchecked(&self, loc: usize) -> Self::Index {
        (self.0.get_unchecked(loc), self.1.get_unchecked(loc))
    }

    fn num_indices(&self) -> usize {
        debug_assert_eq!(self.0.num_indices(), self.1.num_indices());
        self.0.num_indices()
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
