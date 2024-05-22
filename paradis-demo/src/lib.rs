//! Functionality used for demonstration in documentation examples.
//!
//! Not intended to be used outside `paradis` doc tests.

use nalgebra::{DMatrix, Scalar};
use paradis_core::{BoundedParAccess, Bounds, ParAccess};
use std::marker::PhantomData;

pub struct DMatrixParAccessMut<'a, T> {
    ptr: *mut T,
    rows: usize,
    cols: usize,
    marker: PhantomData<&'a T>,
}

unsafe impl<'a, T> Send for DMatrixParAccessMut<'a, T> {}
unsafe impl<'a, T> Sync for DMatrixParAccessMut<'a, T> {}

impl<'a, T> DMatrixParAccessMut<'a, T> {
    pub fn from_matrix_mut(matrix: &'a mut DMatrix<T>) -> Self {
        Self {
            rows: matrix.nrows(),
            cols: matrix.ncols(),
            marker: Default::default(),
            ptr: matrix.as_mut_ptr(),
        }
    }
}

unsafe impl<'a, T: Scalar + Send> ParAccess<(usize, usize)> for DMatrixParAccessMut<'a, T> {
    type Record = &'a mut T;

    unsafe fn clone_access(&self) -> Self {
        Self {
            ptr: self.ptr,
            rows: self.rows,
            cols: self.cols,
            marker: self.marker,
        }
    }

    unsafe fn get_unsync_unchecked(&self, (i, j): (usize, usize)) -> Self::Record {
        // Storage is col major
        let linear_idx = j * self.rows + i;
        &mut *self.ptr.add(linear_idx)
    }
}

unsafe impl<'a, T: Scalar + Send> BoundedParAccess<(usize, usize)> for DMatrixParAccessMut<'a, T> {
    fn bounds(&self) -> Bounds<(usize, usize)> {
        Bounds {
            offset: (0, 0),
            extent: (self.rows, self.cols),
        }
    }

    fn in_bounds(&self, (i, j): (usize, usize)) -> bool {
        i < self.rows && j < self.cols
    }
}
