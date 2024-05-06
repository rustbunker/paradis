use crate::{Bounds, RecordIndex};

/// Unsynchronized access to records in a collection.
///
/// The trait provides *unsynchronized* access to *records*, defined by the
/// associated [`Record`](Self::Record) type, that are indexed by the `Index` parameter.
/// This trait gives data structures a means to give users
/// unfettered access to its records, without exposing internal implementation details
/// to the user. In turn, it is the responsibility of the user to maintain the invariants
/// of Rust's memory safety model.
///
/// An access object for a data structure is usually a lightweight wrapper around one
/// or more pointers, plus necessary metadata.
///
/// # Safety
///
/// A user must ensure that a record — accessed by the same index — is only accessed once,
/// at any given time. It may be helpful to imagine that [`Record`](Self::Record) is
/// a mutable reference, `&mut T`, and so it is undefined behavior to obtain two mutable
/// references to the same object. In other words, once a specific [`Record`](Self::Record)
/// is accessed, it can not be accessed through this access object again until the previous
/// instance is dropped.
///
/// Whether accessing records from a different thread is sound depends on whether the
/// records implement [`Send`].
///
/// An implementor must ensure that, for as long as any access object exists, the size of the
/// data structure remains unchanged, and that the same index always refers to the same "slot"
/// in the data structure.
pub unsafe trait ParAccess<Index: Copy>: Sync + Send {
    /// A record (element) in the underlying collection.
    type Record;

    /// Clones this access.
    ///
    /// # Safety
    ///
    /// This is unsafe, because methods that consume access objects typically assume that
    /// the access is exclusive. If the access is cloned, then the user must ensure that
    /// the invariants are uphold across *all* active accesses. Typically, this is achieved
    /// by having each access work on disjoint sets of records.
    unsafe fn clone_access(&self) -> Self;

    /// Unsynchronized lookup of record without bounds checks.
    ///
    /// # Safety
    ///
    /// See trait documentation.
    unsafe fn get_unsync_unchecked(&self, index: Index) -> Self::Record;
}

/// Facilitates unsynchronized access to records stored in the collection.
///
/// The trait provides *unsynchronized* access to (possibly mutable) *records*, defined by the
/// associated type [`Record`][`ParAccess::Record`].
///
/// # Safety
///
/// An implementor must ensure that it is sound for multiple threads to access a single record
/// *immutably*, provided that no thread accesses the same record mutably.
///
/// An implementor must furthermore ensure that it is sound for multiple threads to access
/// *disjoint* records mutably.
///
/// It is the responsibility of the consumer that:
///
/// - If any thread accesses a record mutably, then no other thread must access the same record.
/// - A mutable record must always be exclusive, even on a single thread.
///   In particular, a single thread is not permitted to obtain two records associated with the
///   same index in the collection if either record is mutable.
///
/// TODO: Make the invariants more precise
pub unsafe trait BoundedParAccess<Index: Copy>: ParAccess<Index> {
    /// The bounds of this data structure.
    ///
    /// # Safety
    ///
    /// The bounds for a collection must never change while an access object still lives.
    fn bounds(&self) -> Bounds<Index>;

    /// Determine if the provided index is in bounds.
    ///
    /// TODO: Remove this method in favor of using self.bounds().contains_index(idx)
    fn in_bounds(&self, index: Index) -> bool
    where
        Index: RecordIndex,
    {
        self.bounds().contains_index(index)
    }

    /// Unsynchronized mutable lookup of record.
    ///
    /// # Safety
    ///
    /// See trait documentation. TODO: Elaborate
    ///
    /// # Panics
    ///
    /// Implementors must ensure that the method panics if the index is out of bounds.
    #[inline(always)]
    unsafe fn get_unsync(&self, index: Index) -> Self::Record
    where
        Index: RecordIndex,
    {
        assert!(self.in_bounds(index), "index out of bounds");
        self.get_unsync_unchecked(index)
    }
}

/// A type that can be converted into a parallel access object.
pub trait IntoParAccess<Index: Copy = usize> {
    /// The access type obtained through this trait.
    type Access: BoundedParAccess<Index>;

    /// Obtain parallel access to this collection.
    fn into_par_access(self) -> Self::Access;
}

impl<Index: Copy, Access: BoundedParAccess<Index>> IntoParAccess<Index> for Access {
    type Access = Self;

    fn into_par_access(self) -> Self::Access {
        self
    }
}

/// An unsynchronized access to an array-like structure, indexed by `usize`.
pub unsafe trait LinearParAccess: BoundedParAccess<usize> {
    /// The number of accessible records.
    ///
    /// An implementor must ensure that this length never changes. In other words,
    /// once an access is obtained, the size of the collection must never not change
    /// while an access is active.
    ///
    /// It must also be equivalent to the result returned by the extent of the
    /// [`Bounds`] of the access.
    fn len(&self) -> usize {
        self.bounds().extent
    }
}
