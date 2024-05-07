use crate::internal;
use crate::RecordIndex;

/// Enables conversion of indices.
///
/// This trait is *sealed*, meaning that it cannot be implemented for external types.
/// The intention behind this trait is to permit converting e.g. `u16` or `u32` type indices
/// (or tuples thereof) to `usize`-based indices. This is primarily useful for saving space
/// when the indices need to be explicitly stored.
pub trait IndexFrom<SourceIndex>: internal::Sealed {
    /// Convert the source index to `Self`, the target index type.
    fn index_from(source: SourceIndex) -> Self;
}

impl IndexFrom<usize> for usize {
    fn index_from(source: usize) -> Self {
        source
    }
}

#[cfg(any(
    target_pointer_width = "32",
    target_pointer_width = "64",
    target_pointer_width = "128",
))]
impl IndexFrom<u32> for usize {
    fn index_from(source: u32) -> Self {
        source
            .try_into()
            .expect("Can always convert u32 to usize since we assume usize is at least 32 bits")
    }
}

#[cfg(any(target_pointer_width = "64", target_pointer_width = "128",))]
impl IndexFrom<u64> for usize {
    fn index_from(source: u64) -> Self {
        source
            .try_into()
            .expect("Can always convert u64 to usize since we assume usize is at least 64 bits")
    }
}

macro_rules! replace_with_usize {
    ($content:tt) => {
        usize
    };
}

macro_rules! impl_tuple_index_from {
    ($($i:tt),*) => {
        impl<$($i),*> IndexFrom<($($i),*)> for ($(replace_with_usize!($i)),*)
        where
            $($i: RecordIndex),*,
            usize: $(IndexFrom<$i> +)*,
        {
            #[allow(non_snake_case)]
            fn index_from(($($i),*): ($($i), *)) -> Self {
                ( $(usize::index_from($i)),* )
            }
        }
    }
}

impl_tuple_index_from!(I0, I1);
impl_tuple_index_from!(I0, I1, I2);
impl_tuple_index_from!(I0, I1, I2, I3);
impl_tuple_index_from!(I0, I1, I2, I3, I4);
