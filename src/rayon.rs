use crate::{IntoUnsyncAccess};
use rayon::iter::plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer};
use rayon::iter::{IndexedParallelIterator, ParallelIterator};
use paradis_core::LinearUnsyncAccess;

#[derive(Debug)]
pub struct LinearUnsyncAccessParIter<Access> {
    access: Access,
}

impl<Access> LinearUnsyncAccessParIter<Access>
{
    pub fn from_access<IntoAccess>(
        access: IntoAccess
    ) -> Self
    where
        IntoAccess: IntoUnsyncAccess<Access=Access>,
        IntoAccess::Access: LinearUnsyncAccess
    {
        let access = access.into_unsync_access();
        Self { access }
    }
}

pub fn linear_unsync_access_par_iter<IntoAccess>(
    access: IntoAccess,
) -> LinearUnsyncAccessParIter<IntoAccess::Access>
where
    IntoAccess: IntoUnsyncAccess,
    IntoAccess::Access: LinearUnsyncAccess
{
    LinearUnsyncAccessParIter::from_access(access)
}

struct AccessProducerMut<Access> {
    access: Access,
    start_idx: usize,
    end_idx: usize,
}

impl<Access> Iterator for AccessProducerMut<Access>
where
    Access: LinearUnsyncAccess
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

impl<Access> ExactSizeIterator for AccessProducerMut<Access>
    where
        Access: LinearUnsyncAccess
{
}

impl<Access> DoubleEndedIterator for AccessProducerMut<Access>
where
    Access: LinearUnsyncAccess
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
    Access: LinearUnsyncAccess
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

impl<Access> ParallelIterator for LinearUnsyncAccessParIter<Access>
where
    Access: LinearUnsyncAccess,
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

impl<Access> IndexedParallelIterator for LinearUnsyncAccessParIter<Access>
where
    Access: LinearUnsyncAccess,
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
