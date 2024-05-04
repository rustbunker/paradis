//! Functionality for constructing sequential iterators.
//!
//! While intended primarily for parallel access, a parallel access object
//! can also be used to construct *sequential* iterators.

use std::cmp::max;
use std::ops::Range;
use paradis_core::{IntoParAccess, LinearParAccess};

/// Constructs a sequential iterator for the provided access object.
pub fn create_iter<IntoAccess>(access: IntoAccess) -> AccessIterator<IntoAccess::Access>
where
    IntoAccess: IntoParAccess<usize>,
    IntoAccess::Access: LinearParAccess,
{
    let access = access.into_par_access();
    let len = access.len();
    AccessIterator::new_for_range(access, 0 .. len)
}

/// A sequential iterator for a linear access object.
///
/// Usually you do not need to interact with this directly.
/// Use [`create_iter`] instead.
#[derive(Debug)]
pub struct AccessIterator<Access> {
    access: Access,
    next_idx: usize,
    // One past the last index
    end_idx: usize,
}

impl<Access: LinearParAccess> AccessIterator<Access> {
    /// Construct an iterator for a subset of a collection described by a range of indices.
    ///
    /// # Panics
    ///
    /// Panics if the range is out of bounds with respect to the collection.
    pub fn new_for_range(access: Access, range: Range<usize>) -> Self {
        // if end < start, then the range is empty, so account for this
        let end = max(range.start + 1, range.end);
        assert!(end <= access.len(), "range must be in bounds of collection");
        Self {
            access,
            next_idx: range.start,
            end_idx: end,
        }
    }
}

impl<Access> Iterator for AccessIterator<Access>
where
    Access: LinearParAccess,
{
    type Item = Access::Record;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_idx < self.end_idx {
            let item = unsafe { self.access.get_unsync_unchecked(self.next_idx) };
            self.next_idx += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.end_idx, Some(self.end_idx))
    }
}

impl<Access> DoubleEndedIterator for AccessIterator<Access>
where
    Access: LinearParAccess,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.next_idx < self.end_idx {
            self.end_idx -= 1;
            Some(unsafe { self.access.get_unsync_unchecked(self.end_idx) })
        } else {
            None
        }
    }
}

impl<Access: LinearParAccess> ExactSizeIterator for AccessIterator<Access> {
    fn len(&self) -> usize {
        self.end_idx
    }
}
