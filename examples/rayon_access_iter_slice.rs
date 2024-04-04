use paradis::rayon::linear_unsync_access_par_iter;
use paradis::{UniqueIndices, IntoUnsyncAccess};
use rayon::iter::ParallelIterator;

fn main() {
    let mut data = vec![1.0; 10000];
    let range = 5..data.len();
    let access = range.combine_with_access(data.as_mut_slice().into_unsync_access());
    linear_unsync_access_par_iter(access)
        .for_each(|x| *x *= 2.0);
    assert!(data[5..].iter().all(|&x| x == 2.0));
    assert!(data[..5].iter().all(|&x| x == 1.0));
}
