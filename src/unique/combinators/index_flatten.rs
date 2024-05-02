use paradis_core::Bounds;
use crate::internal::Sealed;
use crate::unique::unique_indices::IndexList;
use crate::unique::UniqueIndexList;

/// An index combinator that flattens nested tuples.
///
/// See [IndexList::index_flatten](crate::unique::IndexList::index_flatten).
pub struct IndexFlatten<SourceIndices>(pub(crate) SourceIndices);

unsafe impl<SourceIndices> IndexList for IndexFlatten<SourceIndices>
where
    SourceIndices: IndexList,
    SourceIndices::Index: Flatten,
    <SourceIndices::Index as Flatten>::Flattened: Copy,
{
    type Index = <SourceIndices::Index as Flatten>::Flattened;
    const ALWAYS_BOUNDED: bool = SourceIndices::ALWAYS_BOUNDED;

    unsafe fn get_unchecked(&self, loc: usize) -> Self::Index {
        self.0.get_unchecked(loc).flatten()
    }

    fn num_indices(&self) -> usize {
        self.0.num_indices()
    }

    fn bounds(&self) -> Option<Bounds<Self::Index>> {
        self.0.bounds()
            .map(|bounds| Bounds {
                offset: bounds.offset.flatten(),
                extent: bounds.extent.flatten()
            })
    }
}

unsafe impl<SourceIndices> UniqueIndexList for IndexFlatten<SourceIndices>
where
    SourceIndices: UniqueIndexList,
    SourceIndices::Index: Flatten,
    <SourceIndices::Index as Flatten>::Flattened: Copy,
{
}

/// Concatenate tuples.
///
/// This is part of the machinery that drives
/// [IndexList::index_flatten][crate::unique::IndexList::index_flatten].
pub trait Concatenate<T>: Sealed {
    type Concatenated;

    fn concatenate(self, other: T) -> Self::Concatenated;
}

/// The result of concatenating two types.
pub type Concatenated<A, B> = <A as Concatenate<B>>::Concatenated;

/// Flatten nested tuples.
///
/// This is part of the machinery that drives
/// [IndexList::index_flatten][crate::unique::IndexList::index_flatten].
pub trait Flatten: Sealed {
    type Flattened;

    fn flatten(self) -> Self::Flattened;
}

/// Retrieve the type associated with either a single variable identifier or a tuple of
/// variable identifiers.
macro_rules! var_type {
    // Replace a single variable like `a` with `usize`
    ($single_name:ident) => { usize };
    // Replace a tuple like `(a, b)` with `(usize, usize)`
    (($($tuple_elem:ident),+)) => { ($(var_type!(to_type($tuple_elem))),+) };
    // This is just a helper to replace an ident with `usize` (which can be used as a type)
    (to_type($var:ident)) => { usize }
}

/// Impl Concatenate for various tuple combinations, such as (usize, usize) + (usize, usize)
macro_rules! impl_usize_concatenate {
    ($left_var:tt + $right_var:tt => $result_var:tt) => {
        impl Concatenate<var_type!($right_var)> for var_type!($left_var) {
            type Concatenated = var_type!($result_var);

            fn concatenate(self, $right_var: var_type!($right_var)) -> Self::Concatenated {
                let $left_var = self;
                $result_var
            }
        }
    };
}

impl_usize_concatenate!(a + b => (a, b));
impl_usize_concatenate!(a + (b, c) => (a, b, c));
impl_usize_concatenate!(a + (b, c, d) => (a, b, c, d));
impl_usize_concatenate!(a + (b, c, d, e) => (a, b, c, d, e));

impl_usize_concatenate!((a, b) + c => (a, b, c));
impl_usize_concatenate!((a, b) + (c, d) => (a, b, c, d));
impl_usize_concatenate!((a, b) + (c, d, e) => (a, b, c, d, e));

impl_usize_concatenate!((a, b, c) + d => (a, b, c, d));
impl_usize_concatenate!((a, b, c) + (d, e) => (a, b, c, d, e));

impl_usize_concatenate!((a, b, c, d) + e => (a, b, c, d, e));

impl Flatten for usize {
    type Flattened = Self;

    fn flatten(self) -> Self {
        self
    }
}

impl<A: Flatten> Flatten for (A,) {
    type Flattened = (A::Flattened,);

    fn flatten(self) -> Self::Flattened {
        (self.0.flatten(),)
    }
}

impl<A: Flatten, B: Flatten> Flatten for (A, B)
where
    A::Flattened: Concatenate<B::Flattened>,
{
    type Flattened = Concatenated<A::Flattened, B::Flattened>;

    fn flatten(self) -> Self::Flattened {
        self.0.flatten().concatenate(self.1.flatten())
    }
}

impl<A: Flatten, B: Flatten, C: Flatten> Flatten for (A, B, C)
where
    (A, B): Flatten,
    <(A, B) as Flatten>::Flattened: Concatenate<C::Flattened>,
{
    type Flattened = Concatenated<<(A, B) as Flatten>::Flattened, C::Flattened>;

    fn flatten(self) -> Self::Flattened {
        (self.0, self.1).flatten().concatenate(self.2.flatten())
    }
}

impl<A: Flatten, B: Flatten, C: Flatten, D: Flatten> Flatten for (A, B, C, D)
where
    (A, B, C): Flatten,
    <(A, B, C) as Flatten>::Flattened: Concatenate<D::Flattened>,
{
    type Flattened = Concatenated<<(A, B, C) as Flatten>::Flattened, D::Flattened>;

    fn flatten(self) -> Self::Flattened {
        (self.0, self.1, self.2)
            .flatten()
            .concatenate(self.3.flatten())
    }
}

impl<A: Flatten, B: Flatten, C: Flatten, D: Flatten, E: Flatten> Flatten for (A, B, C, D, E)
where
    (A, B, C, D): Flatten,
    <(A, B, C, D) as Flatten>::Flattened: Concatenate<E::Flattened>,
{
    type Flattened = Concatenated<<(A, B, C, D) as Flatten>::Flattened, E::Flattened>;

    fn flatten(self) -> Self::Flattened {
        (self.0, self.1, self.2, self.3)
            .flatten()
            .concatenate(self.4.flatten())
    }
}
