use crate::internal::Sealed;

/// A type suitable for use as an index into a collection of records.
///
/// Any implementor of this trait must uphold the contract that if two indices compare unequal,
/// then they do not index the same location.
///
/// TODO: I'm not sure if it's necessary to seal this trait, but until someone comes up with
/// a compelling use case that requires implementing this trait outside of this crate,
/// it's convenient to do so.
pub unsafe trait RecordIndex: Sealed + Eq + Copy + Send + Sync {
    // fn bounds_overlap(bounds1: &Bounds<Self>, bounds2: &Bounds<Self>) -> bool;

    fn contains_bounds(container: &Bounds<Self>, bounds: &Bounds<Self>) -> bool;
    fn in_bounds(&self, bounds: &Bounds<Self>) -> bool;
}

/// Bounds associated with an index type.
///
/// `Bounds` is essentially a generalization of `len`, i.e. the size of a one-dimensional data
/// structure, to include an `offset` and a possibly multidimensional `extent` that describes
/// the number of entries along each dimension. This way, `Bounds` can be used both to describe
/// the bounds of an index list, or the bounds of a data structure.
/// A motivating factor for this design is that it allows us to quickly check if
/// the bounds of an index set are completely contained inside the bounds of a data structure,
/// which allows us to eliminate bounds checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds<I> {
    pub offset: I,
    pub extent: I,
}

macro_rules! impl_single_dim_index {
    ($ty:ty) => {
        unsafe impl RecordIndex for $ty {
            #[inline(always)]
            fn contains_bounds(container: &Bounds<Self>, bounds: &Bounds<Self>) -> bool {
                let left_contained = container.offset <= bounds.offset;
                let right_contained =
                    bounds.offset + bounds.extent <= container.offset + container.extent;
                left_contained && right_contained
            }

            #[inline(always)]
            fn in_bounds(&self, bounds: &Bounds<Self>) -> bool {
                let Bounds { offset, extent } = *bounds;
                let i = *self;
                offset <= i && i < (offset + extent)
            }
        }
    };
}

impl_single_dim_index!(usize);

#[cfg(any(
    target_pointer_width = "32",
    target_pointer_width = "64",
    target_pointer_width = "128",
))]
impl_single_dim_index!(u32);

#[cfg(any(target_pointer_width = "64", target_pointer_width = "128",))]
impl_single_dim_index!(u64);

macro_rules! impl_tuple_index {
    (($ty_head:tt, $($ty_tail:tt),*), ($idx_head:tt, $($idx_tail:tt),*)) => {
        unsafe impl<$ty_head: RecordIndex, $($ty_tail: RecordIndex),*> RecordIndex for ($ty_head, $($ty_tail),*) {
            #[inline(always)]
            fn contains_bounds(container: &Bounds<Self>, bounds: &Bounds<Self>) -> bool {
                let container_bounds = (
                    Bounds { offset: container.offset.$idx_head, extent: container.extent.$idx_head },
                    $(Bounds { offset: container.offset.$idx_tail, extent: container.extent.$idx_tail }),*
                );
                let bounds = (
                    Bounds { offset: bounds.offset.$idx_head, extent: bounds.extent.$idx_head },
                    $(Bounds { offset: bounds.offset.$idx_tail, extent: bounds.extent.$idx_tail }),*
                );
                // For tuples, we return true if for each one-dimensional tuple element pair,
                // the bound is contained in the container
                // (i.e., separately for each axis)
                $ty_head::contains_bounds(&container_bounds.$idx_head, &bounds.$idx_head)
                    $(&& $ty_tail::contains_bounds(&container_bounds.$idx_tail, &bounds.$idx_tail))*
            }

            #[inline(always)]
            fn in_bounds(&self, bounds: &Bounds<Self>) -> bool {
                let bounds = (
                    Bounds { offset: bounds.offset.$idx_head, extent: bounds.extent.$idx_head },
                    $(Bounds { offset: bounds.offset.$idx_tail, extent: bounds.extent.$idx_tail }),*
                );
                self.$idx_head.in_bounds(&bounds.$idx_head)
                    $(&& self.$idx_tail.in_bounds(&bounds.$idx_tail))*
            }
        }
    }
}

impl_tuple_index!((I0,), (0,));
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
