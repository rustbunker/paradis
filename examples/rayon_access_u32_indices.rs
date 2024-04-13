use paradis::rayon::create_par_iter;
use paradis::unique::UniqueIndices;
use paradis::unique::{compose_access_with_indices, CheckedUniqueIndices};
use rayon::iter::ParallelIterator;

fn main() {
    example_with_checked_indices_u32();
}

fn example_with_checked_indices_u32() {
    let mut data = vec![1.0; 10000];
    let indices: Vec<u32> = vec![900, 5, 10, 400, 1000, 100, 200];
    let checked_indices = CheckedUniqueIndices::from_hashable_indices(indices.clone())
        .expect("All indices unique")
        .cast_index_type();

    let access = compose_access_with_indices(data.as_mut_slice(), &checked_indices);
    create_par_iter(access).for_each(|x| *x *= 2.0);

    for (idx, elem) in data.into_iter().enumerate() {
        if indices.contains(&(idx as u32)) {
            assert_eq!(elem, 2.0);
        } else {
            assert_eq!(elem, 1.0);
        }
    }
}
