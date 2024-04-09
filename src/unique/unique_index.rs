use crate::internal;

/// A type suitable for use as an index.
///
/// Any implementor of this trait must uphold the contract that if two indices compare unequal,
/// then they do not index the same location.
///
/// TODO: I'm not sure if it's necessary to seal this trait, but until someone comes up with
/// a compelling use case that requires implementing this trait outside of this crate,
/// it's convenient to do so.
///
/// TODO: Rename trait...
pub unsafe trait UniqueIndex: internal::Sealed + Eq + Copy + Send + Sync {}

unsafe impl UniqueIndex for usize {}

#[cfg(any(
    target_pointer_width = "32",
    target_pointer_width = "64",
    target_pointer_width = "128",
))]
unsafe impl UniqueIndex for u32 {}

#[cfg(any(target_pointer_width = "64", target_pointer_width = "128",))]
unsafe impl UniqueIndex for u64 {}

unsafe impl<I0: UniqueIndex> UniqueIndex for (I0,) {}

unsafe impl<I0: UniqueIndex, I1: UniqueIndex> UniqueIndex for (I0, I1) {}

unsafe impl<I0: UniqueIndex, I1: UniqueIndex, I2: UniqueIndex> UniqueIndex for (I0, I1, I2) {}

unsafe impl<I0: UniqueIndex, I1: UniqueIndex, I2: UniqueIndex, I3: UniqueIndex> UniqueIndex
    for (I0, I1, I2, I3)
{
}

unsafe impl<I0: UniqueIndex, I1: UniqueIndex, I2: UniqueIndex, I3: UniqueIndex, I4: UniqueIndex>
    UniqueIndex for (I0, I1, I2, I3, I4)
{
}

unsafe impl<
        I0: UniqueIndex,
        I1: UniqueIndex,
        I2: UniqueIndex,
        I3: UniqueIndex,
        I4: UniqueIndex,
        I5: UniqueIndex,
    > UniqueIndex for (I0, I1, I2, I3, I4, I5)
{
}

unsafe impl<
        I0: UniqueIndex,
        I1: UniqueIndex,
        I2: UniqueIndex,
        I3: UniqueIndex,
        I4: UniqueIndex,
        I5: UniqueIndex,
        I6: UniqueIndex,
    > UniqueIndex for (I0, I1, I2, I3, I4, I5, I6)
{
}
