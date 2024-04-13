use nalgebra::{dmatrix, DMatrix, DVectorViewMut, Dyn, Scalar, U1};
use paradis::rayon::create_par_iter;
use paradis::unique::{compose_access_with_indices, CheckedUniqueIndices, UniqueIndices};
use paradis::ParAccess;
use paradis_core::LinearParAccess;
use rayon::iter::ParallelIterator;
use std::marker::PhantomData;

/// Facilitates (parallel) unsynchronized access to mutable columns of a DMatrix
pub struct DMatrixColParAccessMut<'a, T> {
    ptr: *mut T,
    rows: usize,
    cols: usize,
    marker: PhantomData<&'a T>,
}

impl<'a, T> DMatrixColParAccessMut<'a, T> {
    pub fn from_matrix_mut(matrix: &'a mut DMatrix<T>) -> Self {
        DMatrixColParAccessMut {
            rows: matrix.nrows(),
            cols: matrix.ncols(),
            marker: Default::default(),
            ptr: matrix.as_mut_ptr(),
        }
    }
}

unsafe impl<'a, T> Send for DMatrixColParAccessMut<'a, T> {}
unsafe impl<'a, T> Sync for DMatrixColParAccessMut<'a, T> {}

unsafe impl<'a, T: Scalar> ParAccess<usize> for DMatrixColParAccessMut<'a, T> {
    type Record = DVectorViewMut<'a, T>;

    #[inline(always)]
    unsafe fn clone_access(&self) -> Self {
        Self {
            ptr: self.ptr,
            rows: self.rows,
            cols: self.cols,
            marker: Default::default(),
        }
    }

    #[inline(always)]
    unsafe fn get_unsync_unchecked(&self, index: usize) -> Self::Record {
        let offset = index * self.rows;
        let len = self.rows;
        unsafe {
            let slice = std::slice::from_raw_parts_mut(self.ptr.add(offset), len);
            DVectorViewMut::from_slice_generic(slice, Dyn(len), U1)
        }
    }

    #[inline(always)]
    fn in_bounds(&self, index: usize) -> bool {
        index < self.cols
    }
}

unsafe impl<'a, T: Scalar> LinearParAccess for DMatrixColParAccessMut<'a, T> {
    fn len(&self) -> usize {
        self.cols
    }
}

/// Facilitates mutable (parallel) unsynchronized access to elements of a DMatrix
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

unsafe impl<'a, T: Scalar> ParAccess<(usize, usize)> for DMatrixParAccessMut<'a, T> {
    type Record = &'a mut T;

    unsafe fn clone_access(&self) -> Self {
        Self {
            ptr: self.ptr,
            rows: self.rows,
            cols: self.cols,
            marker: self.marker.clone(),
        }
    }

    fn in_bounds(&self, (i, j): (usize, usize)) -> bool {
        i < self.rows && j < self.cols
    }

    unsafe fn get_unsync_unchecked(&self, (i, j): (usize, usize)) -> Self::Record {
        // Storage is col major
        let linear_idx = j * self.rows + i;
        &mut *self.ptr.add(linear_idx)
    }
}

fn main() {
    example_par_matrix_entries_iteration();
    example_par_matrix_submatrix_iteration();
    example_par_matrix_superdiagonal_iteration();
    example_par_column_iteration();
}

fn example_par_matrix_entries_iteration() {
    let m = 100;
    let n = 1000;
    let mut matrix = DMatrix::repeat(m, n, 1.0);

    let matrix_access = DMatrixParAccessMut::from_matrix_mut(&mut matrix);

    let indices = vec![(0, 0), (1, 0), (99, 100)];
    let checked_indices =
        CheckedUniqueIndices::from_hashable_indices(indices.clone()).expect("All indices unique");

    let access = compose_access_with_indices(matrix_access, &checked_indices);
    create_par_iter(access).for_each(|a_ij| *a_ij *= 2.0);

    for (i, j) in (0..m).zip(0..n) {
        let elem = matrix[(i, j)];
        if indices.contains(&(i, j)) {
            assert_eq!(elem, 2.0);
        } else {
            assert_eq!(elem, 1.0);
        }
    }
}

fn example_par_matrix_submatrix_iteration() {
    let mut matrix = dmatrix![ 0,  1,  2,  3,  4;
                               5,  6,  7,  8,  9;
                              10, 11, 12, 13, 14;
                              15, 16, 17, 18, 19 ];
    let matrix_access = DMatrixParAccessMut::from_matrix_mut(&mut matrix);

    // The 2x2 submatrix starting at (1, 2) can be described by a Cartesian product of index ranges
    let indices = (1..=2).index_product(2..=3);
    let access = compose_access_with_indices(matrix_access, &indices);
    create_par_iter(access).for_each(|a_ij| *a_ij *= 2);

    assert_eq!(
        matrix,
        dmatrix![ 0,  1,  2,  3,  4;
                  5,  6, 14, 16,  9;
                 10, 11, 24, 26, 14;
                 15, 16, 17, 18, 19 ]
    );
}

fn example_par_matrix_superdiagonal_iteration() {
    let mut matrix = dmatrix![ 0,  1,  2,  3,  4;
                               5,  6,  7,  8,  9;
                              10, 11, 12, 13, 14 ];
    let matrix_access = DMatrixParAccessMut::from_matrix_mut(&mut matrix);

    // The first superdiagonal corresponds to zipping two index sets
    let superdiagonal_indices = (0..3).index_zip(1..4);
    let access = compose_access_with_indices(matrix_access, &superdiagonal_indices);
    create_par_iter(access).for_each(|a_ij| *a_ij *= 2);

    assert_eq!(
        matrix,
        dmatrix![ 0,  2,  2,  3,  4;
                  5,  6, 14,  8,  9;
                 10, 11, 12, 26, 14 ]
    );
}

fn example_par_column_iteration() {
    // Iterate over columns in parallel
    let m = 100;
    let n = 1000;
    let mut matrix = DMatrix::repeat(m, n, 2.0);
    let col_access = DMatrixColParAccessMut::from_matrix_mut(&mut matrix);

    // TODO: Combine with disjoint index access to show that we can use this to access a subset
    // of indices
    // let indices = 0..n;

    create_par_iter(col_access).for_each(|mut col| {
        assert_eq!(col.nrows(), m);
        assert_eq!(col.ncols(), 1);
        col *= 2.0;
    });

    assert!(matrix.iter().all(|&x| x == 4.0));
}
