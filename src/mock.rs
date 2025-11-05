pub struct MockMaxRng;
impl crate::rng::Rng for MockMaxRng {
    fn random_range(&mut self, range: std::ops::RangeInclusive<usize>) -> usize {
        range.max().unwrap()
    }
}

pub struct MockMinRng;
impl crate::rng::Rng for MockMinRng {
    fn random_range(&mut self, range: std::ops::RangeInclusive<usize>) -> usize {
        range.min().unwrap()
    }
}

pub struct MockRng {
    numbers: Vec<usize>,
    index: usize,
}

impl MockRng {
    pub fn new(numbers: Vec<usize>) -> Self {
        Self { numbers, index: 0 }
    }
}
impl crate::rng::Rng for MockRng {
    fn random_range(&mut self, _range: std::ops::RangeInclusive<usize>) -> usize {
        let result = self.numbers[self.index];
        self.index += 1;
        self.index %= self.numbers.len();
        result
    }
}
