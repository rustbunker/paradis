use crate::IntoParAccess;
use paradis_core::LinearParAccess;
use rayon::iter::plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer};
use rayon::iter::{IndexedParallelIterator, ParallelIterator};

/// A parallel iterator for mutable records in a collection.
#[derive(Debug)]
pub struct LinearParAccessIterMut<Access> {
    access: Access,
}

impl<Access> LinearParAccessIterMut<Access> {
    pub fn from_access<IntoAccess>(access: IntoAccess) -> Self
    where
        IntoAccess: IntoParAccess<Access = Access>,
        IntoAccess::Access: LinearParAccess,
    {
        let access = access.into_par_access();
        Self { access }
    }
}

/// Creates a mutable [`rayon`] parallel iterator for the provided linear parallel access.
pub fn create_par_iter_mut<IntoAccess>(
    access: IntoAccess,
) -> LinearParAccessIterMut<IntoAccess::Access>
where
    IntoAccess: IntoParAccess,
    IntoAccess::Access: LinearParAccess,
{
    LinearParAccessIterMut::from_access(access)
}

struct AccessProducerMut<Access> {
    access: Access,
    start_idx: usize,
    end_idx: usize,
}

impl<Access> Iterator for AccessProducerMut<Access>
where
    Access: LinearParAccess,
{
    type Item = Access::RecordMut;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start_idx < self.end_idx {
            let item = unsafe { self.access.get_unsync_mut(self.start_idx) };
            self.start_idx += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<Access> ExactSizeIterator for AccessProducerMut<Access> where Access: LinearParAccess {}

impl<Access> DoubleEndedIterator for AccessProducerMut<Access>
where
    Access: LinearParAccess,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        // TODO: Need to test this impl
        if self.end_idx > self.start_idx {
            self.end_idx -= 1;
            let item = unsafe { self.access.get_unsync_mut(self.end_idx) };
            Some(item)
        } else {
            None
        }
    }
}

impl<Access> Producer for AccessProducerMut<Access>
where
    Access: LinearParAccess,
{
    type Item = Access::RecordMut;
    type IntoIter = Self;

    fn into_iter(self) -> Self::IntoIter {
        AccessProducerMut {
            access: self.access,
            start_idx: self.start_idx,
            end_idx: self.end_idx,
        }
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

impl<Access> ParallelIterator for LinearParAccessIterMut<Access>
where
    Access: LinearParAccess,
    Access::RecordMut: Send,
{
    type Item = Access::RecordMut;

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

impl<Access> IndexedParallelIterator for LinearParAccessIterMut<Access>
where
    Access: LinearParAccess,
    Access::RecordMut: Send,
{
    fn len(&self) -> usize {
        self.access.len()
    }

    fn drive<C: Consumer<Self::Item>>(self, consumer: C) -> C::Result {
        bridge(self, consumer)
    }

    fn with_producer<CB: ProducerCallback<Self::Item>>(self, callback: CB) -> CB::Output {
        let access = self.access;
        callback.callback(AccessProducerMut {
            start_idx: 0,
            end_idx: access.len(),
            access,
        })
    }
}
