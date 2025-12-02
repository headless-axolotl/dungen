use dungen::Configuration;
use dungen::grid::Grid;
use dungen::maze;
use dungen::room::{Dungeon, Edges};
use dungen::vec;

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

pub enum Request {
    New {
        configuration: Configuration,
        grid_dimensions: vec::Vector2,
        target_room_count: usize,
    },
    CorridorsAndMazes {
        configuration: Configuration,
        grid_dimensions: vec::Vector2,
        dungeon: Dungeon,
        triangulation: Edges,
    },
}

pub enum Result {
    New {
        dungeon: Dungeon,
        triangulation: Edges,
        corridors: Edges,
        grid: Grid,
    },
    Corridors {
        corridors: Edges,
        grid: Grid,
    },
}

pub struct Generator {
    pub requests: Sender<Request>,
    pub results: Receiver<Result>,
    pub handle: JoinHandle<()>,
}

#[cfg(not(tarpaulin_include))]
pub fn make_generator() -> Generator {
    use dungen::{
        grid::make_grid, mst::pick_corridors, room::generate_rooms, triangulation::triangulate,
    };

    let (requests, request_receiver) = mpsc::channel::<Request>();
    let (results_sender, results) = mpsc::channel::<Result>();

    let handle = std::thread::spawn(move || {
        let mut rng = rand::rng();
        #[allow(clippy::while_let_loop)]
        loop {
            let request = match request_receiver.recv() {
                Ok(value) => value,
                _ => break,
            };

            match request {
                Request::New {
                    configuration,
                    grid_dimensions,
                    target_room_count,
                } => {
                    let mut dungeon = generate_rooms(
                        &configuration,
                        grid_dimensions,
                        Some(target_room_count),
                        &mut rng,
                    );
                    let mut triangulation = triangulate(grid_dimensions, &mut dungeon);
                    let corridors =
                        pick_corridors(&configuration, &dungeon, &mut triangulation, &mut rng);
                    let mut grid = make_grid(&configuration, grid_dimensions, &dungeon, &corridors);
                    maze::make_mazes(&mut rng, &configuration, &mut grid, &dungeon);
                    if results_sender
                        .send(Result::New {
                            dungeon,
                            triangulation,
                            corridors,
                            grid,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
                Request::CorridorsAndMazes {
                    configuration,
                    grid_dimensions,
                    dungeon: rooms,
                    mut triangulation,
                } => {
                    let corridors =
                        pick_corridors(&configuration, &rooms, &mut triangulation, &mut rng);
                    let mut grid = make_grid(&configuration, grid_dimensions, &rooms, &corridors);
                    maze::make_mazes(&mut rng, &configuration, &mut grid, &rooms);
                    if results_sender
                        .send(Result::Corridors { corridors, grid })
                        .is_err()
                    {
                        break;
                    };
                }
            }
        }
    });

    Generator {
        requests,
        results,
        handle,
    }
}
