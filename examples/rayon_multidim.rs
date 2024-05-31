use paradis::index::{narrow_access, IndexList};
use paradis::rayon::create_par_iter;
use paradis_core::{BoundedParAccess, Bounds, ParAccess};
use rayon::iter::ParallelIterator;
use std::marker::PhantomData;

fn main() {
    // A 2x2x2x2 multi-dim array
    let array = [
        [[[1, 2], [3, 4]], [[5, 6], [7, 8]]],
        [[[9, 10], [11, 12]], [[13, 14], [15, 16]]],
    ];

    {
        // Iterate over all elements
        let mut array = array;
        let access = FourDimArrayAccessMut::from(&mut array);

        let indices = (0..2)
            .index_product(0..2)
            .index_product(0..2)
            .index_product(0..2)
            // Flatten nested tuple to (usize, usize, usize, usize)
            .index_flatten();
        let access = narrow_access(access, &indices).unwrap();
        create_par_iter(access).for_each(|a_ijkl| *a_ijkl *= 2);

        assert_eq!(
            array,
            [
                [[[2, 4], [6, 8]], [[10, 12], [14, 16]]],
                [[[18, 20], [22, 24]], [[26, 28], [30, 32]]]
            ]
        );
    }

    {
        // Iterate only over select elements
        let mut array = array;
        let access = FourDimArrayAccessMut::from(&mut array);

        // We can think of our 2x2x2x2 array as a matrix of matrices
        // We first select all "outer" matrices as the Cartesian product
        let outer_indices = (0..2).index_product(0..2);
        // Then, for the inner matrices, we only want the diagonals
        let inner_indices = (0..2).index_zip(0..2);
        let indices = outer_indices
            .index_product(inner_indices)
            // We finally have to flatten the nested index type to (usize, usize, usize, usize)
            .index_flatten();

        // Restrict the parallel access to our selected indices
        let access = narrow_access(access, &indices).expect("Indices must be in bounds");
        create_par_iter(access).for_each(|a_ijkl| *a_ijkl *= 2);

        assert_eq!(
            array,
            [
                [[[2, 2], [3, 8]], [[10, 6], [7, 16]]],
                [[[18, 10], [11, 24]], [[26, 14], [15, 32]]]
            ]
        );
    }
}

struct FourDimArrayAccessMut<'data, T> {
    ptr: *mut T,
    dims: [usize; 4],
    marker: PhantomData<&'data mut T>,
}

impl<'data, T, const M: usize, const N: usize, const P: usize, const Q: usize>
    From<&'data mut [[[[T; Q]; P]; N]; M]> for FourDimArrayAccessMut<'data, T>
{
    fn from(array: &'data mut [[[[T; Q]; P]; N]; M]) -> Self {
        Self {
            ptr: array.as_mut_ptr().cast(),
            dims: [M, N, P, Q],
            marker: Default::default(),
        }
    }
}

unsafe impl<'data, T> Send for FourDimArrayAccessMut<'data, T> {}
unsafe impl<'data, T> Sync for FourDimArrayAccessMut<'data, T> {}

unsafe impl<'data, T: Send> ParAccess<(usize, usize, usize, usize)>
    for FourDimArrayAccessMut<'data, T>
{
    type Record = &'data mut T;

    unsafe fn clone_access(&self) -> Self {
        Self {
            ptr: self.ptr,
            dims: self.dims,
            marker: self.marker,
        }
    }

    unsafe fn get_unsync_unchecked(
        &self,
        (i, j, k, l): (usize, usize, usize, usize),
    ) -> Self::Record {
        let [m, n, p, _q] = self.dims;
        let offset = m * n * p * i + n * p * j + p * k + l;
        unsafe { &mut *self.ptr.add(offset) }
    }
}

unsafe impl<'data, T: Send> BoundedParAccess<(usize, usize, usize, usize)>
    for FourDimArrayAccessMut<'data, T>
{
    fn bounds(&self) -> Bounds<(usize, usize, usize, usize)> {
        Bounds {
            offset: (0, 0, 0, 0),
            extent: (self.dims[0], self.dims[1], self.dims[2], self.dims[3]),
        }
    }

    #[inline(always)]
    fn in_bounds(&self, (i, j, k, l): (usize, usize, usize, usize)) -> bool {
        let [m, n, p, q] = self.dims;
        i < m && j < n && k < p && l < q
    }
}
