//! Parallel processing of disjoint indices.
//!
//! **VERY EXPERIMENTAL, DO NOT USE**.

#[cfg(feature = "rayon")]
pub mod rayon;
pub mod unique;

pub use paradis_core::{slice, IntoUnsyncAccess, LinearUnsyncAccess, UnsyncAccess};

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

// TODO: Implement u8, u16 and so on
// unsafe impl UniqueIndex for u8 {}
// unsafe impl UniqueIndex for u16 {}

// TODO: Implement IndexFrom<u8>, <u16>

// TODO: Implement IndexFrom for further tuples
