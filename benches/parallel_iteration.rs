use std::hint::black_box;
use divan::Bencher;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use paradis::rayon::create_par_iter;
use paradis::unique::narrow_access_to_indices;
use paradis_core::IntoParAccess;

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
        black_box(&mut data)
            .par_iter_mut()
            .for_each(|x| *x *= 2);
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
        create_par_iter(access)
            .for_each(|x| *x *= 2);
    });
}

/// Get baseline numbers for "redundantly indexed" access.
/// Ideally this should compile down to basically the same as
/// the standard Rayon parallel iterator.
#[divan::bench(
    args = [1_000_000, 10_000_000, 100_000_000]
)]
fn slice_indexed_access(bencher: Bencher, n: usize) {
    let mut data = vec![5; n];

    bencher.bench_local(|| {
        let access = black_box(data.as_mut_slice()).into_par_access();
        let indices = 0 .. n;
        let access = narrow_access_to_indices(access, &indices);
        create_par_iter(access)
            .for_each(|x| *x *= 2);
    });
}