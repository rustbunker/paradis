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
/// In order to use higher-level functionality in `paradis`,
/// implementors should try to implement [`BoundedParAccess`] too, whenever possible.
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

/// Unsynchronized access to a bounded collection.
///
/// This trait allows a data structure that is structurally similar to a multidimensional array
/// to describe its bounds, which enables use of the data structure with higher-level functionality
/// in `paradis`.
///
/// # Safety
///
/// The bounds reported *must* be correct, in the sense that any index contained in the bounds
/// may be used to access a valid record.
pub unsafe trait BoundedParAccess<Index: Copy>: ParAccess<Index> {
    /// The bounds of this data structure.
    ///
    /// # Safety
    ///
    /// The bounds for a collection must never change while an access object still lives.
    fn bounds(&self) -> Bounds<Index>;

    /// Determine if the provided index is in bounds.
    ///
    /// Can be overridden by implementors for a simpler implementation than the default,
    /// which may aid the compiler in eliding bounds checks in situations where bounds may
    /// not be eliminated upfront.
    fn in_bounds(&self, index: Index) -> bool
    where
        Index: RecordIndex,
    {
        self.bounds().contains_index(index)
    }

    /// Unsynchronized mutable lookup of record.
    ///
    /// The access is unsynchronized (and therefore unsafe), but bounds checked.
    ///
    /// # Safety
    ///
    /// See trait documentation.
    ///
    /// # Panics
    ///
    /// Panics if index is out of bounds.
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
///
/// # Safety
///
/// The length of the collection must be reported correctly.
pub unsafe trait LinearParAccess: BoundedParAccess<usize> {
    /// The number of accessible records in the collection.
    ///
    /// An implementor must ensure that this length never changes. In other words,
    /// once an access is obtained, the size of the collection must never not change
    /// while an access is active.
    ///
    /// It must also be equivalent to the result returned by the extent of the
    /// [`Bounds`] of the access.
    fn collection_len(&self) -> usize {
        self.bounds().extent
    }
}
