use dungen::Configuration;
use dungen::room::generate_rooms;
use dungen::triangulation::triangulate;
use dungen::vec::{vec2, vec2i};

use raylib::prelude::*;

const CONTROLS: &str = "\
[I, J, K, L] - up, left, down, right;
[U, O] - zoom in / out
[C] - reset position & zoom
[R] - generate new dungeon
[ESC] - exit
";

#[cfg(not(tarpaulin_include))]
fn main() {
    let mut rng = rand::rng();
    let grid_dimensions = vec2i(100, 100);
    let configuration = Configuration::new(5, 20, 3, 2, 20);
    let mut rooms = generate_rooms(&configuration, grid_dimensions, Some(15), &mut rng);
    let mut triangulation = triangulate(grid_dimensions, rooms);

    let (mut rl, thread) = raylib::init().size(640, 640).title("Dungeon").build();

    let mut scale: f32 = 2.0;
    let speed: f32 = 20.0;
    let window_center: Vector2 = vec2i(640, 640) / 2.0;
    let mut offset: Vector2 = window_center - grid_dimensions * scale / 2.0;

    while !rl.window_should_close() {
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            rooms = generate_rooms(&configuration, grid_dimensions, Some(15), &mut rng);
            triangulation = triangulate(grid_dimensions, rooms);
        }

        // Delta time.
        let dt = rl.get_frame_time();

        if rl.is_key_pressed(KeyboardKey::KEY_C) {
            scale = 2.0;
            offset = window_center - grid_dimensions * scale / 2.0;
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

        draw_handle.draw_rectangle_v(offset, grid_dimensions * scale, Color::GRAY);

        for room in &triangulation.rooms {
            let room_corner = offset + vec2(room.bounds.x, room.bounds.y) * scale;
            let room_dimensions = vec2(room.bounds.width, room.bounds.height) * scale;
            draw_handle.draw_rectangle_v(room_corner, room_dimensions, Color::PURPLE);
        }

        for doorway in &triangulation.doorways {
            draw_handle.draw_rectangle_v(
                offset + doorway.position * scale,
                vec2(scale, scale),
                Color::BLUE,
            );
        }

        for edge in &triangulation.edges {
            draw_handle.draw_line_v(
                offset + triangulation.doorways[edge.0].position * scale + scale / 2.0,
                offset + triangulation.doorways[edge.1].position * scale + scale / 2.0,
                Color::LIME,
            );
        }

        draw_handle.draw_text(CONTROLS, 5, 5, 20, Color::WHITE);
    }
}
