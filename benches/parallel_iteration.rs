use divan::Bencher;
use paradis::rayon::create_par_iter;
use paradis::unique::{CheckedIndexList, narrow_access_to_indices};
use paradis_core::{IntoParAccess, ParAccess};
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::hint::black_box;

fn main() {
    divan::main()
}

/// Get some baseline numbers for basic Rayon parallel iteration
#[divan::bench(
    args = [1_000_000, 10_000_000, 100_000_000]
)]
fn slice_baseline_rayon(bencher: Bencher, n: usize) {
    let mut data = vec![5; n];

    bencher.bench_local(|| {
        black_box(&mut data).par_iter_mut().for_each(|x| *x *= 2);
    });
}

/// Get baseline numbers for parallel iterator created through rayon
#[divan::bench(
    args = [1_000_000, 10_000_000, 100_000_000]
)]
fn slice_baseline_access(bencher: Bencher, n: usize) {
    let mut data = vec![5; n];

    bencher.bench_local(|| {
        let access = black_box(data.as_mut_slice()).into_par_access();
        create_par_iter(access).for_each(|x| *x *= 2);
    });
}

/// Get baseline numbers for "redundantly indexed" access.
/// Ideally this should compile down to basically the same as
/// the standard Rayon parallel iterator.
#[divan::bench(
    args = [1_000_000, 10_000_000, 100_000_000]
)]
fn slice_redundantly_indexed_access(bencher: Bencher, n: usize) {
    let mut data = vec![5; n];

    bencher.bench_local(|| {
        let access = black_box(data.as_mut_slice()).into_par_access();
        let indices = 0..n;
        let access = narrow_access_to_indices(access, &indices);
        create_par_iter(access).for_each(|x| *x *= 2);
    });
}

#[divan::bench(
args = [1_000_000, 10_000_000, 100_000_000]
)]
fn slice_subset_indexed_access(bencher: Bencher, n: usize) {
    let mut data = vec![5; n];

    let indices: Vec<_> = (0 .. n).step_by(2).collect();
    let indices = CheckedIndexList::from_hashable_indices(indices).unwrap();

    bencher.bench_local(|| {
        let access = black_box(data.as_mut_slice()).into_par_access();
        let access = narrow_access_to_indices(access, &indices);
        create_par_iter(access).for_each(|x| *x *= 2);
    });
}

/// Perform unsafe indexing through an access for iterating over a subset of indices.
#[divan::bench(
args = [1_000_000, 10_000_000, 100_000_000]
)]
fn slice_subset_unsafe_access(bencher: Bencher, n: usize) {
    let mut data = vec![5; n];

    let indices: Vec<_> = (0 .. n).step_by(2).collect();

    bencher.bench_local(|| {
        let access = black_box(data.as_mut_slice()).into_par_access();
        let indices = black_box(&indices);

        indices
            .par_iter()
            .for_each(|idx| unsafe { *access.get_unsync_unchecked(*idx) *= 2 });
    });
}
