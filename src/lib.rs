//! Parallel processing of disjoint indices.
//!
//! **VERY EXPERIMENTAL, DO NOT USE**.

#[cfg(feature = "rayon")]
pub mod rayon;

use std::collections::HashSet;
use std::hash::Hash;
pub use paradis_core::{slice, IntoUnsyncAccess, LinearUnsyncAccess, UnsyncAccess};
use std::ops::Range;

pub unsafe trait UniqueIndices: Sync + Send {
    type Index: Copy;

    unsafe fn get_unchecked(&self, i: usize) -> Self::Index;
    fn num_indices(&self) -> usize;

    fn get(&self, i: usize) -> Self::Index {
        assert!(i < self.num_indices(), "Index must be in bounds");
        unsafe { self.get_unchecked(i) }
    }
}

pub fn compose_access_with_indices<IntoAccess, Indices>(access: IntoAccess, indices: &Indices)
    -> UniqueIndicesWithAccess<'_, Indices, IntoAccess::Access>
where
    // TODO: Is the Sized bound necessary? Do we want it? The alternative is to sprinkle
    // ?Sized around, but I'm not sure whether we want that either. Gotta figure out...
    Indices: UniqueIndices + Sized,
    IntoAccess: IntoUnsyncAccess<Indices::Index>,
{
    UniqueIndicesWithAccess {
        indices,
        access: access.into_unsync_access(),
    }
}

/// Marker trait for types suitable as indices.
///
/// Any implementor of this trait must uphold the contract that if two indices compare unequal,
/// then they do not index the same location.
pub unsafe trait UniqueIndex: Eq + Copy {}

unsafe impl UniqueIndex for usize {}
unsafe impl UniqueIndex for (usize, usize) {}
unsafe impl UniqueIndex for (usize, usize, usize) {}
unsafe impl UniqueIndex for (usize, usize, usize, usize) {}
unsafe impl UniqueIndex for [usize; 1] {}
unsafe impl UniqueIndex for [usize; 2] {}
unsafe impl UniqueIndex for [usize; 3] {}
unsafe impl UniqueIndex for [usize; 4] {}
// TODO: More tuples, arrays etc.

#[derive(Debug)]
pub struct UniqueIndicesWithAccess<'a, Indices, Access> {
    indices: &'a Indices,
    access: Access
}

unsafe impl<'a, Indices, Access> UnsyncAccess<usize> for UniqueIndicesWithAccess<'a, Indices, Access>
where
    Indices: UniqueIndices,
    Access: UnsyncAccess<Indices::Index>,
{
    type Record = Access::Record;
    type RecordMut = Access::RecordMut;

    #[inline(always)]
    unsafe fn clone_access(&self) -> Self {
        Self {
            indices: self.indices,
            access: unsafe { self.access.clone_access() },
        }
    }

    #[inline(always)]
    fn in_bounds(&self, index: usize) -> bool {
        let in_bounds_in_index_list = index < self.indices.num_indices();
        if in_bounds_in_index_list {
            let index = unsafe { self.indices.get_unchecked(index) };
            self.access.in_bounds(index)
        } else {
            false
        }
    }

    #[inline(always)]
    unsafe fn get_unsync_unchecked(&self, index: usize) -> Self::Record {
        // Cannot use unchecked indexing here, see note in _mut
        self.access.get_unsync(self.indices.get(index))
    }

    #[inline(always)]
    unsafe fn get_unsync_unchecked_mut(&self, index: usize) -> Self::RecordMut {
        // Note: We can not use unchecked indexing here because
        // we can not know that the index we obtain for indexing into the access
        // is actually in bounds
        unsafe { self.access.get_unsync_mut(self.indices.get_unchecked(index)) }
    }
}

unsafe impl<'a, Indices, Access> LinearUnsyncAccess for UniqueIndicesWithAccess<'a, Indices, Access>
where
    Indices: UniqueIndices,
    Access: UnsyncAccess<Indices::Index>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.indices.num_indices()
    }
}

unsafe impl UniqueIndices for Range<usize> {
    type Index = usize;

    unsafe fn get_unchecked(&self, i: usize) -> usize {
        self.start + i
    }

    fn num_indices(&self) -> usize {
        self.end.saturating_sub(self.start)
    }
}

pub struct CheckedUniqueIndices<Idx> {
    // TODO: Generalize to something like IndexContainer that supports e.g. Vec<Idx>, &[Idx]
    indices: Vec<Idx>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonUniqueIndex;

impl<Idx: UniqueIndex> CheckedUniqueIndices<Idx> {
    pub fn from_hashable_indices(indices: Vec<Idx>) -> Result<Self, NonUniqueIndex>
    where
        Idx: Hash
    {
        // TODO: Implement re-usable "checker" for re-using allocations
        let hashed: HashSet<Idx> = indices.iter().copied().collect();
        if hashed.len() == indices.len() {
            Ok(Self { indices })
        } else {
            Err(NonUniqueIndex)
        }
    }
}

unsafe impl<Idx> UniqueIndices for CheckedUniqueIndices<Idx>
where
    Idx: UniqueIndex + Send + Sync
{
    type Index = Idx;

    unsafe fn get_unchecked(&self, i: usize) -> Self::Index {
        *self.indices.get_unchecked(i)
    }

    fn num_indices(&self) -> usize {
        self.indices.len()
    }
}

// pub unsafe trait DisjointIndexSubsets {
//     type IndexSubset<'subset>;

//     fn num_subsets(&self) -> usize;
//     fn subset_len(&self, subset_index: usize) -> usize;
//     fn get_subset<'subset>(&self, subset_index: usize) -> Self::IndexSubset<'subset>;
// }

// #[derive(Debug, Clone)]
// pub struct DisjointIndicesVec {
//     indices: Vec<usize>,
//     // max_idx: usize,
// }

// #[derive(Debug)]
// pub struct NotDisjoint;

// impl DisjointIndicesVec {
//     pub fn try_from_index_iter<I>(iter: I) -> Result<Self, NotDisjoint>
//     where
//         I: IntoIterator<Item = usize>,
//     {
//         // Remove outer generic call to avoid excessive monomorphization
//         Self::try_from_index_iter_inner(iter.into_iter())
//     }

//     fn try_from_index_iter_inner<I>(iter: I) -> Result<Self, NotDisjoint>
//     where
//         I: Iterator<Item = usize>,
//     {
//         // let mut max_idx = 0;
//         // TODO: Use faster hash? And/or switch to bitvec for sufficiently large number of indices
//         let mut visited_indices = HashSet::new();

//         let indices = iter
//             .map(|idx| {
//                 // if idx > max_idx {
//                 //     max_idx = idx;
//                 // }
//                 if visited_indices.insert(idx) {
//                     Ok(idx)
//                 } else {
//                     Err(NotDisjoint)
//                 }
//             })
//             .collect::<Result<Vec<_>, _>>()?;

//         Ok(Self {
//             indices,
//             // max_idx,
//         })
//     }
// }

// unsafe impl DisjointIndices for DisjointIndicesVec {
//     type Index = usize;

//     fn num_indices(&self) -> usize {
//         self.indices.len()
//     }

//     unsafe fn get_unchecked(&self, i: usize) -> usize {
//         *self.indices.get_unchecked(i)
//     }
// }
