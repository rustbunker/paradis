use crate::unique::unique_indices::IndexList;
use crate::unique::UniqueIndexList;

/// The result of zipping two *equal-length* index sets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexZip<A, B>(A, B);

impl<A, B> IndexZip<A, B>
where
    A: UniqueIndexList,
    B: UniqueIndexList,
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
    A: UniqueIndexList,
    B: UniqueIndexList,
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
    A: UniqueIndexList,
    B: UniqueIndexList,
{
}
