//! Parallel processing of disjoint indices.
//!
//! **`paradis` is currently at an early, experimental stage.
//!   Test coverage is deliberately poor in order to make it easier to iterate on the
//!   overall design. Community feedback is very welcome!**.
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
//!
//! # Safe parallel access with index lists
//!
//!
//!
//! **VERY EXPERIMENTAL, DO NOT USE**.
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
