//! Parallel processing of disjoint indices.
//!
//! **`paradis` is currently at an early, experimental stage.
//!   Test coverage is deliberately poor in order to make it easier to iterate on the
//!   overall design. Community feedback is very welcome!**
//!
//! `paradis` makes it easier to implement non-trivial parallel algorithms that require
//! access to a subset of indices into data structures that are structurally similar
//! to multidimensional arrays. It does so by providing abstractions at incrementally higher levels:
//!
//! 1. A low-level, unsafe abstraction for unsynchronized access to independent
//!    *records* of a collection.
//! 2. Higher-level abstractions built on top of the unsafe base layer that allow many
//!    parallel access patterns to be expressed in safe code, or with a minimum of unsafe code.
//!
//! The low-level abstractions are provided by the very lightweight `paradis-core` crate.
//! Library authors are encouraged to depend only on this crate in order to expose their
//! data structures for parallel access.
//!
//! To use `paradis`, add the following to your `Cargo.toml`:
//! ```toml
//! [dependencies]
#![doc = concat!("paradis = ", env!("CARGO_PKG_VERSION"))]
//!
//! # if you need to use rayon iterators
#![doc = concat!("paradis = { version = ", env!("CARGO_PKG_VERSION"), ", features = ['rayon'] }")]
//! ```
//!
//! # Low-level unsynchronized access
//!
//! Consider a toy problem in which we want to access every even and odd entry in a slice
//! with separate threads. This seemingly simple task is surprisingly tricky in Rust,
//! because we can not use [`split_at_mut`](slice::split_at_mut) to obtain disjoint
//! mutable portions of the slice, and have to result to pointer manipulation in order to
//! avoid obtaining two mutable references to the same object, which would be instant UB.
//! With low-level primitives in `paradis`, we can write the code in a more straightforward
//! way, resembling how you might resolve the issue in C++.
//!
//! ```rust
//! use paradis_core::{BoundedParAccess, IntoParAccess};
//! use std::thread::scope;
//!
//! let mut data = vec![0; 100];
//! let n = data.len();
//! let access = data.into_par_access();
//!
//! scope(|s| {
//!     s.spawn(|| {
//!         // The first thread touches elements at even indices
//!         for i in (0 .. n).step_by(2) {
//!             unsafe { *access.get_unsync(i) = 1; }
//!         }
//!     });
//!
//!     s.spawn(|| {
//!         // The second thread touches elements at odd indices
//!         for i in (1 .. n).step_by(2) {
//!             unsafe { *access.get_unsync(i) = 2; }
//!         }
//!     });
//! })
//! ```
//!
//! A key motivation of the low-level abstraction layer is the separation between
//! *data structure access* and *indexing*. The necessary pointer manipulation is contained
//! entirely inside the trait implementation of [`ParAccess`] and [`BoundedParAccess`] for slices,
//! leaving the user only with the responsibility to ensure that the indexing is correct.
//! In this example, this means that the two threads access non-overlapping subsets of the indices.
//!
//! Though unsafe, this example still exhibits some level of safety not present in analogous
//! C++ code. For one, obtaining the access borrows `data` mutably, and by the requirements
//! of the [`ParAccess`] trait, the collection can not be modified as long as an access
//! has been obtained. Therefore, it's not possible for the user to accidentally modify
//! the collection by adding/removing entries during iteration. Additionally, we require the
//! records to be `Send`, so in the example above, compilation will fail if the record type can not
//! safely be moved between threads.
//!
//! # Safe parallel access with index lists
//!
//! Once the access traits have been implemented for a particular data structure,
//! we can build abstractions on top that facilitate *safe* parallel access to arbitrary indices.
//! Not every access pattern can be immediately accommodated, but the vision for `paradis`
//! is to over time grow the set of patterns that can be expressed with safe code, and otherwise
//! reduce the amount of `unsafe` code necessary to accommodate the rest.
//!
//! The access implementations for a collection encapsulate all the internal unsafe details
//! necessary to expose parallel access to the collection. It is then the responsibility of the
//! user to ensure that the indices used for indexing into the collection are in bounds and unique.
//!
//! The contrived example that we used in the previous section can not be expressed safely
//! with the current version of `paradis`. However, this will be possible with features planned
//! for a future release. Instead, in this section, we will focus on a more common problem:
//! how can we mutate a subset of our collection in parallel, where the subset is
//! described by a list of unique indices? Consider the following code.
//!
//! ```rust
//! use paradis::index::{IndexList, narrow_access_to_indices};
//! use paradis::rayon::create_par_iter;
//! use rayon::iter::ParallelIterator;
//!
//! let mut data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
//! let indices = vec![4, 7, 1].check_unique().expect("Indices are unique");
//! let access = narrow_access_to_indices(data.as_mut_slice(), &indices)
//!     .expect("Indices are in bounds of the data structure");
//! create_par_iter(access).for_each(|x_i| *x_i = 0);
//!
//! assert_eq!(data, vec![0, 0, 2, 3, 0, 5, 6, 0, 8, 9]);
//! ```
//!
//! The above code shows how we can replace the value of selected integers in a slice
//! in parallel. A central idea here is the act of *narrowing* an access to a subset described
//! by a [`UniqueIndexList`](crate::index::IndexList). This creates a new access, which represents
//! a conceptual collection of records located at the provided indices in the original collection.
//!
//! However, the vector of indices at first only satisfies the
//! [`IndexList`](crate::index::IndexList) trait, which does not guarantee uniqueness.
//! To use the indices in a narrowing operation, we must prove to the compiler that they are
//! truly unique. The `.check_unique()` call turns an instance of
//! [`IndexList`](crate::index::IndexList) into an instance
//! of [`UniqueIndexList`](crate::index::UniqueIndexList).
//!
//! Checking that indices are unique is not free. We must, at the very least, visit all indices
//! in the list upfront. In many cases, the set of indices we want to access is *structured*.
//! An example of a structured index list is a range `start .. end`. The indices in the range are
//! unique by construction, and can therefore serve as input for narrowing an access object.
//!
//! Structured index lists are somewhat less common in one dimension, but the utility is
//! immediately clear in higher dimensions. For example, consider the following example of mutating
//! the first [superdiagonal](https://en.wikipedia.org/wiki/Diagonal#Matrices) of an
//! [nalgebra](https://nalgebra.org) matrix:
//!
//! ```rust
//! # use std::marker::PhantomData;
//! # use nalgebra::{Scalar, DMatrix};
//! # use paradis_core::{BoundedParAccess, Bounds, ParAccess};
//! #
//! # pub struct DMatrixParAccessMut<'a, T> {
//! #     ptr: *mut T,
//! #     rows: usize,
//! #     cols: usize,
//! #     marker: PhantomData<&'a T>,
//! # }
//! #
//! # unsafe impl<'a, T> Send for DMatrixParAccessMut<'a, T> {}
//! # unsafe impl<'a, T> Sync for DMatrixParAccessMut<'a, T> {}
//! #
//! # impl<'a, T> DMatrixParAccessMut<'a, T> {
//! #     pub fn from_matrix_mut(matrix: &'a mut DMatrix<T>) -> Self {
//! #         Self {
//! #             rows: matrix.nrows(),
//! #             cols: matrix.ncols(),
//! #             marker: Default::default(),
//! #             ptr: matrix.as_mut_ptr(),
//! #         }
//! #     }
//! # }
//! #
//! # unsafe impl<'a, T: Scalar + Send> ParAccess<(usize, usize)> for DMatrixParAccessMut<'a, T> {
//! #     type Record = &'a mut T;
//! #
//! #     unsafe fn clone_access(&self) -> Self {
//! #         Self {
//! #             ptr: self.ptr,
//! #             rows: self.rows,
//! #             cols: self.cols,
//! #             marker: self.marker,
//! #         }
//! #     }
//! #
//! #     unsafe fn get_unsync_unchecked(&self, (i, j): (usize, usize)) -> Self::Record {
//! #         // Storage is col major
//! #         let linear_idx = j * self.rows + i;
//! #         &mut *self.ptr.add(linear_idx)
//! #     }
//! # }
//! #
//! # unsafe impl<'a, T: Scalar + Send> BoundedParAccess<(usize, usize)> for DMatrixParAccessMut<'a, T> {
//! #     fn bounds(&self) -> Bounds<(usize, usize)> {
//! #         Bounds {
//! #             offset: (0, 0),
//! #             extent: (self.rows, self.cols),
//! #         }
//! #     }
//! #
//! #     fn in_bounds(&self, (i, j): (usize, usize)) -> bool {
//! #         i < self.rows && j < self.cols
//! #     }
//! # }
//! use nalgebra::dmatrix;
//! use paradis::index::{IndexList, narrow_access_to_indices};
//! use paradis::rayon::create_par_iter;
//! use rayon::iter::ParallelIterator;
//!
//! let mut matrix = dmatrix![1, 1, 1, 1, 1;
//!                           1, 1, 1, 1, 1;
//!                           1, 1, 1, 1, 1];
//!
//! // Superdiagonal indices are [(0, 1), (1, 2), (2, 3)]
//! let superdiagonal_indices = (0 .. 3).index_zip(1 .. 4);
//!
//! // We omit the implementation of the access object, which hopefully may be
//! // provided by nalgebra itself in the future
//! let access = DMatrixParAccessMut::from_matrix_mut(&mut matrix);
//! let superdiagonal_access = narrow_access_to_indices(access, &superdiagonal_indices)
//!     .expect("Indices are in bounds");
//!
//! create_par_iter(superdiagonal_access).for_each(|x_ij| *x_ij = 0);
//!
//! assert_eq!(matrix,
//!            dmatrix![1, 0, 1, 1, 1;
//!                     1, 1, 0, 1, 1;
//!                     1, 1, 1, 0, 1]);
//! ```
//!
//! In the above example, we constructed the indices of the superdiagonal by *zipping*
//! two one-dimensional index lists, creating a new list of two-dimensional indices.
//! The resulting indices are unique if either of the two index lists is unique, and therefore
//! we can directly use it for parallel iteration without further checking.
//!
//! [`index_zip`](crate::index::IndexList::index_zip) is an example of an *index combinator*.
//! With combinators, we can prove to the compiler that an index list has unique indices.
//! Most combinators maintain uniqueness under certain conditions. For example:
//!
//! - [`index_zip`](crate::index::IndexList::index_zip) is unique if one of the lists
//!   is unique (with some restrictions, see docs).
//! - [`index_product`](crate::index::IndexList::index_product), a Cartesian product of index lists,
//!   is unique if *both* of the lists are unique.
//! - [`index_transpose`](crate::index::IndexList::index_transpose) is unique if the input list
//!  is unique.
//! - and so on. See the documentation for index combinators in
//!   [`IndexList`](crate::index::IndexList) for more information.
//!
//! Since combinators are binary operators that produce nested tuples when chained together,
//! you may need to use [`index_flatten`](crate::index::IndexList::index_flatten)
//! when working with indices of three or more dimensions.
//!
//! There are however significant limitations in the patterns that can be currently expressed.
//! For example, imagine that we wanted to mutate over the first superdiagonal *and*
//! the first subdiagonal in the previous example. Simply concatenating lists together
//! does not preserve uniqueness, and so it is not quite clear how to do so in a fashion
//! that avoids runtime checking or an `unsafe` escape hatch.
//!
//! The vision for `paradis` is for the set of structured index lists that can be expressed
//! with safe combinators to expand over time, in order to cover more use cases.
//!
//! # TODOs
//!
//! TODO: Indicate necessary features for feature-gated functionality, such as rayon

#![warn(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod error;
pub mod index;
pub mod iter;
#[cfg(feature = "rayon")]
pub mod rayon;

mod index_from;

pub use index_from::IndexFrom;
pub use paradis_core::{
    slice, BoundedParAccess, Bounds, IntoParAccess, LinearParAccess, ParAccess, RecordIndex,
};

mod internal {
    pub trait Sealed {}

    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
    impl Sealed for usize {}

    impl<I0> Sealed for (I0,) {}
    impl<I0, I1> Sealed for (I0, I1) {}
    impl<I0, I1, I2> Sealed for (I0, I1, I2) {}
    impl<I0, I1, I2, I3> Sealed for (I0, I1, I2, I3) {}
    impl<I0, I1, I2, I3, I4> Sealed for (I0, I1, I2, I3, I4) {}
    impl<I0, I1, I2, I3, I4, I5> Sealed for (I0, I1, I2, I3, I4, I5) {}
    impl<I0, I1, I2, I3, I4, I5, I6> Sealed for (I0, I1, I2, I3, I4, I5, I6) {}
}
