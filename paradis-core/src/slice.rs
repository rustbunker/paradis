//! Core primitives for slices.
use crate::par_access::ParAccess;
use crate::{BoundedParAccess, Bounds, IntoParAccess, LinearParAccess};
use std::marker::PhantomData;

/// Parallel access to a mutable slice.
#[derive(Debug)]
pub struct ParSliceAccessMut<'a, T> {
    ptr: *mut T,
    len: usize,
    marker: PhantomData<&'a mut T>,
}

impl<'a, T> ParSliceAccessMut<'a, T> {
    /// Obtain parallel access to a mutable slice.
    ///
    /// In most cases, prefer to go through the implementation of [`IntoParAccess`] instead of this
    /// method directly.
    pub fn from_slice_mut(slice: &'a mut [T]) -> Self {
        Self {
            ptr: slice.as_mut_ptr(),
            len: slice.len(),
            marker: PhantomData,
        }
    }
}

unsafe impl<'a, T: Send> Sync for ParSliceAccessMut<'a, T> {}
unsafe impl<'a, T: Send> Send for ParSliceAccessMut<'a, T> {}

unsafe impl<'a, T: Send> ParAccess<usize> for ParSliceAccessMut<'a, T> {
    type Record = &'a mut T;

    #[inline(always)]
    unsafe fn clone_access(&self) -> Self {
        Self {
            ptr: self.ptr,
            len: self.len,
            marker: Default::default(),
        }
    }

    #[inline(always)]
    unsafe fn get_unsync_unchecked(&self, index: usize) -> Self::Record {
        unsafe { &mut *self.ptr.add(index) }
    }
}

unsafe impl<'a, T: Send> BoundedParAccess<usize> for ParSliceAccessMut<'a, T> {
    #[inline(always)]
    fn in_bounds(&self, index: usize) -> bool {
        index < self.len
    }

    fn bounds(&self) -> Bounds<usize> {
        Bounds {
            offset: 0,
            extent: self.len,
        }
    }
}

impl<'a, T: Send> IntoParAccess<usize> for &'a mut [T] {
    type Access = ParSliceAccessMut<'a, T>;

    fn into_par_access(self) -> Self::Access {
        ParSliceAccessMut::from_slice_mut(self)
    }
}

unsafe impl<'a, T: Send> LinearParAccess for ParSliceAccessMut<'a, T> {
    fn collection_len(&self) -> usize {
        self.len
    }
}
