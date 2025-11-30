use crate::Configuration;
use crate::a_star;
use crate::binary_heap::Heap;
use crate::room::{Dungeon, Edges};
use crate::vec::{self, Vector2};

use std::fmt::Write;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Blocker,
    Wall,
    Room,
    Doorway,
    Corridor,
    CorridorNeighbor,
    Empty,
}

/// Convert a character to a tile.
impl From<char> for Tile {
    fn from(value: char) -> Self {
        use Tile::*;
        match value {
            '%' => Blocker,
            '#' => Wall,
            '_' => Room,
            'd' => Doorway,
            'c' => Corridor,
            '@' => CorridorNeighbor,
            _ => Empty,
        }
    }
}

/// Convert a tile to a character.
impl From<Tile> for char {
    fn from(value: Tile) -> Self {
        use Tile::*;
        match value {
            Blocker => '%',
            Wall => '#',
            Room => '_',
            Doorway => 'd',
            Corridor => 'c',
            CorridorNeighbor => '@',
            Empty => '.',
        }
    }
}

#[derive(Debug)]
pub struct Grid {
    pub width: usize,
    pub tiles: Vec<Tile>,
}

/// Convert the grid into a string.
impl std::fmt::Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut index = 0;
        for _ in 0..self.tiles.len() / self.width {
            for _ in 0..self.width {
                f.write_char(self.tiles[index].into())?;
                index += 1;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

/// Convert a string into a grid. Assumes the string is a valid grid,
/// however, it does not error or panic if it isn't.
impl From<&str> for Grid {
    fn from(value: &str) -> Self {
        let mut height: usize = 0;
        let mut result = Grid {
            width: 0,
            tiles: vec![],
        };
        for line in value.lines() {
            for character in line.chars() {
                result.tiles.push(character.into());
            }
            height += 1;
        }
        result.width = result.tiles.len() / height;
        result
    }
}

/// Places a corridor in the grid. Surrounds the corridor with marker tiles
/// so that the search algorithm knows to avoid creating 2x2 corridor tile blocks.
/// When there is a turn in the corridor, places a blocking tile again to prevent
/// the pathfinding algorithm from creating 2x2 corridor tile blocks.
fn place_corridor(width: usize, tiles: &mut [Tile], path: &[usize]) {
    use Tile::*;

    #[inline]
    fn place_corridor_neighbor(index: usize, tiles: &mut [Tile]) {
        if matches!(tiles[index], CorridorNeighbor) {
            // There was a turn in the corridor.
            tiles[index] = Blocker;
        } else if matches!(tiles[index], Wall) {
            tiles[index] = CorridorNeighbor;
        } else if matches!(tiles[index], Room) {
            // Place a doorway marker tile inside the room to help the maze generation algorithm
            // procedure.
            tiles[index] = Doorway;
        }
    }

    // The first and the last tiles in the path are doorways.
    for &current in path {
        if !matches!(tiles[current], Corridor) {
            place_corridor_neighbor(current + 1, tiles);
            place_corridor_neighbor(current - 1, tiles);
            place_corridor_neighbor(current + width, tiles);
            place_corridor_neighbor(current - width, tiles);
        }

        if !matches!(tiles[current], Doorway) {
            tiles[current] = Corridor;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn try_place_corridors(
    configuration: &Configuration,
    dungeon: &Dungeon,
    corridors: &Edges,
    width: usize,
    tiles: &mut [Tile],
    open_set: &mut Heap<usize, usize>,
    g_scores: &mut Vec<usize>,
    parent: &mut Vec<usize>,
    path: &mut Vec<usize>,
) -> bool {
    use Tile::*;
    for edge in corridors {
        let position_a = vec::to_index(dungeon.doorways[edge.0].position, width);
        let position_b = vec::to_index(dungeon.doorways[edge.1].position, width);
        // Place doorways (which replace blocking tiles around the rooms) and create corridors.
        tiles[position_a] = Doorway;
        tiles[position_b] = Doorway;
        a_star::a_star(
            configuration,
            position_a,
            position_b,
            width,
            tiles,
            open_set,
            g_scores,
            parent,
            path,
        );
        if path.is_empty() {
            return false;
        }
        place_corridor(width, tiles, path);
    }
    true
}

/// Takes a room graph and creates the corresponding grid given the options in the configuration
/// structure. Uses the A* algorithm to carve corridors between the rooms, while ensuring that no
/// corridors make a 2x2 square (aesthetic choice).
///
/// If the corridor generation fails (which can happen if the configuration values for the
/// different costs are more extreme) it regenerates the corridor with the default values.
pub fn make_grid(
    configuration: &Configuration,
    grid_dimensions: Vector2,
    dungeon: &Dungeon,
    corridors: &Edges,
) -> Grid {
    use Tile::*;

    let grid_width = grid_dimensions.x as usize;
    let grid_height = grid_dimensions.y as usize;
    let mut tiles: Vec<Tile> = vec![Wall; grid_width * grid_height];

    // Create a one wide perimeter of blocking tiles around the grid.
    // Removes the need to check for edge cases when generating neighbors of a tile.
    for column in 0..grid_width {
        tiles[column] = Blocker; // north
        tiles[column + grid_width * (grid_height - 1)] = Blocker; // south
    }
    for row in 0..grid_height {
        tiles[row * grid_width] = Blocker; // west
        tiles[row * grid_width + grid_width - 1] = Blocker; // east
    }

    // Carve the rectangles the rooms occupy and place a one wide
    // perimeter of blocking tiles. Some of the blocking tiles will
    // be replaced by doorways in the following step. One can enter
    // a room only through a doorway.
    for room in &dungeon.rooms {
        let top_left_corner = room.bounds.x + room.bounds.y * grid_width;
        let room_width = room.bounds.width;
        let room_height = room.bounds.height;
        for row in 0..room_height {
            for column in 0..room_width {
                tiles[top_left_corner + row * grid_width + column] = Room;
            }
        }

        // Move the corner up and to the left.
        let top_left_corner = top_left_corner - grid_width - 1;
        for column in 0..room_width + 2 {
            tiles[top_left_corner + column] = Blocker; // north
            tiles[top_left_corner + column + grid_width * (room_height + 1)] = Blocker; // south
        }
        for row in 0..room_height + 2 {
            tiles[top_left_corner + row * grid_width] = Blocker; // west
            tiles[top_left_corner + row * grid_width + room_width + 1] = Blocker; // east
        }
    }

    let mut open_set: Heap<usize, usize> = Heap::with_capacity(tiles.len() / 4);
    let mut g_scores: Vec<usize> = vec![];
    let mut parent: Vec<usize> = vec![];
    let mut path: Vec<usize> = vec![];

    let mut tiles_clone = tiles.clone();
    if try_place_corridors(
        configuration,
        dungeon,
        corridors,
        grid_width,
        &mut tiles_clone,
        &mut open_set,
        &mut g_scores,
        &mut parent,
        &mut path,
    ) {
        Grid {
            width: grid_width,
            tiles: tiles_clone,
        }
    } else {
        try_place_corridors(
            &Default::default(),
            dungeon,
            corridors,
            grid_width,
            &mut tiles,
            &mut open_set,
            &mut g_scores,
            &mut parent,
            &mut path,
        );
        Grid {
            width: grid_width,
            tiles,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        mock::{doorway, room},
        vec::{to_index, vec2u},
    };

    #[test]
    fn tile_characters() {
        use Tile::*;
        let tiles = [
            Blocker,
            Wall,
            Room,
            Doorway,
            Corridor,
            CorridorNeighbor,
            Empty,
        ];
        for tile in &tiles {
            assert_eq!(
                *tile,
                <Tile>::from(<char>::from(*tile)),
                "Tile does not match char and vice versa."
            );
        }
    }

    #[test]
    fn grid_serialization_deserialization() {
        use Tile::*;
        let grid = Grid {
            width: 4,
            tiles: vec![
                Blocker,
                Wall,
                Room,
                Doorway,
                Doorway,
                Corridor,
                CorridorNeighbor,
                Empty,
            ],
        };

        let string: &str = "%#_d\ndc@.\n";
        assert_eq!(
            string,
            format!("{}", grid),
            "Grid does not display properly."
        );

        let result: Grid = string.into();
        assert!(
            grid.width == result.width && grid.tiles.len() == result.tiles.len(),
            "Parsed grid does not match size."
        );
        assert_eq!(
            grid.tiles, result.tiles,
            "Parsed grid does not match contents."
        );
    }

    #[test]
    fn corridor_placement() {
        // This grid tests that the place_corridor procedure does not override blockers, rooms and
        // doorways and already placed corridors. It tests whether the corridor neighbors are
        // placed appropriatelly and whether blockers are placed in spots where we are certain a
        // corridor should not be placed.
        let Grid { width, mut tiles } = Grid::from(
            "\
            %%%%%%%%%%%%%\n\
            %_%##@c@####%\n\
            %_d##@c@####%\n\
            %%%##@c@##%%%\n\
            %####@c@##d_%\n\
            %####@c@##%_%\n\
            %%%%%%%%%%%%%\n",
        );
        let path = &[
            to_index(vec2u(2, 2), width),
            to_index(vec2u(3, 2), width),
            to_index(vec2u(3, 3), width),
            to_index(vec2u(3, 4), width),
            to_index(vec2u(4, 4), width),
            to_index(vec2u(5, 4), width),
            to_index(vec2u(6, 4), width),
            to_index(vec2u(7, 4), width),
            to_index(vec2u(8, 4), width),
            to_index(vec2u(9, 4), width),
            to_index(vec2u(10, 4), width),
        ];

        place_corridor(width, &mut tiles, path);

        let correct_grid = Grid::from(
            "\
            %%%%%%%%%%%%%\n\
            %_%@#@c@####%\n\
            %ddc@@c@####%\n\
            %%%c%%c%@@%%%\n\
            %#@cccccccdd%\n\
            %##@@%c%@@%_%\n\
            %%%%%%%%%%%%%\n",
        );

        assert_eq!(
            &tiles, &correct_grid.tiles,
            "Corridor placement is incorrect."
        );
    }

    #[test]
    fn grid_generation() {
        // This test is mainly focused on the correct placement of rooms and doorways since A* and
        // corridor placement already have been tested.
        let configuration = Configuration::default();

        let dungeon = Dungeon {
            rooms: vec![
                room(3, 3, 5, 5),
                room(11, 3, 5, 5),
                room(3, 11, 5, 5),
                room(11, 11, 5, 5),
            ],
            doorways: vec![
                doorway(8, 5, 0),
                doorway(5, 8, 0),
                doorway(10, 5, 0),
                doorway(13, 8, 0),
                doorway(5, 10, 0),
                doorway(8, 13, 0),
                doorway(13, 10, 0),
                doorway(10, 13, 0),
            ],
        };
        let corridors = vec![(0, 2), (1, 4), (3, 6), (5, 7)];

        let grid_dimension = configuration.min_padding * 3 + configuration.min_room_dimension * 2;
        let grid_dimensions = vec2u(grid_dimension, grid_dimension);

        let grid = make_grid(&configuration, grid_dimensions, &dungeon, &corridors);
        let correct_grid = Grid::from(
            "\
            %%%%%%%%%%%%%%%%%%%\n\
            %#################%\n\
            %#%%%%%%%#%%%%%%%#%\n\
            %#%_____%#%_____%#%\n\
            %#%_____%@%_____%#%\n\
            %#%____ddcdd____%#%\n\
            %#%_____%@%_____%#%\n\
            %#%__d__%#%__d__%#%\n\
            %#%%%d%%%#%%%d%%%#%\n\
            %###@c@#####@c@###%\n\
            %#%%%d%%%#%%%d%%%#%\n\
            %#%__d__%#%__d__%#%\n\
            %#%_____%@%_____%#%\n\
            %#%____ddcdd____%#%\n\
            %#%_____%@%_____%#%\n\
            %#%_____%#%_____%#%\n\
            %#%%%%%%%#%%%%%%%#%\n\
            %#################%\n\
            %%%%%%%%%%%%%%%%%%%\n",
        );

        assert_eq!(grid.width, correct_grid.width, "Grids should match widths.");
        assert_eq!(
            &grid.tiles, &correct_grid.tiles,
            "Grids should match contents."
        );
    }
}
