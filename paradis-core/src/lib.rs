//! Core primitives for `paradis`.
//!
//! `paradis-core` contains the core abstractions used by `paradis`. `paradis-core` is expected
//! to need breaking changes very rarely. Hopefully once the APIs are stabilized
//! no further breaking changes are necessary. Therefore, library authors who only want to
//! expose their data structures to `paradis` algorithms should depend on this crate
//! instead `paradis`.

mod par_access;

pub use par_access::{IntoParAccess, LinearParAccess, ParAccess};

pub mod slice;
