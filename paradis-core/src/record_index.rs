use crate::internal::Sealed;

/// A type suitable for use as an index into a collection of records.
///
/// Any implementor of this trait must uphold the contract that if two indices compare unequal,
/// then they do not index the same location.
///
/// TODO: I'm not sure if it's necessary to seal this trait, but until someone comes up with
/// a compelling use case that requires implementing this trait outside of this crate,
/// it's convenient to do so.
pub unsafe trait RecordIndex: Sealed + Eq + Copy + Send + Sync {}

unsafe impl RecordIndex for usize {}

#[cfg(any(
    target_pointer_width = "32",
    target_pointer_width = "64",
    target_pointer_width = "128",
))]
unsafe impl RecordIndex for u32 {}

#[cfg(any(target_pointer_width = "64", target_pointer_width = "128",))]
unsafe impl RecordIndex for u64 {}

unsafe impl<I0: RecordIndex> RecordIndex for (I0,) {}

unsafe impl<I0: RecordIndex, I1: RecordIndex> RecordIndex for (I0, I1) {}

unsafe impl<I0: RecordIndex, I1: RecordIndex, I2: RecordIndex> RecordIndex for (I0, I1, I2) {}

unsafe impl<I0: RecordIndex, I1: RecordIndex, I2: RecordIndex, I3: RecordIndex> RecordIndex
    for (I0, I1, I2, I3)
{
}

unsafe impl<I0: RecordIndex, I1: RecordIndex, I2: RecordIndex, I3: RecordIndex, I4: RecordIndex>
    RecordIndex for (I0, I1, I2, I3, I4)
{
}

unsafe impl<
        I0: RecordIndex,
        I1: RecordIndex,
        I2: RecordIndex,
        I3: RecordIndex,
        I4: RecordIndex,
        I5: RecordIndex,
    > RecordIndex for (I0, I1, I2, I3, I4, I5)
{
}

unsafe impl<
        I0: RecordIndex,
        I1: RecordIndex,
        I2: RecordIndex,
        I3: RecordIndex,
        I4: RecordIndex,
        I5: RecordIndex,
        I6: RecordIndex,
    > RecordIndex for (I0, I1, I2, I3, I4, I5, I6)
{
}
