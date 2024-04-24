use paradis_core::{IntoParAccess, LinearParAccess};

pub fn create_iter<IntoAccess>(access: IntoAccess) -> AccessIterator<IntoAccess::Access>
where
    IntoAccess: IntoParAccess<usize>,
    IntoAccess::Access: LinearParAccess,
{
    let access = access.into_par_access();
    AccessIterator {
        next_idx: 0,
        len: access.len(),
        access,
    }
}

#[derive(Debug)]
pub struct AccessIterator<Access> {
    access: Access,
    next_idx: usize,
    // We cache the length to make sure that the compiler sees that it does not change
    len: usize,
}

impl<Access> Iterator for AccessIterator<Access>
where
    Access: LinearParAccess,
{
    type Item = Access::Record;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_idx < self.len {
            let item = unsafe { self.access.get_unsync_unchecked(self.next_idx) };
            self.next_idx += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<Access: LinearParAccess> ExactSizeIterator for AccessIterator<Access> {
    fn len(&self) -> usize {
        self.len
    }
}
