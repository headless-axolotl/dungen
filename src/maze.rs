#![allow(unused)]

use crate::grid::Grid;
use crate::Configuration;
use crate::room::RoomGraph;
use crate::mst::DisjointSet;

use rand::Rng;

pub fn make_mazes(
    configuration: &Configuration,
    grid: Grid,
    room_graph: &RoomGraph,
) -> Grid {
    grid
}

