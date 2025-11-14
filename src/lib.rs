//! 2D Dungeon generator.

pub mod a_star;
pub mod binary_heap;
pub mod grid;
pub mod mst;
pub mod rng;
pub mod room;
pub mod triangulation;
pub mod vec;

#[cfg(test)]
pub mod mock;

use std::marker::PhantomData;

pub struct Configuration {
    pub min_room_dimension: usize,
    pub max_room_dimension: usize,
    pub min_padding: usize,
    pub doorway_offset: usize,
    pub max_fail_count: usize,
    /// What proportion of edges on average should be reintroduced as corridors i.e. (0) out of
    /// every (1).
    pub reintroduced_corridor_density: (usize, usize),
    // The costs which of different types of corridors.
    pub corridor_cost: usize,
    pub straight_cost: usize,
    pub standard_cost: usize,
    phantom: PhantomData<()>,
}

impl Configuration {
    pub fn is_valid(&self) -> bool {
        if self.min_room_dimension < 5 { return false }
        if self.min_room_dimension > self.max_room_dimension { return false }
        if self.min_padding < 3 { return false }
        if self.doorway_offset < 1 { return false }
        if self.reintroduced_corridor_density.0 > self.reintroduced_corridor_density.1
            || self.reintroduced_corridor_density.1 < 1 {
            return false
        }
        if self.corridor_cost < 1 || self.straight_cost < 1 || self.standard_cost < 1 {
            return false
        }
        true
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            min_room_dimension: 5,
            max_room_dimension: 20,
            min_padding: 3,
            doorway_offset: 2,
            max_fail_count: 10,
            reintroduced_corridor_density: (1, 2),
            corridor_cost: 1,
            straight_cost: 2,
            standard_cost: 3,
            phantom: Default::default(),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn default_values_are_correct() {
        let config = super::Configuration::default();
        assert!(config.is_valid(), "Default configuration should be valid.");
    }
}
