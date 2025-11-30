mod thread;
mod ui;

use dungen::Configuration;
use dungen::grid::Grid;
use dungen::room::{Doorway, Dungeon, Room};

use thread::{Generator, Request, Result};
use ui::ExportResult;

use std::sync::mpsc;

use raylib::prelude::*;
use raylib_imgui::RaylibGui;

const CONTROLS: &str = "\
[I, J, K, L] - up, left, down, right
[U, O] - zoom in / out
[C] - reset position & zoom
[R] - generate new dungeon
[ESC] - exit
";

const MAX_ROOM_COUNT: usize = 2_000;
const MAX_MAP_DIMENSIONS: usize = 512;

// Shorthands for raylib::math::Vector2
#[cfg(not(tarpaulin_include))]
fn vec2(x: f32, y: f32) -> Vector2 {
    Vector2::new(x, y)
}
#[cfg(not(tarpaulin_include))]
fn vec2u(x: usize, y: usize) -> Vector2 {
    Vector2::new(x as f32, y as f32)
}
#[cfg(not(tarpaulin_include))]
fn cast2f(vector: dungen::vec::Vector2) -> Vector2 {
    Vector2::new(vector.x as f32, vector.y as f32)
}

#[cfg(not(tarpaulin_include))]
fn draw_rooms(scale: f32, offset: Vector2, draw_handle: &mut impl RaylibDraw, rooms: &[Room]) {
    for room in rooms {
        let room_corner = offset + vec2u(room.bounds.x, room.bounds.y) * scale;
        let room_dimensions = vec2u(room.bounds.width, room.bounds.height) * scale;
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
            offset + cast2f(doorway.position) * scale,
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
            offset + cast2f(doorways[edge.0].position) * scale + vec2(1.0, 1.0) * scale / 2.0,
            offset + cast2f(doorways[edge.1].position) * scale + vec2(1.0, 1.0) * scale / 2.0,
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
    dungeon: &Dungeon,
    edges: &[(usize, usize)],
) {
    draw_handle.draw_rectangle_v(offset, grid_dimensions * scale, Color::GRAY);

    draw_rooms(scale, offset, draw_handle, &dungeon.rooms);
    draw_doorways(scale, offset, draw_handle, &dungeon.doorways);
    draw_edges(scale, offset, draw_handle, &dungeon.doorways, edges);
}

#[cfg(not(tarpaulin_include))]
fn draw_grid(grid: &Grid, draw_handle: &mut impl RaylibDraw, highlight_special: bool) {
    use dungen::grid::Tile::*;
    draw_handle.clear_background(Color::BROWN);
    for tile_index in 0..grid.tiles.len() {
        let x = (tile_index % grid.width) as i32;
        let y = (tile_index / grid.width) as i32;
        if matches!(grid.tiles[tile_index], Room | Corridor | Doorway) {
            draw_handle.draw_pixel(x, y, Color::YELLOW);
        }
        if highlight_special {
            if matches!(grid.tiles[tile_index], Doorway) {
                draw_handle.draw_pixel(x, y, Color::BLUEVIOLET);
            }
            if matches!(grid.tiles[tile_index], Blocker) {
                draw_handle.draw_pixel(x, y, Color::PURPLE);
            }
            if matches!(grid.tiles[tile_index], CorridorNeighbor) {
                draw_handle.draw_pixel(x, y, Color::YELLOWGREEN);
            }
        }
    }
}

#[cfg(not(tarpaulin_include))]
fn main() {
    // ============================== Library variables
    let (mut rl, thread) = raylib::init()
        .size(1280, 720)
        .title("Dungeon Generator")
        .build();
    let mut gui = RaylibGui::new(&mut rl, &thread);
    // ==============================

    // ============================== Configuration variables
    let mut grid_width: usize = 100;
    let mut grid_height: usize = 100;
    let mut grid_dimensions = dungen::vec::vec2u(grid_width, grid_height);
    let mut target_room_count: usize = 30;
    let mut reintroduced_corridor_density: f32 = 0.5;
    let mut configuration = Configuration::default();
    // ============================== Configuration variables

    // ============================== State variables
    // Drawing on the render texture is flipped vertically, so we need to modify the source
    // rectangle.
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
    let generator = thread::make_generator();
    // Generate a grid that won't take much time before the start of the application.
    let request = Request::New {
        configuration: configuration.clone(),
        grid_dimensions,
        target_room_count: 20,
    };
    if generator.requests.send(request).is_err() {
        return;
    }
    let (mut rooms, mut triangulation, mut corridors, mut grid) = if let Ok(Result::New {
        dungeon: rooms,
        triangulation,
        corridors,
        grid,
    }) = generator.results.recv()
    {
        (rooms, triangulation, corridors, grid)
    } else {
        return;
    };
    let highlight_special = false;
    rl.draw_texture_mode(&thread, &mut render_texture, |mut handle| {
        draw_grid(&grid, &mut handle, highlight_special);
    });
    let mut dimensions_changed: bool = false;
    let mut draw_option: ui::DrawOption = ui::DrawOption::Grid;
    // ============================== State variables

    // ============================== Exporting
    let mut export_path = String::new();
    let mut export_result: ExportResult = Ok(());
    let mut editing_text: bool = false;
    // ============================== Exporting

    // ============================== Progress variables
    let dot_delay: f32 = 0.75;
    let mut dot_delay_timer: f32 = 0.0;
    let max_dot_count: usize = 3;
    let mut current_dot_mask: usize = 0;
    let message = "generating...";
    // ============================== Progress variables

    // ============================== Movement variables
    let scale_speed: f32 = 5.0;
    let mut scale: f32 = 5.0;
    let speed: f32 = 10.0;
    let view_center: Vector2 = vec2(1280.0 * 3.0 / 4.0, 720.0 / 2.0);
    let mut offset: Vector2 = view_center - cast2f(grid_dimensions) * scale / 2.0;
    // ============================== Movement variables

    'main_thread: while !rl.window_should_close() {
        let ui = gui.begin(&mut rl);
        ui::draw_ui(
            ui,
            &mut configuration,
            &mut reintroduced_corridor_density,
            &mut dimensions_changed,
            &mut draw_option,
            &mut grid_width,
            &mut grid_height,
            &mut target_room_count,
            &mut generating,
            &mut export_path,
            &mut export_result,
            &mut editing_text,
            &grid,
            &generator,
            &rooms,
            &triangulation,
        );

        // Delta time.
        let dt = rl.get_frame_time();

        // ============================== Input
        'input: {
            if editing_text {
                break 'input;
            }

            if !generating && rl.is_key_pressed(KeyboardKey::KEY_R) {
                generating = true;
                if generator
                    .requests
                    .send(Request::New {
                        configuration: configuration.clone(),
                        grid_dimensions: dungen::vec::vec2u(grid_width, grid_height),
                        target_room_count,
                    })
                    .is_err()
                {
                    break 'main_thread;
                };
            }

            if rl.is_key_pressed(KeyboardKey::KEY_C) {
                scale = 5.0;
                offset = view_center - cast2f(grid_dimensions) * scale / 2.0;
            }

            if rl.is_key_down(KeyboardKey::KEY_U) {
                let previous_scale = scale;
                scale += scale * scale_speed * dt;
                offset = (offset - view_center) * scale / previous_scale + view_center;
            }
            if rl.is_key_down(KeyboardKey::KEY_O) {
                let previous_scale = scale;
                scale -= scale * scale_speed * dt;
                if scale < 2.0 {
                    scale = 2.0;
                }
                offset = (offset - view_center) * scale / previous_scale + view_center;
            }

            let step = scale * speed * speed * dt;
            if rl.is_key_down(KeyboardKey::KEY_I) {
                offset.y += step;
            }
            if rl.is_key_down(KeyboardKey::KEY_K) {
                offset.y -= step;
            }
            if rl.is_key_down(KeyboardKey::KEY_J) {
                offset.x += step;
            }
            if rl.is_key_down(KeyboardKey::KEY_L) {
                offset.x -= step;
            }
        }
        // ============================== Input

        // Try to receive a result from the generator and update the state variables.
        match generator.results.try_recv() {
            Ok(result) => {
                match result {
                    Result::New {
                        dungeon: new_rooms,
                        triangulation: new_triangulation,
                        corridors: new_corridors,
                        grid: new_grid,
                    } => {
                        if dimensions_changed {
                            grid_dimensions = dungen::vec::vec2u(grid_width, grid_height);
                            offset = view_center - cast2f(grid_dimensions) * scale / 2.0;
                            source_rectangle = Rectangle::new(
                                0.0,
                                MAX_MAP_DIMENSIONS as f32 - grid_dimensions.y as f32,
                                grid_dimensions.x as f32,
                                -grid_dimensions.y as f32,
                            );
                            dimensions_changed = false;
                        }
                        rooms = new_rooms;
                        triangulation = new_triangulation;
                        corridors = new_corridors;
                        grid = new_grid;
                        rl.draw_texture_mode(&thread, &mut render_texture, |mut handle| {
                            draw_grid(&grid, &mut handle, highlight_special);
                        });
                    }
                    Result::Corridors {
                        corridors: new_corridors,
                        grid: new_grid,
                    } => {
                        corridors = new_corridors;
                        grid = new_grid;
                        rl.draw_texture_mode(&thread, &mut render_texture, |mut handle| {
                            draw_grid(&grid, &mut handle, highlight_special);
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

        // ============================== Drawing
        let mut draw_handle = rl.begin_drawing(&thread);

        draw_handle.clear_background(Color::BLACK);

        use ui::DrawOption::*;
        match draw_option {
            Grid => {
                let destination_rectangle = Rectangle::new(
                    offset.x,
                    offset.y,
                    grid_dimensions.x as f32 * scale,
                    grid_dimensions.y as f32 * scale,
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
            Corridors => draw_graph(
                scale,
                offset,
                cast2f(grid_dimensions),
                &mut draw_handle,
                &rooms,
                &corridors,
            ),
            Triangulation => draw_graph(
                scale,
                offset,
                cast2f(grid_dimensions),
                &mut draw_handle,
                &rooms,
                &triangulation,
            ),
        };

        // ============================== Visual feedback for dungeon generation
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
        // ============================== Visual feedback for dungeon generation
        // ============================== Drawing

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
