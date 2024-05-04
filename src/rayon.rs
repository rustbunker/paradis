//! Interoperability with `rayon` parallel iterators.
//!
use crate::iter::AccessIterator;
use crate::IntoParAccess;
use paradis_core::LinearParAccess;
use rayon::iter::plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer};
use rayon::iter::{IndexedParallelIterator, ParallelIterator};

/// A parallel iterator for records in a collection.
///
/// Usually you do not need to interact with this directly.
/// Use [`create_par_iter`] instead.
#[derive(Debug)]
pub struct LinearParAccessIter<Access> {
    access: Access,
}

impl<Access> LinearParAccessIter<Access> {
    pub(crate) fn from_access<IntoAccess>(access: IntoAccess) -> Self
    where
        IntoAccess: IntoParAccess<Access = Access>,
        IntoAccess::Access: LinearParAccess,
    {
        let access = access.into_par_access();
        Self { access }
    }
}

/// Creates a [`rayon`] parallel iterator for the provided linear parallel access.
pub fn create_par_iter<IntoAccess>(access: IntoAccess) -> LinearParAccessIter<IntoAccess::Access>
where
    IntoAccess: IntoParAccess,
    IntoAccess::Access: LinearParAccess,
{
    LinearParAccessIter::from_access(access)
}

struct AccessProducer<Access> {
    access: Access,
    start_idx: usize,
    end_idx: usize,
}

impl<Access> Producer for AccessProducer<Access>
where
    Access: LinearParAccess,
{
    type Item = Access::Record;
    type IntoIter = AccessIterator<Access>;

    fn into_iter(self) -> Self::IntoIter {
        AccessIterator::new_for_range(self.access, self.start_idx..self.end_idx)
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        debug_assert!(index < (self.end_idx - self.start_idx));
        // SAFETY: The two producers both obtain unsyncrhonized access to the underlying data structure,
        // but they work on non-overlapping index sets
        let left = Self {
            access: unsafe { self.access.clone_access() },
            start_idx: self.start_idx,
            end_idx: self.start_idx + index,
        };
        let right = Self {
            access: self.access,
            start_idx: left.end_idx,
            end_idx: self.end_idx,
        };
        (left, right)
    }
}

impl<Access> ParallelIterator for LinearParAccessIter<Access>
where
    Access: LinearParAccess,
    Access::Record: Send,
{
    type Item = Access::Record;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.access.len())
    }
}

impl<Access> IndexedParallelIterator for LinearParAccessIter<Access>
where
    Access: LinearParAccess,
    Access::Record: Send,
{
    fn len(&self) -> usize {
        self.access.len()
    }

    fn drive<C: Consumer<Self::Item>>(self, consumer: C) -> C::Result {
        bridge(self, consumer)
    }

    fn with_producer<CB: ProducerCallback<Self::Item>>(self, callback: CB) -> CB::Output {
        let access = self.access;

        callback.callback(AccessProducer {
            start_idx: 0,
            end_idx: access.len(),
            access,
        })
    }
}
