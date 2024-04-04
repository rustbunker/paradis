use paradis::rayon::linear_unsync_access_par_iter;
use rayon::iter::ParallelIterator;

fn main() {
    let mut data = vec![1.0; 10000];
    // let range = 0..data.len();
    // TODO: update this example to work with an example of disjoint indices
    linear_unsync_access_par_iter(data.as_mut_slice()).for_each(|x| *x *= 2.0);
    assert!(data.iter().all(|&x| x == 2.0));
}
