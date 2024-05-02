use crate::unique::{IndexList, UniqueIndexList};
use paradis_core::Bounds;

/// An index combinator that transposed indices.
///
/// See [IndexList::index_transpose](crate::unique::IndexList::index_transpose).
#[derive(Debug)]
pub struct IndexTranspose<I>(pub I);

unsafe impl<I> IndexList for IndexTranspose<I>
where
    I: IndexList,
    I::Index: Transpose,
    <I::Index as Transpose>::Transposed: Copy,
{
    type Index = <I::Index as Transpose>::Transposed;

    const ALWAYS_BOUNDED: bool = I::ALWAYS_BOUNDED;

    unsafe fn get_unchecked(&self, loc: usize) -> Self::Index {
        let source_idx = self.0.get_unchecked(loc);
        source_idx.transpose()
    }

    fn num_indices(&self) -> usize {
        self.0.num_indices()
    }

    fn bounds(&self) -> Option<Bounds<Self::Index>> {
        self.0.bounds().map(|bounds| Bounds {
            offset: bounds.offset.transpose(),
            extent: bounds.extent.transpose(),
        })
    }
}

unsafe impl<I: UniqueIndexList> UniqueIndexList for IndexTranspose<I>
where
    I: IndexList,
    I::Index: Transpose,
    <I::Index as Transpose>::Transposed: Copy,
{
}

/// Transpose an index, i.e. reversing the order in a tuple.
pub trait Transpose {
    type Transposed;

    fn transpose(self) -> Self::Transposed;
}

impl Transpose for usize {
    type Transposed = Self;

    fn transpose(self) -> usize {
        self
    }
}

macro_rules! impl_tuple_transpose {
    (($($input_var:tt),*): ($($input_ty:tt),*) => ($($output_var:tt),*): ($($output:ty),*)) => {
        impl<$($input_ty),*> Transpose for ($($input_ty),*) {
            type Transposed = ($($output),*);

            fn transpose(self) -> Self::Transposed {
                let ($($input_var),*) = self;
                ($($output_var),*)
            }
        }
    };
}

impl_tuple_transpose!((a, b): (A, B) => (b, a): (B, A));
impl_tuple_transpose!((a, b, c): (A, B, C) => (c, b, a): (C, B, A));
impl_tuple_transpose!((a, b, c, d): (A, B, C, D) => (d, c, b, a): (D, C, B, A));
impl_tuple_transpose!((a, b, c, d, e): (A, B, C, D, E) => (e, d, c, b, a): (E, D, C, B, A));
