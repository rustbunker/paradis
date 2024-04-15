//! Combinators used in the construction of unique index sets.

mod index_flatten;
mod index_product;
mod index_zip;

pub use index_flatten::{Concatenate, Concatenated, Flatten, IndexFlatten};
pub use index_product::IndexProduct;
pub use index_zip::IndexZip;
