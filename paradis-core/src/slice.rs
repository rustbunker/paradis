//! Core primitives for slices.
use crate::{IntoParAccess, LinearParAccess, ParAccess};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct ParSliceAccessMut<'a, T> {
    ptr: *mut T,
    len: usize,
    marker: PhantomData<&'a mut T>,
}

unsafe impl<'a, T: Sync> Sync for ParSliceAccessMut<'a, T> {}
unsafe impl<'a, T: Send> Send for ParSliceAccessMut<'a, T> {}

unsafe impl<'a, T: Sync + Send> ParAccess for ParSliceAccessMut<'a, T> {
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
        &mut *self.ptr.add(index)
    }

    #[inline(always)]
    fn in_bounds(&self, index: usize) -> bool {
        index < self.len
    }
}

impl<'a, T: Sync + Send> IntoParAccess<usize> for &'a mut [T] {
    type Access = ParSliceAccessMut<'a, T>;

    fn into_par_access(self) -> Self::Access {
        ParSliceAccessMut {
            ptr: self.as_mut_ptr(),
            len: self.len(),
            marker: PhantomData,
        }
    }
}

unsafe impl<'a, T: Sync + Send> LinearParAccess for ParSliceAccessMut<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}
