use crate::Configuration;
use crate::grid::{Grid, Tile};
use crate::mst::DisjointSet;
use crate::room::{Room, RoomGraph};
use crate::vec;

use rand::Rng;
use rand::distr::Distribution;
use rand::seq::SliceRandom;

/// Uses the Disjoint Set structure to construct a maze in a room in a similar fassion to
/// [this article](https://en.wikipedia.org/wiki/Maze_generation_algorithm#Iterative_randomized_Kruskal's_algorithm_(with_sets)).
#[cfg(not(tarpaulin_include))]
pub fn place_maze<R: Rng>(rng: &mut R, room: &Room, grid: &mut Grid) {
    let northwest_corner = vec::to_index(vec::vec2(room.bounds.x, room.bounds.y), grid.width);
    let room_width = room.bounds.width as usize;
    let room_height = room.bounds.height as usize;

    // The maze generation algorithm works with 2x2 tiless. To cover the whole room we need the
    // following maze dimensions.
    let maze_width = (room_width >> 1) + (room_width & 1);
    let maze_height = (room_height >> 1) + (room_height & 1);

    let mut edges: Vec<(usize, usize)> =
        Vec::with_capacity((2 * maze_width * maze_height) - maze_width - maze_height);
    for column in 0..maze_width - 1 {
        for row in 0..maze_height - 1 {
            let northwest_tile = column + row * maze_width;
            edges.push((northwest_tile, northwest_tile + 1));
            edges.push((northwest_tile, northwest_tile + maze_width));
        }
    }
    for column in 0..maze_width - 1 {
        let west_tile = column + (maze_height - 1) * maze_width;
        edges.push((west_tile, west_tile + 1));
    }
    for row in 0..maze_height - 1 {
        let north_tile = (maze_width - 1) + row * maze_width;
        edges.push((north_tile, north_tile + maze_width));
    }
    edges.shuffle(rng);
    let tile_count = maze_width * maze_height;
    let mut disjoint_set = DisjointSet::new(tile_count);

    // Since the rooms are already empty, we need the edges between tiles which would place walls
    // i.e. the edges which are not part of the spanning tree.

    let mut current_edge = 0;
    let mut added_edges = 1;
    while current_edge < edges.len() && added_edges < tile_count {
        let tile_a = edges[current_edge].0;
        let tile_b = edges[current_edge].1;
        if disjoint_set.find_set(tile_a) == disjoint_set.find_set(tile_b) {
            current_edge += 1;
            continue;
        }
        added_edges += 1;
        edges.swap_remove(current_edge);
        disjoint_set.union_sets(tile_a, tile_b);
    }

    // Add fake edges to make walls on the south and east wall of the room if its dimensions are
    // divisible by 2. (Removes an ugly artefact in this case.)
    if room_height & 1 == 0 {
        for column in 0..maze_width {
            let tile = column + (maze_height - 1) * maze_width;
            edges.push((tile, tile + 2));
        }
    }
    if room_width & 1 == 0 {
        for row in 0..maze_height {
            let tile = (maze_width - 1) + row * maze_width;
            edges.push((tile, tile + 1));
        }
    }

    use Tile::*;
    // Place the tiles which are always walls.
    for column in 0..maze_width {
        for row in 0..maze_height {
            let tile_index =
                northwest_corner + ((column << 1) + 1) + ((row << 1) * grid.width + grid.width);
            if matches!(grid.tiles[tile_index], Room) {
                grid.tiles[tile_index] = Wall;
            }
        }
    }

    let mut nw_maze_subtile;
    let mut ne_maze_subtile;
    let mut sw_maze_subtile;
    let mut se_maze_subtile;
    for edge in edges {
        let column = edge.0 % maze_width;
        let row = edge.0 / maze_width;
        nw_maze_subtile = northwest_corner + (column << 1) + ((row << 1) * grid.width);
        ne_maze_subtile = nw_maze_subtile + 1;
        sw_maze_subtile = nw_maze_subtile + grid.width;
        se_maze_subtile = ne_maze_subtile + grid.width;

        // Check the direction in which we must place a wall. Then, check whether by placing a wall
        // we'll block a doorway. If we are not going to block a doorway, put the wall.
        if edge.1 - edge.0 == 1 {
            if grid.tiles[ne_maze_subtile] == Room
                && !(grid.tiles[se_maze_subtile] == Doorway
                    && matches!(grid.tiles[sw_maze_subtile], Wall | Blocker))
            {
                grid.tiles[ne_maze_subtile] = Wall;
            }
        } else if grid.tiles[sw_maze_subtile] == Room
            && !(grid.tiles[se_maze_subtile] == Doorway
                && matches!(grid.tiles[ne_maze_subtile], Wall | Blocker))
        {
            grid.tiles[sw_maze_subtile] = Wall;
        }
    }
}

#[cfg(not(tarpaulin_include))]
pub fn make_mazes<R: Rng>(
    rng: &mut R,
    configuration: &Configuration,
    grid: &mut Grid,
    room_graph: &RoomGraph,
) {
    let uniform = rand::distr::Uniform::new(0.0, 1.0).expect("Uniform range should be ok.");

    for room in &room_graph.rooms {
        if room.bounds.width as usize >= configuration.min_maze_dimension
            && room.bounds.height as usize >= configuration.min_maze_dimension
            && uniform.sample(rng) < configuration.maze_chance
        {
            place_maze(rng, room, grid);
        }
    }
}
