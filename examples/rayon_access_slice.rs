use paradis::index::{narrow_access, IndexList};
use paradis::rayon::create_par_iter;
use rayon::iter::ParallelIterator;

fn main() {
    example_with_range();
    example_with_checked_indices();
    example_with_checked_indices_u32();
}

fn example_with_range() {
    let mut data = vec![1.0; 10000];
    let range = 5..data.len();
    let access = narrow_access(data.as_mut_slice(), &range).unwrap();

    create_par_iter(access).for_each(|x| *x *= 2.0);

    assert!(data[5..].iter().all(|&x| x == 2.0));
    assert!(data[..5].iter().all(|&x| x == 1.0));
}

fn example_with_checked_indices() {
    let mut data = vec![1.0; 10000];
    let indices = vec![900, 5, 10, 400, 1000, 100, 200]
        .check_unique()
        .expect("All indices unique");

    let access = narrow_access(data.as_mut_slice(), &indices).expect("Indices are in bounds");
    create_par_iter(access).for_each(|x| *x *= 2.0);

    for (idx, elem) in data.into_iter().enumerate() {
        if indices.get_inner().contains(&idx) {
            assert_eq!(elem, 2.0);
        } else {
            assert_eq!(elem, 1.0);
        }
    }
}

fn example_with_checked_indices_u32() {
    let mut data = vec![1.0; 10000];
    let index_data: &Vec<u32> = &vec![900, 5, 10, 400, 1000, 100, 200];
    let indices = index_data
        .check_unique()
        .expect("All indices unique")
        .index_cast();

    let access = narrow_access(data.as_mut_slice(), &indices).expect("indices must be unique");
    create_par_iter(access).for_each(|x| *x *= 2.0);

    for (idx, elem) in data.into_iter().enumerate() {
        if index_data.contains(&(idx as u32)) {
            assert_eq!(elem, 2.0);
        } else {
            assert_eq!(elem, 1.0);
        }
    }
}
