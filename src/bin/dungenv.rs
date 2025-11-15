use dungen::Configuration;
use dungen::grid::make_grid;
use dungen::mst::pick_corridors;
use dungen::room::{Doorway, Room, RoomGraph, generate_rooms};
use dungen::triangulation::triangulate;
use dungen::vec::{vec2, vec2u};

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

#[cfg(not(tarpaulin_include))]
fn draw_rooms(scale: f32, offset: Vector2, draw_handle: &mut RaylibDrawHandle<'_>, rooms: &[Room]) {
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
    draw_handle: &mut RaylibDrawHandle<'_>,
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
    draw_handle: &mut RaylibDrawHandle<'_>,
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
    draw_handle: &mut RaylibDrawHandle<'_>,
    graph: &RoomGraph,
) {
    draw_handle.draw_rectangle_v(offset, grid_dimensions * scale, Color::GRAY);

    draw_rooms(scale, offset, draw_handle, &graph.rooms);
    draw_doorways(scale, offset, draw_handle, &graph.doorways);
    draw_edges(scale, offset, draw_handle, &graph.doorways, &graph.edges);
}

#[cfg(not(tarpaulin_include))]
fn draw_grid_to_image(
    configuration: &Configuration,
    grid_dimensions: Vector2,
    room_graph: &RoomGraph,
    image: &mut Image,
) {
    use dungen::grid::Tile::*;
    let grid = make_grid(configuration, grid_dimensions, room_graph);
    image.clear_background(Color::BROWN);
    for tile_index in 0..grid.tiles.len() {
        let x = (tile_index % grid.width) as i32;
        let y = (tile_index / grid.width) as i32;
        if matches!(grid.tiles[tile_index], Room | Corridor | Doorway) {
            image.draw_pixel(x, y, Color::YELLOW);
        }
        // if matches!(grid.tiles[tile_index], Blocker) {
        //     image.draw_pixel(x, y, Color::BLACK);
        // }
        // if matches!(grid.tiles[tile_index], CorridorNeighbor) {
        //     image.draw_pixel(x, y, Color::YELLOWGREEN);
        // }
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
    let mut rng = rand::rng();
    let mut grid_width: usize = 100;
    let mut grid_height: usize = 100;
    let mut grid_dimensions = vec2u(grid_width, grid_height);
    let mut target_room_count: usize = 30;
    let mut reintroduced_corridor_density: f32 = 0.5;
    let mut configuration = Configuration::default();
    // ============================== Configuration variables


    // ============================== State variables
    let rooms = generate_rooms(
        &configuration,
        grid_dimensions,
        Some(target_room_count),
        &mut rng,
    );
    let mut triangulation = triangulate(grid_dimensions, rooms);
    let mut corridors = pick_corridors(&configuration, triangulation.clone(), &mut rng);
    let mut dimensions_changed: bool = false;
    let mut image = Image::gen_image_color(
        grid_dimensions.x as i32,
        grid_dimensions.y as i32,
        Color::BROWN,
    );
    draw_grid_to_image(&configuration, grid_dimensions, &corridors, &mut image);
    let mut texture = rl.load_texture_from_image(&thread, &image);
    let mut draw_option: DrawOption = DrawOption::Grid;
    // ============================== State variables


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
                        400,
                        &mut grid_width,
                    );
                    dimensions_changed |= ui.slider(
                        "Grid Height",
                        configuration.min_room_dimension + configuration.min_padding * 2,
                        400,
                        &mut grid_height,
                    );
                    grid_dimensions = vec2u(grid_width, grid_height);
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

                if ui.button("Regenerate") {
                    if dimensions_changed {
                        image.resize_nn(grid_width as i32, grid_height as i32);
                        offset = view_center - grid_dimensions * scale / 2.0;
                        dimensions_changed = false;
                    }
                    let rooms = generate_rooms(
                        &configuration,
                        grid_dimensions,
                        Some(target_room_count),
                        &mut rng,
                    );
                    triangulation = triangulate(grid_dimensions, rooms);
                    corridors = pick_corridors(&configuration, triangulation.clone(), &mut rng);
                    draw_grid_to_image(&configuration, grid_dimensions, &corridors, &mut image);
                    texture = rl.load_texture_from_image(&thread, &image);
                }

                if ui.button("Regenerate Corridors") {
                    corridors = pick_corridors(&configuration, triangulation.clone(), &mut rng);
                    draw_grid_to_image(&configuration, grid_dimensions, &corridors, &mut image);
                    texture = rl.load_texture_from_image(&thread, &image);
                }

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

        { // ============================== Input
            if rl.is_key_pressed(KeyboardKey::KEY_R) {
                let rooms = generate_rooms(
                    &configuration,
                    grid_dimensions,
                    Some(target_room_count),
                    &mut rng,
                );
                triangulation = triangulate(grid_dimensions, rooms);
                corridors = pick_corridors(&configuration, triangulation.clone(), &mut rng);
                draw_grid_to_image(&configuration, grid_dimensions, &corridors, &mut image);
                texture = rl.load_texture_from_image(&thread, &image);
            }

            // Delta time.
            let dt = rl.get_frame_time();

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
        } // ============================== Input

        let mut draw_handle = rl.begin_drawing(&thread);

        draw_handle.clear_background(Color::BLACK);

        use DrawOption::*;
        match draw_option {
            Grid => {
                if let Ok(texture) = &texture {
                    draw_handle.draw_texture_ex(texture, offset, 0.0, scale, Color::WHITE)
                }
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

        gui.end();
    }
}
