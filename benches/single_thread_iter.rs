use divan::Bencher;
use paradis::iter::create_iter;
use paradis::unique::narrow_access_to_indices;
use paradis_core::IntoParAccess;
use std::hint::black_box;

fn main() {
    divan::main()
}

#[divan::bench(
    args = [1_000_000]
)]
fn slice_baseline_std(bencher: Bencher, n: usize) {
    let mut data = black_box(vec![5; n]);
    let factor = black_box(2);

    bencher.bench_local(|| {
        for elem in black_box(&mut data) {
            *elem *= factor;
        }
    })
}

#[divan::bench(
args = [1_000_000]
)]
fn slice_baseline_access(bencher: Bencher, n: usize) {
    let mut data = black_box(vec![5; n]);
    let factor = black_box(2);

    bencher.bench_local(|| {
        for elem in create_iter(black_box(&mut data).into_par_access()) {
            *elem *= factor;
        }
    })
}

#[divan::bench(
args = [1_000_000]
)]
fn slice_redundantly_indexed_access(bencher: Bencher, n: usize) {
    let mut data = black_box(vec![5; n]);
    let factor = black_box(2);

    bencher.bench_local(|| {
        let indices = 0..n;
        let access = black_box(&mut data).into_par_access();
        let access = narrow_access_to_indices(access, &indices);
        for elem in create_iter(access) {
            *elem *= factor;
        }
    })
}
