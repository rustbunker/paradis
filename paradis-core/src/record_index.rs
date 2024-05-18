use crate::internal::Sealed;
use std::hash::Hash;

/// A type suitable for use as an index into a collection of records.
///
/// This trait is currently sealed in order to make changes more easily, and because
/// there currently is no use case for implementing it outside this crate. If you have a
/// use case for this, please file an issue.
///
/// # Safety
///
/// Consumers of this trait must be able to rely on the correctness of the provided methods
/// in `unsafe` contexts.
///
/// All auxiliary traits required by this trait, such as `Eq`, `Ord` or `Hash`,
/// *must* be implemented correctly.
///
/// If two indices compare unequal, then they must not access the same record in a collection.
pub unsafe trait RecordIndex: Sealed + Eq + Copy + Send + Sync + Ord + Hash {
    // fn bounds_overlap(bounds1: &Bounds<Self>, bounds2: &Bounds<Self>) -> bool;

    /// Determine if a set of bounds contains another set of bounds.
    fn contains_bounds(container: &Bounds<Self>, bounds: &Bounds<Self>) -> bool;

    /// Determine if this index is contained inside the provided bounds.
    fn in_bounds(&self, bounds: &Bounds<Self>) -> bool;

    /// Expand these bounds to include the given index.
    fn enclose_index(bounds: &mut Bounds<Self>, index: Self);

    /// Returns a set of bounds that are empty (zero extent).
    fn empty_bounds() -> Bounds<Self>;

    /// Returns a set of bounds that exactly contain only the provided index.
    fn bounds_for_index(index: Self) -> Bounds<Self>;
}

/// Bounds associated with an index type.
///
/// This is a description of the bounds of a multidimensional array, through the definition of
/// an *offset* and an *extent*. [`Bounds`] can also be used to describe the bounds of an
/// index list, which can be used to ensure that a collection of indices are *all* in bounds,
/// and therefore bounds checks can be eliminated up front, prior to iteration.
///
/// In general, [`Bounds`] is expected to be used with either integers like `usize` for
/// one-dimensional arrays or tuples like `(usize, usize)` for multidimensional arrays.
/// For data structures, the offset is typically zero along each dimension, and the `extent`
/// describes the "length" of the array along each dimension.
///
/// *Offset* primarily exists in order for index lists to more tightly describe their bounds,
/// which can be used, for example, to ensure that index lists with disjoint bounds contain
/// disjoint indices.
/// However, a data structure *can* have non-zero offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds<I> {
    /// The offset of the bounds.
    pub offset: I,
    /// The extent of the bounds.
    pub extent: I,
}

impl<I> Bounds<I> {
    /// Zips the offset and extent of two [`Bounds`] instances.
    pub fn zip<I2>(self, other: Bounds<I2>) -> Bounds<(I, I2)> {
        Bounds {
            offset: (self.offset, other.offset),
            extent: (self.extent, other.extent),
        }
    }
}

impl<I: RecordIndex> Bounds<I> {
    /// Check if these bounds contain `other`.
    pub fn contains_bounds(&self, other: &Bounds<I>) -> bool {
        I::contains_bounds(self, other)
    }

    /// Check if these bounds contain the given index.
    pub fn contains_index(&self, index: I) -> bool {
        index.in_bounds(self)
    }

    /// Expand these bounds — if needed — so that the given index is contained in the
    /// updated bounds.
    pub fn enclose_index(&mut self, index: I) {
        I::enclose_index(self, index)
    }

    /// Constructs empty bounds (zero extent along each dimension).
    pub fn new_empty() -> Self {
        I::empty_bounds()
    }

    /// Constructs bounds large enough to hold only exactly the given index.
    pub fn bounds_for_index(index: I) -> Self {
        I::bounds_for_index(index)
    }
}

macro_rules! impl_single_dim_index {
    ($ty:ty) => {
        unsafe impl RecordIndex for $ty {
            #[inline]
            fn contains_bounds(container: &Bounds<Self>, bounds: &Bounds<Self>) -> bool {
                let left_contained = container.offset <= bounds.offset;
                let right_contained =
                    bounds.offset + bounds.extent <= container.offset + container.extent;
                left_contained && right_contained
            }

            #[inline]
            fn in_bounds(&self, bounds: &Bounds<Self>) -> bool {
                let Bounds { offset, extent } = *bounds;
                let i = *self;
                offset <= i && i < (offset + extent)
            }

            #[inline]
            fn enclose_index(bounds: &mut Bounds<Self>, index: Self) {
                let new_offset = Self::min(bounds.offset, index);
                bounds.offset = new_offset;
                bounds.extent = Self::max(bounds.extent, index - new_offset + 1)
            }

            #[inline]
            fn empty_bounds() -> Bounds<Self> {
                Bounds {
                    offset: 0,
                    extent: 0,
                }
            }

            #[inline]
            fn bounds_for_index(index: Self) -> Bounds<Self> {
                Bounds {
                    offset: index,
                    extent: 1,
                }
            }
        }
    };
}

impl_single_dim_index!(usize);

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64",))]
impl_single_dim_index!(u32);

#[cfg(any(target_pointer_width = "64"))]
impl_single_dim_index!(u64);

/// Joins the provided list of expressions with the given separator
macro_rules! join_expressions {
    ($separator:tt; $token_head:expr, $($token_tail:expr),*) => {
        $token_head $($separator $token_tail)*
    }
}

/// Implement the RecordIndex trait for tuples
macro_rules! impl_tuple_index {
    (($($idx_type:tt),*), ($($idx:tt),*)) => {
        unsafe impl<$($idx_type: RecordIndex),*> RecordIndex for ($($idx_type),*) {
            #[inline]
            fn contains_bounds(container: &Bounds<Self>, bounds: &Bounds<Self>) -> bool {
                // First construct 1D bounds
                let container_bounds = (
                    $(Bounds { offset: container.offset.$idx, extent: container.extent.$idx }),*
                );
                let bounds = (
                    $(Bounds { offset: bounds.offset.$idx, extent: bounds.extent.$idx }),*
                );
                // For tuples, we return true if for each one-dimensional tuple element pair,
                // the bound is contained in the container (i.e., separately for each axis)
                join_expressions!(
                    &&;
                    $($idx_type::contains_bounds(&container_bounds.$idx, &bounds.$idx)),*
                )
            }

            #[inline]
            fn in_bounds(&self, bounds: &Bounds<Self>) -> bool {
                // First construct 1D bounds
                let bounds = (
                    $(Bounds { offset: bounds.offset.$idx, extent: bounds.extent.$idx }),*
                );
                // For tuples, we return true if the index is in bounds along every dimension
                join_expressions!(
                    &&;
                    $(self.$idx.in_bounds(&bounds.$idx)),*
                )
            }

            #[inline]
            fn enclose_index(bounds: &mut Bounds<Self>, index: Self) {
                // First create 1D bounds
                let mut bounds_1d = (
                    $(Bounds { offset: bounds.offset.$idx, extent: bounds.extent.$idx }),*
                );
                // Update along each axis
                $(bounds_1d.$idx.enclose_index(index.$idx);)*

                // Store the results back in tuple bounds
                $(bounds.offset.$idx = bounds_1d.$idx.offset;)*
                $(bounds.extent.$idx = bounds_1d.$idx.extent;)*
            }

            #[inline]
            fn empty_bounds() -> Bounds<Self> {
                // First create 1D bounds
                let bounds_1d = ($($idx_type::empty_bounds()),*);

                // Then merge
                Bounds {
                    offset: ($(bounds_1d.$idx.offset),*),
                    extent: ($(bounds_1d.$idx.offset),*)
                }
            }

            #[inline]
            fn bounds_for_index(index: Self) -> Bounds<Self> {
                // // First create 1D bounds
                let bounds_1d = ($($idx_type::bounds_for_index(index.$idx)),*);

                // Then merge
                Bounds {
                    offset: ($(bounds_1d.$idx.offset),*),
                    extent: ($(bounds_1d.$idx.offset),*)
                }
            }
        }
    };
}

impl_tuple_index!((I0, I1), (0, 1));
impl_tuple_index!((I0, I1, I2), (0, 1, 2));
impl_tuple_index!((I0, I1, I2, I3), (0, 1, 2, 3));
impl_tuple_index!((I0, I1, I2, I3, I4), (0, 1, 2, 3, 4));

#[cfg(test)]
mod tests {
    use crate::{Bounds, RecordIndex};

    #[rustfmt::skip]
    #[test]
    fn usize_in_bounds() {
        // Positive tests
        assert!(0usize.in_bounds(&Bounds { offset: 0, extent: 1 }));
        assert!(1usize.in_bounds(&Bounds { offset: 1, extent: 1 }));
        assert!(1usize.in_bounds(&Bounds { offset: 1, extent: 1 }));
        assert!(1usize.in_bounds(&Bounds { offset: 0, extent: 2 }));

        // Negative tests
        assert!(!0usize.in_bounds(&Bounds { offset: 0, extent: 0 }));
        assert!(!1usize.in_bounds(&Bounds { offset: 0, extent: 0 }));
        assert!(!1usize.in_bounds(&Bounds { offset: 0, extent: 1 }));
    }

    #[rustfmt::skip]
    #[test]
    fn usize_2dim_in_bounds() {
        // Zero extent
        let bounds_zero_extent = Bounds { offset: (0, 0), extent: (0, 0) };
        assert!(!(0usize, 0usize).in_bounds(&bounds_zero_extent)); // Point at the origin
        assert!(!(1usize, 1usize).in_bounds(&bounds_zero_extent)); // Any other point

        // Non-zero extent
        let bounds_normal = Bounds { offset: (0, 0), extent: (2, 2) };
        assert!((0usize, 0usize).in_bounds(&bounds_normal)); // Inside bounds
        assert!((1usize, 1usize).in_bounds(&bounds_normal)); // Inside bounds
        assert!(!(2usize, 2usize).in_bounds(&bounds_normal)); // Outside bounds

        // Bigger bounds with offset
        let bounds_offset = Bounds { offset: (1, 1), extent: (2, 2) };
        assert!(!(0usize, 0usize).in_bounds(&bounds_offset)); // Outside bounds
        assert!((1usize, 1usize).in_bounds(&bounds_offset)); // Edge of bounds
        assert!((2usize, 2usize).in_bounds(&bounds_offset)); // Inside bounds
        assert!(!(3usize, 3usize).in_bounds(&bounds_offset)); // Outside bounds

        // Bounds covering a larger area
        let bounds_large = Bounds { offset: (0, 0), extent: (5, 5) };
        assert!((0usize, 0usize).in_bounds(&bounds_large)); // Inside bounds
        assert!((4usize, 4usize).in_bounds(&bounds_large)); // Edge of bounds
        assert!(!(5usize, 5usize).in_bounds(&bounds_large)); // Outside bounds
    }

    #[rustfmt::skip]
    #[test]
    fn usize_3dim_in_bounds() {
        // Bounds with zero extent
        let bounds_zero_extent = Bounds { offset: (0, 0, 0), extent: (0, 0, 0) };
        assert!(!(0usize, 0usize, 0usize).in_bounds(&bounds_zero_extent)); // Origin
        assert!(!(1usize, 1usize, 1usize).in_bounds(&bounds_zero_extent)); // Any other point

        // Normal bounds
        let bounds_normal = Bounds { offset: (0, 0, 0), extent: (3, 3, 3) };
        assert!((0usize, 0usize, 0usize).in_bounds(&bounds_normal)); // Inside at origin
        assert!((1usize, 1usize, 1usize).in_bounds(&bounds_normal)); // Center point
        assert!((2usize, 2usize, 2usize).in_bounds(&bounds_normal)); // Edge of bounds
        assert!(!(3usize, 3usize, 3usize).in_bounds(&bounds_normal)); // Outside bounds

        // Bounds with offset
        let bounds_offset = Bounds { offset: (1, 1, 1), extent: (2, 2, 2) };
        assert!(!(0usize, 0usize, 0usize).in_bounds(&bounds_offset)); // Outside bounds
        assert!((1usize, 1usize, 1usize).in_bounds(&bounds_offset)); // Edge of bounds
        assert!((2usize, 2usize, 2usize).in_bounds(&bounds_offset)); // Inside bounds
        assert!(!(3usize, 3usize, 3usize).in_bounds(&bounds_offset)); // Outside bounds

        // Large bounds
        let bounds_large = Bounds { offset: (0, 0, 0), extent: (5, 5, 5) };
        assert!((0usize, 0usize, 0usize).in_bounds(&bounds_large)); // Inside at origin
        assert!((4usize, 4usize, 4usize).in_bounds(&bounds_large)); // Edge of bounds
        assert!(!(5usize, 5usize, 5usize).in_bounds(&bounds_large)); // Outside bounds
    }

    #[rustfmt::skip]
    #[test]
    fn usize_contains_bounds() {
        // Identical bounds - should contain
        assert!(usize::contains_bounds(&Bounds { offset: 0, extent: 1 },
                                       &Bounds { offset: 0, extent: 1 }));

        // Outer bounds larger than inner bounds - should contain
        assert!(usize::contains_bounds(&Bounds { offset: 0, extent: 5 },
                                       &Bounds { offset: 1, extent: 3 }));

        // Inner bounds larger than outer bounds - should not contain
        assert!(!usize::contains_bounds(&Bounds { offset: 1, extent: 3 },
                                        &Bounds { offset: 0, extent: 5 }));

        // Partial overlap - should not contain
        assert!(!usize::contains_bounds(&Bounds { offset: 0, extent: 3 },
                                        &Bounds { offset: 2, extent: 2 }));

        // No overlap - should not contain
        assert!(!usize::contains_bounds(&Bounds { offset: 0, extent: 2 },
                                        &Bounds { offset: 3, extent: 2 }));

        // Inner bounds start at the edge of outer bounds - should not contain
        assert!(!usize::contains_bounds(&Bounds { offset: 0, extent: 2 },
                                        &Bounds { offset: 2, extent: 1 }));

        // Inner bounds is exactly inside but offset is different - should contain
        assert!(usize::contains_bounds(&Bounds { offset: 0, extent: 5 },
                                       &Bounds { offset: 1, extent: 1 }));

        // Inner bounds touches the boundary edge of outer bounds - should contain
        assert!(usize::contains_bounds(&Bounds { offset: 0, extent: 4 },
                                       &Bounds { offset: 0, extent: 4 }));
    }

    #[rustfmt::skip]
    #[test]
    fn usize_2dim_contains_bounds() {
        // Identical bounds - should contain
        assert!(<(usize, usize)>::contains_bounds(&Bounds { offset: (0, 0), extent: (0, 0) },
                                                  &Bounds { offset: (0, 0), extent: (0, 0) }));
        assert!(<(usize, usize)>::contains_bounds(&Bounds { offset: (0, 0), extent: (1, 1) },
                                                  &Bounds { offset: (0, 0), extent: (1, 1) }));

        // Outer bounds larger than inner bounds - should contain
        assert!(<(usize, usize)>::contains_bounds(&Bounds { offset: (0, 0), extent: (3, 3) },
                                                  &Bounds { offset: (1, 1), extent: (1, 1) }));
        assert!(<(usize, usize)>::contains_bounds(&Bounds { offset: (0, 0), extent: (5, 5) },
                                                  &Bounds { offset: (1, 1), extent: (3, 3) }));

        // Inner bounds larger than outer bounds - should not contain
        assert!(!<(usize, usize)>::contains_bounds(&Bounds { offset: (1, 1), extent: (3, 3) },
                                                   &Bounds { offset: (0, 0), extent: (2, 2) }));

        // Partial overlap - should not contain
        assert!(!<(usize, usize)>::contains_bounds(&Bounds { offset: (0, 0), extent: (2, 2) },
                                                   &Bounds { offset: (1, 1), extent: (2, 2) }));

        // No overlap - should not contain
        assert!(!<(usize, usize)>::contains_bounds(&Bounds { offset: (0, 0), extent: (1, 1) },
                                                   &Bounds { offset: (2, 2), extent: (1, 1) }));

        // Inner bounds start at the edge of outer bounds - should not contain
        assert!(!<(usize, usize)>::contains_bounds(&Bounds { offset: (0, 0), extent: (2, 2) },
                                                   &Bounds { offset: (2, 2), extent: (1, 1) }));
    }

    #[rustfmt::skip]
    #[test]
    fn usize_3dim_contains_bounds() {
        // Identical bounds - should contain
        assert!(<(usize, usize, usize)>::contains_bounds(&Bounds { offset: (0, 0, 0), extent: (0, 0, 0) },
                                                         &Bounds { offset: (0, 0, 0), extent: (0, 0, 0) }));
        assert!(<(usize, usize, usize)>::contains_bounds(&Bounds { offset: (0, 0, 0), extent: (1, 1, 1) },
                                                         &Bounds { offset: (0, 0, 0), extent: (1, 1, 1) }));

        // Outer bounds larger than inner bounds - should contain
        assert!(<(usize, usize, usize)>::contains_bounds(&Bounds { offset: (0, 0, 0), extent: (3, 3, 3) },
                                                         &Bounds { offset: (1, 1, 1), extent: (1, 1, 1) }));
        assert!(<(usize, usize, usize)>::contains_bounds(&Bounds { offset: (0, 0, 0), extent: (5, 5, 5) },
                                                         &Bounds { offset: (1, 1, 1), extent: (3, 3, 3) }));

        // Inner bounds larger than outer bounds - should not contain
        assert!(!<(usize, usize, usize)>::contains_bounds(&Bounds { offset: (1, 1, 1), extent: (4, 4, 4) },
                                                          &Bounds { offset: (0, 0, 0), extent: (3, 3, 3) }));

        // Partial overlap - should not contain
        assert!(!<(usize, usize, usize)>::contains_bounds(&Bounds { offset: (0, 0, 0), extent: (2, 2, 2) },
                                                          &Bounds { offset: (1, 1, 1), extent: (2, 2, 2) }));

        // No overlap - should not contain
        assert!(!<(usize, usize, usize)>::contains_bounds(&Bounds { offset: (0, 0, 0), extent: (1, 1, 1) },
                                                          &Bounds { offset: (2, 2, 2), extent: (1, 1, 1) }));

        // Inner bounds start at the edge of outer bounds - should not contain
        assert!(!<(usize, usize, usize)>::contains_bounds(&Bounds { offset: (0, 0, 0), extent: (2, 2, 2) },
                                                          &Bounds { offset: (2, 2, 2), extent: (1, 1, 1) }));
    }
}
