use paradis::rayon::create_par_iter;
use paradis::unique::{narrow_access_to_indices, CheckedIndexList};
use rayon::iter::ParallelIterator;

fn main() {
    example_with_range();
    example_with_checked_indices();
}

fn example_with_range() {
    let mut data = vec![1.0; 10000];
    let range = 5..data.len();
    let access = narrow_access_to_indices(data.as_mut_slice(), &range).unwrap();

    create_par_iter(access).for_each(|x| *x *= 2.0);

    assert!(data[5..].iter().all(|&x| x == 2.0));
    assert!(data[..5].iter().all(|&x| x == 1.0));
}

fn example_with_checked_indices() {
    let mut data = vec![1.0; 10000];
    let indices = vec![900, 5, 10, 400, 1000, 100, 200];
    let checked_indices =
        CheckedIndexList::from_hashable_indices(indices.clone()).expect("All indices unique");

    let access = narrow_access_to_indices(data.as_mut_slice(), &checked_indices)
        .expect("Indices are in bounds");
    create_par_iter(access).for_each(|x| *x *= 2.0);

    for (idx, elem) in data.into_iter().enumerate() {
        if indices.contains(&idx) {
            assert_eq!(elem, 2.0);
        } else {
            assert_eq!(elem, 1.0);
        }
    }
}
