//! Core primitives for `paradis`.
//!
//! `paradis-core` contains the core abstractions used by `paradis`. `paradis-core` is expected
//! to need breaking changes very rarely. Hopefully once the APIs are stabilized
//! no further breaking changes are necessary. Therefore, library authors who only want to
//! expose their data structures to `paradis` algorithms should depend on this crate
//! instead `paradis`.

mod par_access;
mod record_index;

pub use par_access::{BoundedParAccess, IntoParAccess, LinearParAccess, ParAccess};
pub use record_index::{Bounds, RecordIndex};

pub mod slice;

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
