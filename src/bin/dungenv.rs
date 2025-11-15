use dungen::Configuration;
use dungen::grid::{Grid, make_grid};
use dungen::mst::pick_corridors;
use dungen::room::{Doorway, Room, RoomGraph, generate_rooms};
use dungen::triangulation::triangulate;
use dungen::vec::{vec2, vec2u};

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

use raylib::prelude::*;
use raylib_imgui::RaylibGui;

const CONTROLS: &str = "\
[I, J, K, L] - up, left, down, right;
[U, O] - zoom in / out
[C] - reset position & zoom
[R] - generate new dungeon
[ESC] - exit
";

const MAX_ROOM_COUNT: usize = 2_000;
const MAX_MAP_DIMENSIONS: usize = 512;

#[cfg(not(tarpaulin_include))]
fn draw_rooms(scale: f32, offset: Vector2, draw_handle: &mut impl RaylibDraw, rooms: &[Room]) {
    for room in rooms {
        let room_corner = offset + vec2(room.bounds.x, room.bounds.y) * scale;
        let room_dimensions = vec2(room.bounds.width, room.bounds.height) * scale;
        draw_handle.draw_rectangle_v(room_corner, room_dimensions, Color::PURPLE);
    }
}

#[cfg(not(tarpaulin_include))]
fn draw_doorways(
    scale: f32,
    offset: Vector2,
    draw_handle: &mut impl RaylibDraw,
    doorways: &[Doorway],
) {
    for doorway in doorways {
        draw_handle.draw_rectangle_v(
            offset + doorway.position * scale,
            vec2(scale, scale),
            Color::BLUE,
        );
    }
}

#[cfg(not(tarpaulin_include))]
fn draw_edges(
    scale: f32,
    offset: Vector2,
    draw_handle: &mut impl RaylibDraw,
    doorways: &[Doorway],
    edges: &[(usize, usize)],
) {
    for edge in edges {
        draw_handle.draw_line_v(
            offset + doorways[edge.0].position * scale + scale / 2.0,
            offset + doorways[edge.1].position * scale + scale / 2.0,
            Color::LIME,
        );
    }
}

#[cfg(not(tarpaulin_include))]
fn draw_graph(
    scale: f32,
    offset: Vector2,
    grid_dimensions: Vector2,
    draw_handle: &mut impl RaylibDraw,
    graph: &RoomGraph,
) {
    draw_handle.draw_rectangle_v(offset, grid_dimensions * scale, Color::GRAY);

    draw_rooms(scale, offset, draw_handle, &graph.rooms);
    draw_doorways(scale, offset, draw_handle, &graph.doorways);
    draw_edges(scale, offset, draw_handle, &graph.doorways, &graph.edges);
}

#[cfg(not(tarpaulin_include))]
fn draw_grid(grid: &Grid, draw_handle: &mut impl RaylibDraw) {
    use dungen::grid::Tile::*;
    draw_handle.clear_background(Color::BROWN);
    for tile_index in 0..grid.tiles.len() {
        let x = (tile_index % grid.width) as i32;
        let y = (tile_index / grid.width) as i32;
        if matches!(grid.tiles[tile_index], Room | Corridor | Doorway) {
            draw_handle.draw_pixel(x, y, Color::YELLOW);
        }
        // if matches!(grid.tiles[tile_index], Blocker) {
        //     draw_handle.draw_pixel(x, y, Color::PURPLE);
        // }
        // if matches!(grid.tiles[tile_index], CorridorNeighbor) {
        //     draw_handle.draw_pixel(x, y, Color::YELLOWGREEN);
        // }
    }
}

enum Request {
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

enum Result {
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

struct Generator {
    requests: Sender<Request>,
    results: Receiver<Result>,
    handle: JoinHandle<()>,
}

#[cfg(not(tarpaulin_include))]
fn make_generator() -> Generator {
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

enum DrawOption {
    Grid,
    Corridors,
    Triangulation,
}

#[cfg(not(tarpaulin_include))]
fn main() {
    // ============================== Library variables
    let (mut rl, thread) = raylib::init().size(1280, 720).title("Dungeon").build();
    let mut gui = RaylibGui::new(&mut rl, &thread);
    // ==============================

    // ============================== Configuration variables
    let mut grid_width: usize = 100;
    let mut grid_height: usize = 100;
    let mut grid_dimensions = vec2u(grid_width, grid_height);
    let mut target_room_count: usize = 30;
    let mut reintroduced_corridor_density: f32 = 0.5;
    let mut configuration = Configuration::default();
    // ============================== Configuration variables

    // ============================== State variables
    let mut source_rectangle = Rectangle::new(
        0.0,
        (MAX_MAP_DIMENSIONS - grid_height) as f32,
        grid_width as f32,
        -(grid_height as f32),
    );
    let mut render_texture = match rl.load_render_texture(
        &thread,
        MAX_MAP_DIMENSIONS as u32,
        MAX_MAP_DIMENSIONS as u32,
    ) {
        Ok(value) => value,
        _ => return,
    };
    let mut generating = false;
    let generator = make_generator();
    // Generate a grid that won't take much time before the start of the application.
    let request = Request::New {
        configuration: configuration.clone(),
        grid_dimensions,
        target_room_count: 20,
    };
    if generator.requests.send(request).is_err() {
        return;
    }
    let (mut triangulation, mut corridors, mut grid) = if let Ok(Result::New {
        triangulation,
        corridors,
        grid,
    }) = generator.results.recv()
    {
        (triangulation, corridors, grid)
    } else {
        return;
    };
    rl.draw_texture_mode(&thread, &mut render_texture, |mut handle| {
        draw_grid(&grid, &mut handle);
    });
    let mut dimensions_changed: bool = false;
    let mut draw_option: DrawOption = DrawOption::Grid;
    // ============================== State variables

    // ============================== Progress variables
    let dot_delay: f32 = 0.75;
    let mut dot_delay_timer: f32 = 0.0;
    let max_dot_count: usize = 3;
    let mut current_dot_mask: usize = 0;
    let message = "generating...";
    // ============================== Progress variables

    // ============================== Movement variables
    let mut scale: f32 = 5.0;
    let speed: f32 = 20.0;
    let view_center: Vector2 = vec2(1280.0 * 3.0 / 4.0, 720.0 / 2.0);
    let mut offset: Vector2 = view_center - grid_dimensions * scale / 2.0;
    // ============================== Movement variables

    while !rl.window_should_close() {
        let ui = gui.begin(&mut rl);

        // ============================== User Interface
        ui.window("Configuration")
            .size([600.0, 500.0], imgui::Condition::Always)
            .position([20.0, 20.0], imgui::Condition::Always)
            .resizable(false)
            .movable(false)
            .build(|| {
                { // ============================== min_room_dimension
                    ui.slider(
                        "Min Room Dimensions",
                        5,
                        100,
                        &mut configuration.min_room_dimension,
                    );
                    if ui.is_item_hovered() {
                        ui.tooltip_text(
                            "Minimum tile length of a room. Valid for both width and height.",
                        );
                    }
                } // ============================== min_room_dimension


                { // ============================== max_room_dimension
                    configuration.max_room_dimension = configuration
                        .max_room_dimension
                        .max(configuration.min_room_dimension);
                    ui.slider(
                        "Max Room Dimensions",
                        configuration.min_room_dimension,
                        100,
                        &mut configuration.max_room_dimension,
                    );
                    if ui.is_item_hovered() {
                        ui.tooltip_text(
                            "Maximum tile length of a room. Must be greater than or equal to the minimum."
                        );
                    }
                } // ============================== max_room_dimension


                { // ============================== min_padding
                    ui.slider("Min padding", 3, 20, &mut configuration.min_padding);
                    if ui.is_item_hovered() {
                        ui.tooltip_text(
                            "The minimum distance between rooms and the map border to guarantee that doorways \
                             are accessible.");
                    }
                } // ============================== min_padding


                { // ============================== doorway_offset
                    configuration.doorway_offset = configuration
                        .doorway_offset
                        .min(configuration.min_room_dimension >> 1);
                    ui.slider(
                        "Doorway offset",
                        1,
                        configuration.min_room_dimension >> 1,
                        &mut configuration.doorway_offset,
                    );
                    if ui.is_item_hovered() {
                        ui.tooltip_text("Offset from the edges of the room. Aesthetic option.");
                    }
                } // ============================== doorway_offset


                { // ============================== max_fail_count
                    ui.slider("Max Fail Count", 1, 200, &mut configuration.max_fail_count);
                    if ui.slider(
                        "Corridor Density",
                        0.0,
                        1.0,
                        &mut reintroduced_corridor_density,
                    ) {
                        configuration.reintroduced_corridor_density =
                            ((reintroduced_corridor_density * 1000.0) as usize, 1000);
                    }
                    if ui.is_item_hovered() {
                        ui.tooltip_text(
                            "What percentage of edges from the triangulation on average \
                             should be reintroduced as corridors.");
                    }
                } // ============================== max_fail_count

                ui.spacing();

                { // ============================== corridor costs
                    ui.slider("Corridor Cost", 1, 40, &mut configuration.corridor_cost);
                    if ui.is_item_hovered() {
                        ui.tooltip_text(
                            "Cost for the A* algorithm when we go through an already \
                             placed corridor. The relationship between this value and \
                             the other two costs determines the shape of the corridors.");
                    }
                    ui.slider("Straight Cost", 1, 40, &mut configuration.straight_cost);
                    if ui.is_item_hovered() {
                        ui.tooltip_text(
                            "Cost for the A* algorithm when we go to a tile which is in the same \
                             direction (horizontal or vertical) from which we came to the current \
                             tile. When lower than the standard cost makes the corridors straight \
                             hence the name.");
                    }
                    ui.slider("Standard Cost", 1, 40, &mut configuration.standard_cost);
                    if ui.is_item_hovered() {
                        ui.tooltip_text(
                            "Default cost for the A* algorithm. The corridors can move \
                             only horizontally or vertically.");
                    }
                } // ============================== corridor costs

                ui.spacing();

                { // ============================== grid dimensions
                    dimensions_changed |= ui.slider(
                        "Grid Width",
                        configuration.min_room_dimension + configuration.min_padding * 2,
                        MAX_MAP_DIMENSIONS,
                        &mut grid_width,
                    );
                    dimensions_changed |= ui.slider(
                        "Grid Height",
                        configuration.min_room_dimension + configuration.min_padding * 2,
                        MAX_MAP_DIMENSIONS,
                        &mut grid_height,
                    );
                } // ============================== grid dimensions

                let max_room_count = (grid_height * grid_width).min(MAX_ROOM_COUNT);
                target_room_count = target_room_count.min(max_room_count);
                ui.slider(
                    "Target Room Count",
                    1,
                    max_room_count,
                    &mut target_room_count,
                );

                ui.label_text("Controls", CONTROLS);

                let token = ui.begin_disabled(generating);
                if ui.button("Regenerate") {
                    generating = true;
                    if generator.requests.send(Request::New {
                        configuration: configuration.clone(),
                        grid_dimensions: vec2u(grid_width, grid_height),
                        target_room_count
                    }).is_err() {
                        return;
                    };
                }

                if ui.button("Regenerate Corridors") {
                    generating = true;
                    if generator.requests.send(Request::Corridors {
                        configuration: configuration.clone(),
                        grid_dimensions: vec2u(grid_width, grid_height),
                        triangulation: triangulation.clone()
                    }).is_err() {
                        return;
                    };
                }
                token.end();

                use DrawOption::*;
                ui.columns(3, "Views", false);
                if ui.button("Grid") {
                    draw_option = Grid;
                }
                ui.next_column();
                if ui.button("Corridors") {
                    draw_option = Corridors;
                }
                ui.next_column();
                if ui.button("Triangulation") {
                    draw_option = Triangulation;
                }
                ui.next_column();
                ui.columns(1, "", false);
            });
        // ============================== User Interface

        // Delta time.
        let dt = rl.get_frame_time();

        // ============================== Input
        {
            if !generating && rl.is_key_pressed(KeyboardKey::KEY_R) {
                generating = true;
                if generator
                    .requests
                    .send(Request::New {
                        configuration: configuration.clone(),
                        grid_dimensions: vec2u(grid_width, grid_height),
                        target_room_count,
                    })
                    .is_err()
                {
                    break;
                };
            }

            if rl.is_key_pressed(KeyboardKey::KEY_C) {
                scale = 5.0;
                offset = view_center - grid_dimensions * scale / 2.0;
            }

            if rl.is_key_down(KeyboardKey::KEY_U) {
                let previous_scale = scale;
                scale += 10.0 * dt;
                offset = (offset - view_center) * scale / previous_scale + view_center;
            }
            if rl.is_key_down(KeyboardKey::KEY_O) {
                let previous_scale = scale;
                scale -= 10.0 * dt;
                if scale < 2.0 {
                    scale = 2.0;
                }
                offset = (offset - view_center) * scale / previous_scale + view_center;
            }

            if rl.is_key_down(KeyboardKey::KEY_I) {
                offset.y += scale * speed * dt;
            }
            if rl.is_key_down(KeyboardKey::KEY_K) {
                offset.y -= scale * speed * dt;
            }
            if rl.is_key_down(KeyboardKey::KEY_J) {
                offset.x += scale * speed * dt;
            }
            if rl.is_key_down(KeyboardKey::KEY_L) {
                offset.x -= scale * speed * dt;
            }
        }
        // ============================== Input

        //
        match generator.results.try_recv() {
            Ok(result) => {
                match result {
                    Result::New {
                        triangulation: new_triangulation,
                        corridors: new_corridors,
                        grid: new_grid,
                    } => {
                        if dimensions_changed {
                            offset = view_center - grid_dimensions * scale / 2.0;
                            grid_dimensions = vec2u(grid_width, grid_height);
                            source_rectangle = Rectangle::new(
                                0.0,
                                MAX_MAP_DIMENSIONS as f32 - grid_dimensions.y,
                                grid_dimensions.x,
                                -grid_dimensions.y,
                            );
                            dimensions_changed = false;
                        }
                        triangulation = new_triangulation;
                        corridors = new_corridors;
                        grid = new_grid;
                        rl.draw_texture_mode(&thread, &mut render_texture, |mut handle| {
                            draw_grid(&grid, &mut handle);
                        });
                    }
                    Result::Corridors {
                        corridors: new_corridors,
                        grid: new_grid,
                    } => {
                        corridors = new_corridors;
                        grid = new_grid;
                        rl.draw_texture_mode(&thread, &mut render_texture, |mut handle| {
                            draw_grid(&grid, &mut handle);
                        });
                    }
                }
                generating = false;
            }
            Err(error) => {
                if error == mpsc::TryRecvError::Disconnected {
                    break;
                }
            }
        }

        let mut draw_handle = rl.begin_drawing(&thread);

        draw_handle.clear_background(Color::BLACK);

        use DrawOption::*;
        match draw_option {
            Grid => {
                let destination_rectangle = Rectangle::new(
                    offset.x,
                    offset.y,
                    grid_dimensions.x * scale,
                    grid_dimensions.y * scale,
                );
                draw_handle.draw_texture_pro(
                    &render_texture,
                    source_rectangle,
                    destination_rectangle,
                    vec2(0.0, 0.0),
                    0.0,
                    Color::WHITE,
                );
            }
            Corridors => draw_graph(scale, offset, grid_dimensions, &mut draw_handle, &corridors),
            Triangulation => draw_graph(
                scale,
                offset,
                grid_dimensions,
                &mut draw_handle,
                &triangulation,
            ),
        };

        if generating {
            dot_delay_timer += dt;
            if dot_delay_timer >= dot_delay {
                dot_delay_timer = 0.0;
                current_dot_mask += 1;
                current_dot_mask %= max_dot_count;
            }
            draw_handle.draw_text(
                &message[..message.len() - max_dot_count + current_dot_mask + 1],
                20,
                720 - 40,
                20,
                Color::WHITE,
            );
        } else {
            dot_delay_timer = 0.0;
            current_dot_mask = 0;
        }

        gui.end();
    }

    let Generator {
        requests,
        results,
        handle,
    } = generator;
    drop((requests, results));
    let _ = handle.join();
}
