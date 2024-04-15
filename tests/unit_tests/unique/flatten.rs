use paradis::unique::combinators::{Concatenate, Flatten};

#[test]
fn test_concatenate() {
    assert_eq!(0.concatenate(1), (0, 1));
    assert_eq!(0.concatenate((1, 2)), (0, 1, 2));
    assert_eq!(0.concatenate((1, 2, 3)), (0, 1, 2, 3));
    assert_eq!(0.concatenate((1, 2, 3, 4)), (0, 1, 2, 3, 4));

    assert_eq!((0, 1).concatenate(2), (0, 1, 2));
    assert_eq!((0, 1).concatenate((2, 3)), (0, 1, 2, 3));
    assert_eq!((0, 1).concatenate((2, 3, 4)), (0, 1, 2, 3, 4));

    assert_eq!((0, 1, 2).concatenate(3), (0, 1, 2, 3));
    assert_eq!((0, 1, 2).concatenate((3, 4)), (0, 1, 2, 3, 4));
}

#[test]
fn flatten() {
    assert_eq!(0, 0.flatten());

    assert_eq!((0, 1), (0, 1).flatten());

    assert_eq!((0, 1, 2), (0, 1, 2).flatten());
    assert_eq!((0, 1, 2), ((0, 1), 2).flatten());
    assert_eq!((0, 1, 2), (0, (1, 2)).flatten());

    assert_eq!((0, 1, 2, 3), (0, 1, 2, 3).flatten());
    assert_eq!((0, 1, 2, 3), ((0, 1), 2, 3).flatten());
    assert_eq!((0, 1, 2, 3), (0, (1, 2), 3).flatten());
    assert_eq!((0, 1, 2, 3), (0, 1, (2, 3)).flatten());
    assert_eq!((0, 1, 2, 3), ((0, 1), (2, 3)).flatten());
    assert_eq!((0, 1, 2, 3), ((0, 1, 2), 3).flatten());
    assert_eq!((0, 1, 2, 3), (0, (1, 2, 3)).flatten());

    assert_eq!((0, 1, 2, 3, 4), (0, 1, 2, 3, 4).flatten());
    assert_eq!((0, 1, 2, 3, 4), ((0, 1), 2, 3, 4).flatten());
    assert_eq!((0, 1, 2, 3, 4), (0, (1, 2), 3, 4).flatten());
    assert_eq!((0, 1, 2, 3, 4), (0, 1, (2, 3), 4).flatten());
    assert_eq!((0, 1, 2, 3, 4), (0, 1, 2, (3, 4)).flatten());
    assert_eq!((0, 1, 2, 3, 4), ((0, 1), (2, 3), 4).flatten());
    assert_eq!((0, 1, 2, 3, 4), (0, (1, 2), (3, 4)).flatten());
    assert_eq!((0, 1, 2, 3, 4), ((0, 1), 2, (3, 4)).flatten());
    assert_eq!((0, 1, 2, 3, 4), ((0, 1, 2), 3, 4).flatten());
    assert_eq!((0, 1, 2, 3, 4), (0, (1, 2, 3), 4).flatten());
    assert_eq!((0, 1, 2, 3, 4), (0, 1, (2, 3, 4)).flatten());
    assert_eq!((0, 1, 2, 3, 4), ((0, 1, 2), (3, 4)).flatten());
    assert_eq!((0, 1, 2, 3, 4), ((0, 1), (2, 3, 4)).flatten());
    assert_eq!((0, 1, 2, 3, 4), ((0, 1, 2, 3), 4).flatten());
    assert_eq!((0, 1, 2, 3, 4), (0, (1, 2, 3, 4)).flatten());
}
