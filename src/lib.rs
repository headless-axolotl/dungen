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
    phantom: PhantomData<()>,
}

impl Configuration {
    #[cfg(not(tarpaulin_include))]
    pub fn new(
        min_room_dimension: usize,
        max_room_dimension: usize,
        min_padding: usize,
        doorway_offset: usize,
        max_fail_count: usize,
        reintroduced_corridor_density: (usize, usize),
    ) -> Self {
        assert!(
            min_room_dimension >= 5,
            "The minimum room size must be greater than 4."
        );
        assert!(
            min_room_dimension <= max_room_dimension,
            "The maximum room size must be greater or equal to \
            the minimum room size."
        );
        assert!(
            min_padding > 2,
            "The minimum padding must be greater than or equal to 3 \
            to leave enough space for corridors."
        );
        assert!(
            doorway_offset > 0,
            "The doorway offset must be greater than 0 because \
            the doorways should not be in the corners of the rooms."
        );
        assert!(
            reintroduced_corridor_density.0 <= reintroduced_corridor_density.1
                && reintroduced_corridor_density.1 >= 1,
        );

        Self {
            min_room_dimension,
            max_room_dimension,
            min_padding,
            doorway_offset,
            max_fail_count,
            reintroduced_corridor_density,
            phantom: PhantomData,
        }
    }
}

