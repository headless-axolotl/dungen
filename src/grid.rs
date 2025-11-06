mod astar;

use crate::{Configuration, room::RoomGraph};

use raylib::math::Vector2;

#[derive(Clone, Debug)]
pub enum Tile {
    Blocker,
    Wall,
    Room,
    Doorway,
    Corridor,
    CorridorNeighbor,
    Empty,
}

#[derive(Debug)]
pub struct Grid {
    pub grid: Vec<Tile>,
}

pub fn make_grid(config: &Configuration, grid_dimensions: Vector2, room_graph: RoomGraph) -> Grid {
    use Tile::*;

    let width = grid_dimensions.x as usize;
    let height = grid_dimensions.y as usize;
    let grid: Vec<Tile> = vec![Wall; width * height];

    // Make grid outline

    // Carve rooms

    // Make room borders

    // Place doorways which participate in corridors

    // Carve corridors

    Grid { grid }
}
