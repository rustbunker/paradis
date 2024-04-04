//! Parallel processing of disjoint indices.
//!
//! **VERY EXPERIMENTAL, DO NOT USE**.

#[cfg(feature = "rayon")]
pub mod rayon;

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

    fn combine_with_access<Access>(&self, access: Access) -> UniqueIndicesWithAccess<'_, Self, Access>
    where
        Access: UnsyncAccess<Self::Index>,
        // TODO: Is this bound necessary? Do we want it? The alternative is to sprinkle
        // ?Sized around, but I'm not sure whether we want that either. Gotta figure out...
        Self: Sized
    {
        UniqueIndicesWithAccess {
            indices: self,
            access,
        }
    }
}

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
            let index = self.indices.get(index);
            self.access.in_bounds(index)
        } else {
            false
        }
    }

    #[inline(always)]
    unsafe fn get_unsync_unchecked(&self, index: usize) -> Self::Record {
        self.access.get_unsync_unchecked(self.indices.get(index))
    }

    #[inline(always)]
    unsafe fn get_unsync_unchecked_mut(&self, index: usize) -> Self::RecordMut {
        self.access.get_unsync_unchecked_mut(self.indices.get(index))
    }
}

unsafe impl<'a, Indices, Access> LinearUnsyncAccess for UniqueIndicesWithAccess<'a, Indices, Access>
where
    Indices: UniqueIndices,
    Access: UnsyncAccess<Indices::Index>,
{
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
