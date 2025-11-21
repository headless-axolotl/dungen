use dungen::Configuration;
use dungen::grid::Grid;
use dungen::room::RoomGraph;

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

use raylib::math::Vector2;

pub enum Request {
    New {
        configuration: Configuration,
        grid_dimensions: Vector2,
        target_room_count: usize,
    },
    Corridors {
        configuration: Configuration,
        grid_dimensions: Vector2,
        triangulation: RoomGraph,
    },
}

pub enum Result {
    New {
        triangulation: RoomGraph,
        corridors: RoomGraph,
        grid: Grid,
    },
    Corridors {
        corridors: RoomGraph,
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
                    let rooms = generate_rooms(
                        &configuration,
                        grid_dimensions,
                        Some(target_room_count),
                        &mut rng,
                    );
                    let triangulation = triangulate(grid_dimensions, rooms);
                    let corridors = pick_corridors(&configuration, triangulation.clone(), &mut rng);
                    let grid = make_grid(&configuration, grid_dimensions, &corridors);
                    if results_sender
                        .send(Result::New {
                            triangulation,
                            corridors,
                            grid,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
                Request::Corridors {
                    configuration,
                    grid_dimensions,
                    triangulation,
                } => {
                    let corridors = pick_corridors(&configuration, triangulation, &mut rng);
                    let grid = make_grid(&configuration, grid_dimensions, &corridors);
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
