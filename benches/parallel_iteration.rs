use divan::Bencher;
use paradis::rayon::create_par_iter;
use paradis::unique::{CheckedIndexList, narrow_access_to_indices};
use paradis_core::{IntoParAccess, ParAccess};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::hint::black_box;
use rayon::{ThreadPoolBuilder};

fn main() {
    divan::main()
}

fn warmup<F: FnMut()>(mut f: F) {
    for _ in 0 .. 10 {
        f();
    }
}

/// Helper macro to run a benchmark in a rayon thread pool with the prescribed number of threads
macro_rules! run_rayon_bench {
    ($bencher:expr, num_threads = $num_threads:expr, $closure:expr) => {
        {
            let thread_pool = ThreadPoolBuilder::new()
                .num_threads($num_threads)
                .build()
                .unwrap();
            let mut bench_fn = || {
                thread_pool.install($closure);
            };
            warmup(&mut bench_fn);
            $bencher.bench_local(bench_fn);
        }
    }
}

/// Get some baseline numbers for basic Rayon parallel iteration
#[divan::bench(
    args = [(10_000_000, 1), (10_000_000, 2), (10_000_000, 3), (10_000_000, 4), (10_000_000, 8)]
)]
fn slice_baseline_rayon(bencher: Bencher, (n, num_threads): (usize, usize)) {
    let mut data = black_box(vec![5; n]);
    let factor = black_box(2);

    run_rayon_bench!(bencher, num_threads = num_threads,
        || {
            black_box(&mut data)
                .par_iter_mut()
                .with_min_len(100000)
                .for_each(|x| *x *= factor);
        });
}

/// Get baseline numbers for parallel iterator created through rayon
#[divan::bench(
    args = [(10_000_000, 1), (10_000_000, 2), (10_000_000, 3), (10_000_000, 4), (10_000_000, 8)]
)]
fn slice_baseline_access(bencher: Bencher, (n, num_threads): (usize, usize)) {
    let mut data = black_box(vec![5; n]);
    let factor = black_box(2);

    run_rayon_bench!(bencher, num_threads = num_threads,
        || {
            let access = black_box(data.as_mut_slice()).into_par_access();
            create_par_iter(access)
                .with_min_len(100000)
                .for_each(|x| *x *= factor);
        });
}

/// Get baseline numbers for "redundantly indexed" access.
/// Ideally this should compile down to basically the same as
/// the standard Rayon parallel iterator.
#[divan::bench(
    args = [(10_000_000, 1), (10_000_000, 2), (10_000_000, 3), (10_000_000, 4), (10_000_000, 8)]
)]
fn slice_redundantly_indexed_access(bencher: Bencher, (n, num_threads): (usize, usize)) {
    let mut data = black_box(vec![5; n]);
    let factor = black_box(2);

    run_rayon_bench!(bencher, num_threads = num_threads,
        || {
            let access = black_box(data.as_mut_slice()).into_par_access();
            let indices = 0..n;
            let access = narrow_access_to_indices(access, &indices);
            create_par_iter(access)
                .with_min_len(10000)
                .for_each(|x| *x *= factor);
        });
}

#[divan::bench(
    args = [(10_000_000, 1), (10_000_000, 2), (10_000_000, 3), (10_000_000, 4), (10_000_000, 8)]
)]
fn slice_subset_indexed_access(bencher: Bencher, (n, num_threads): (usize, usize)) {
    let mut data = black_box(vec![5; n]);
    let factor = black_box(2);

    let indices: Vec<_> = (0 .. n).step_by(2).collect();
    let indices = CheckedIndexList::from_hashable_indices(indices).unwrap();

    run_rayon_bench!(bencher, num_threads = num_threads,
        || {
            let access = black_box(data.as_mut_slice()).into_par_access();
            let access = narrow_access_to_indices(access, &indices);
            create_par_iter(access)
                .with_min_len(10000)
                .for_each(|x| *x *= factor);
        });
}

/// Perform unsafe indexing through an access for iterating over a subset of indices.
#[divan::bench(
    args = [(10_000_000, 1), (10_000_000, 2), (10_000_000, 3), (10_000_000, 4), (10_000_000, 8)]
)]
fn slice_subset_unsafe_access(bencher: Bencher, (n, num_threads): (usize, usize)) {
    let mut data = black_box(vec![5; n]);
    let factor = black_box(2);

    let indices: Vec<_> = (0 .. n).step_by(2).collect();

    run_rayon_bench!(bencher, num_threads = num_threads,
        || {
            let access = black_box(data.as_mut_slice()).into_par_access();
            let indices = black_box(&indices);

            indices
                .par_iter()
                .with_min_len(10000)
                .for_each(|idx| unsafe { *access.get_unsync_unchecked(*idx) *= factor });
        });
}
