//! Combinators used in the construction of unique index sets.

mod index_cast;
mod index_flatten;
mod index_product;
mod index_transpose;
mod index_zip;

pub use index_cast::IndexCast;
pub use index_flatten::{Concatenate, Concatenated, Flatten, IndexFlatten};
pub use index_product::IndexProduct;
pub use index_transpose::{IndexTranspose, Transpose};
pub use index_zip::{IndexAZip, IndexZip};
