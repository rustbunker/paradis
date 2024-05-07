use paradis_core::{BoundedParAccess, IntoParAccess, LinearParAccess, ParAccess};

#[test]
fn test_basic_access() {
    let slice = &mut [0, 1, 2, 3];
    let access = slice.into_par_access();

    assert_eq!(access.collection_len(), 4);
    assert_eq!(unsafe { access.get_unsync(0) }, &0);
    assert_eq!(unsafe { access.get_unsync(1) }, &1);
    assert_eq!(unsafe { access.get_unsync(2) }, &2);
    assert_eq!(unsafe { access.get_unsync(3) }, &3);

    let access2 = unsafe { access.clone_access() };
    assert_eq!(access2.collection_len(), 4);
    assert_eq!(unsafe { access2.get_unsync(0) }, &0);
    assert_eq!(unsafe { access2.get_unsync(1) }, &1);
    assert_eq!(unsafe { access2.get_unsync(2) }, &2);
    assert_eq!(unsafe { access2.get_unsync(3) }, &3);

    // Obtain mutable references to non-overlapping entries from two different accesses.
    {
        {
            let a: &mut u32 = unsafe { access.get_unsync(0) };
            let b: &mut u32 = unsafe { access2.get_unsync(1) };
            let c: &mut u32 = unsafe { access.get_unsync(2) };
            let d: &mut u32 = unsafe { access2.get_unsync(3) };

            *a = 4;
            *b = 5;
            *c = 6;
            *d = 7;
        }

        assert_eq!(unsafe { access2.get_unsync(0) }, &4);
        assert_eq!(unsafe { access.get_unsync(1) }, &5);
        assert_eq!(unsafe { access2.get_unsync(2) }, &6);
        assert_eq!(unsafe { access.get_unsync(3) }, &7);
    }
}
