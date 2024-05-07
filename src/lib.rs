//! Parallel processing of disjoint indices.
//!
//! **VERY EXPERIMENTAL, DO NOT USE**.

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
