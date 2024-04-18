use paradis::iter::create_iter;

#[test]
fn basic_iteration() {
    let mut data = vec![0, 1, 2, 3, 4, 5];
    let iter = create_iter(data.as_mut_slice());

    for elem in iter {
        *elem *= 2;
    }

    assert_eq!(data, vec![0, 2, 4, 6, 8, 10]);
}