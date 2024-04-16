use crate::unique::IndexList;
use crate::RecordIndex;

/// An index list consisting of an index repeated a finite number of times.
#[derive(Debug, Clone)]
pub struct Repeat<I> {
    value: I,
    times: usize,
}

impl<I> Repeat<I> {
    /// Construct an index list with 0 repetitions of the provided index value.
    pub fn value(value: I) -> Self {
        Self { value, times: 0 }
    }

    /// Construct a new index list where the current value is repeated the specified number of times.
    pub fn times(self, times: usize) -> Self {
        Self {
            value: self.value,
            times,
        }
    }
}

unsafe impl<I: RecordIndex> IndexList for Repeat<I> {
    type Index = I;

    unsafe fn get_unchecked(&self, _: usize) -> Self::Index {
        self.value
    }

    fn num_indices(&self) -> usize {
        self.times
    }
}
