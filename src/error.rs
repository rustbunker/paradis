//! Error types used throughout the library.

use std::fmt::{Display, Formatter};

/// An error indicating that indices were out of bounds.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct OutOfBounds;

impl Display for OutOfBounds {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "an index is out of bounds")
    }
}

impl std::error::Error for OutOfBounds {}

/// An error indicating that the indices in an index list were not unique.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonUniqueIndex;

impl Display for NonUniqueIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "indices are not unique")
    }
}

impl std::error::Error for NonUniqueIndex {}
