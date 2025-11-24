//! 2D Dungeon generator.

pub mod a_star;
pub mod binary_heap;
pub mod grid;
pub mod maze;
pub mod mst;
pub mod rng;
pub mod room;
pub mod triangulation;
pub mod vec;

#[cfg(test)]
pub mod mock;

#[derive(Clone, Debug)]
pub struct Configuration {
    /// Minimum tile length of a room. Valid for both width and height.
    pub min_room_dimension: usize,
    /// Maximum tile length of a room. Must be greater than or equal to the minimum.
    pub max_room_dimension: usize,
    /// The minimum distance between rooms and the map border to guarantee that doorways are
    /// accessible.
    pub min_padding: usize,
    /// Offset from the edges of the room. Aesthetic option.
    pub doorway_offset: usize,
    /// The number of failed attempts to place a room before we abort the algorithm. The bigger the
    /// number the higher the likelyhood that the target room count is reached.
    pub max_fail_count: usize,
    /// What proportion of edges on average should be reintroduced as corridors i.e. (0) out of
    /// every (1).
    pub reintroduced_corridor_density: (usize, usize),
    /// Cost for the A* algorithm when we go through an already placed corridor. The relationship
    /// between this value and the other two costs determines the shape of the corridors.
    pub corridor_cost: usize,
    /// Cost for the A* algorithm when we go to a tile which is in the same direction (horizontal
    /// or vertical) from which we came to the current tile. When lower than the standard cost
    /// makes the corridors straight hence the name.
    pub straight_cost: usize,
    /// Default cost for the A* algorithm. The corridors can move only horizontally or vertically.
    pub standard_cost: usize,
    pub min_maze_dimension: usize,
    pub maze_chance: f32,
}

impl Configuration {
    pub fn is_valid(&self) -> bool {
        self.min_room_dimension >= 5
            && self.min_room_dimension <= self.max_room_dimension
            && self.min_padding >= 3
            && self.doorway_offset >= 1
            && self.reintroduced_corridor_density.0 <= self.reintroduced_corridor_density.1
            && self.reintroduced_corridor_density.1 >= 1
            && self.corridor_cost >= 1
            && self.straight_cost >= 1
            && self.standard_cost >= 1
            && self.min_maze_dimension >= 5
            && 0.0 <= self.maze_chance
            && self.maze_chance <= 1.0
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
            min_maze_dimension: 5,
            maze_chance: 0.1,
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
