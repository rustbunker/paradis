use crate::unique::UniqueIndices;

/// A Cartesian product of index sets.
///
/// TODO: Example, document row-major behavior etc.
/// TODO: Also provide `IndexRProduct` for alternative column-major ordering
///       that's probably a bad name, since it's not "reverse" as "R" might suggest.
///       IndexCProduct? Not sure..
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexProduct<A, B>(pub A, pub B);

unsafe impl<A, B> UniqueIndices for IndexProduct<A, B>
where
    A: UniqueIndices,
    B: UniqueIndices,
{
    type Index = (A::Index, B::Index);

    unsafe fn get_unchecked(&self, loc: usize) -> Self::Index {
        let m = self.1.num_indices();
        let i = loc / m;
        let j = loc % m;
        (self.0.get_unchecked(i), self.1.get_unchecked(j))
    }

    fn num_indices(&self) -> usize {
        self.0.num_indices() * self.1.num_indices()
    }
}

#[cfg(test)]
mod tests {
    use crate::unique::combinators::IndexProduct;
    use crate::unique::UniqueIndices;

    #[test]
    fn index_product_basic_tests() {
        let product = IndexProduct(0..3, 1..4);
        assert_eq!(product.num_indices(), 9);

        assert_eq!(product.get(0), (0, 1));
        assert_eq!(product.get(1), (0, 2));
        assert_eq!(product.get(2), (0, 3));
        assert_eq!(product.get(3), (1, 1));
        assert_eq!(product.get(4), (1, 2));
        assert_eq!(product.get(5), (1, 3));
        assert_eq!(product.get(6), (2, 1));
        assert_eq!(product.get(7), (2, 2));
        assert_eq!(product.get(8), (2, 3));
    }
}
