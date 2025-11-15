use crate::room::{Doorway, Room};
use crate::vec::vec2u;
use raylib::math::Rectangle;

// Shorthands:
pub fn room(x: usize, y: usize, w: usize, h: usize) -> Room {
    Room {
        bounds: Rectangle::new(x as f32, y as f32, w as f32, h as f32),
    }
}
pub fn doorway(x: usize, y: usize, room_index: usize) -> Doorway {
    Doorway {
        room_index,
        position: vec2u(x, y),
    }
}
pub fn doorwayp(x: usize, y: usize) -> Doorway {
    Doorway {
        room_index: 0,
        position: vec2u(x, y),
    }
}

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
