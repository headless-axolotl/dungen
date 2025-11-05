use std::ops::RangeInclusive;

/// For the purposes of testing the procedures I write will use structures which implement this
/// wrapper trait. During testing I can then use a mock which implements this traint and returns
/// specific values.
pub trait Rng {
    fn random_range(&mut self, range: RangeInclusive<usize>) -> usize;
}

impl<T> Rng for T
where
    T: rand::Rng,
{
    #[cfg(not(tarpaulin_include))]
    fn random_range(&mut self, range: RangeInclusive<usize>) -> usize {
        self.random_range(range)
    }
}
