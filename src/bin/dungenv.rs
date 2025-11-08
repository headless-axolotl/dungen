use dungen::Configuration;
use dungen::grid::make_grid;
use dungen::mst::pick_corridors;
use dungen::room::{Doorway, Room, RoomGraph, generate_rooms};
use dungen::triangulation::triangulate;
use dungen::vec::{vec2, vec2i};

use raylib::prelude::*;

const CONTROLS: &str = "\
[I, J, K, L] - up, left, down, right;
[U, O] - zoom in / out
[C] - reset position & zoom
[R] - generate new dungeon
[N] - next draw option
[ESC] - exit
";

const DRAW_OPTION_COUNT: usize = 3;

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

#[cfg(not(tarpaulin_include))]
fn main() {
    let (mut rl, thread) = raylib::init().size(640, 640).title("Dungeon").build();

    let mut rng = rand::rng();
    let grid_dimensions = vec2i(100, 100);
    let configuration = Configuration::new(5, 20, 3, 2, 100, (1, 5), 1, 2, 3);
    let mut rooms = generate_rooms(&configuration, grid_dimensions, Some(30), &mut rng);
    let mut triangulation = triangulate(grid_dimensions, rooms);
    let mut corridors = pick_corridors(&configuration, triangulation.clone(), &mut rng);
    let mut image = Image::gen_image_color(
        grid_dimensions.x as i32,
        grid_dimensions.y as i32,
        Color::BROWN,
    );
    draw_grid_to_image(&configuration, grid_dimensions, &corridors, &mut image);
    let mut texture = rl.load_texture_from_image(&thread, &image);

    let mut draw_option: usize = 0;

    let mut scale: f32 = 2.0;
    let speed: f32 = 20.0;
    let window_center: Vector2 = vec2i(640, 640) / 2.0;
    let mut offset: Vector2 = window_center - grid_dimensions * scale / 2.0;

    while !rl.window_should_close() {
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            rooms = generate_rooms(&configuration, grid_dimensions, Some(30), &mut rng);
            triangulation = triangulate(grid_dimensions, rooms);
            corridors = pick_corridors(&configuration, triangulation.clone(), &mut rng);
            draw_grid_to_image(&configuration, grid_dimensions, &corridors, &mut image);
            texture = rl.load_texture_from_image(&thread, &image);
        }

        // Delta time.
        let dt = rl.get_frame_time();

        if rl.is_key_pressed(KeyboardKey::KEY_C) {
            scale = 2.0;
            offset = window_center - grid_dimensions * scale / 2.0;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_N) {
            draw_option += 1;
            draw_option %= DRAW_OPTION_COUNT;
        }

        if rl.is_key_down(KeyboardKey::KEY_U) {
            let previous_scale = scale;
            scale += 10.0 * dt;
            offset = (offset - window_center) * scale / previous_scale + window_center;
        }
        if rl.is_key_down(KeyboardKey::KEY_O) {
            let previous_scale = scale;
            scale -= 10.0 * dt;
            if scale < 1.0 {
                scale = 1.0;
            }
            offset = (offset - window_center) * scale / previous_scale + window_center;
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

        let mut draw_handle = rl.begin_drawing(&thread);

        draw_handle.clear_background(Color::BLACK);

        match draw_option {
            0 => draw_graph(
                scale,
                offset,
                grid_dimensions,
                &mut draw_handle,
                &triangulation,
            ),
            1 => draw_graph(scale, offset, grid_dimensions, &mut draw_handle, &corridors),
            2 => {
                if let Ok(texture) = &texture {
                    draw_handle.draw_texture_ex(texture, offset, 0.0, scale, Color::WHITE)
                }
            }
            _ => {}
        };

        draw_handle.draw_text(CONTROLS, 5, 5, 20, Color::WHITE);
    }
}
