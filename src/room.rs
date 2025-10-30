use crate::vec::vec2;

use std::ops::RangeInclusive;

use rand::Rng;
use raylib::math::{Rectangle, Vector2};

pub const MAX_GRID_SIZE: usize = 256;
pub const MIN_ROOM_DIMENSION: usize = 5;
pub const MAX_ROOM_DIMENSION: usize = 50;
pub const MIN_PADDING: usize = 3;
pub const DOORWAY_OFFSET: usize = 2;
pub const MAX_FAIL_COUNT: usize = 10;

const EAST: usize = 0;
const NORTH: usize = 1;
const WEST: usize = 2;
const SOUTH: usize = 3;

/// TODO: doc
#[derive(Debug)]
pub struct Room {
    rectangle: Rectangle,
    doorways: [Vector2; 4],
    doorway_count: usize,
}

/// TODO: doc
pub fn overlap_with_padding(a: &Rectangle, b: &Rectangle) -> bool {
    let mut padded_a = *a;
    padded_a.width += MIN_PADDING as f32;
    padded_a.height += MIN_PADDING as f32;
    let mut padded_b = *b;
    padded_b.width += MIN_PADDING as f32;
    padded_b.height += MIN_PADDING as f32;
    padded_a.check_collision_recs(&padded_b)
}

/// TODO: doc
pub fn generate_doorways<R: Rng>(rectangle: &Rectangle, rng: &mut R) -> ([Vector2; 4], usize) {
    let mut doorways: [Vector2; 4] = [Vector2::zero(); 4];
    // Generate a random number representing which doorways exist.
    let doorway_mask = rng.random_range(1..=15);
    let corner = vec2(rectangle.x, rectangle.y);
    let mut doorway_count = 0;

    if doorway_mask & (1 << EAST) != 0 {
        doorways[doorway_count] = corner + vec2(
            rectangle.width,
            rng.random_range(DOORWAY_OFFSET..=rectangle.height as usize - DOORWAY_OFFSET) as f32
        );
        doorway_count += 1;
    }
    if doorway_mask & (1 << NORTH) != 0 {
        doorways[doorway_count] = corner + vec2(
            rng.random_range(DOORWAY_OFFSET..=rectangle.width as usize - DOORWAY_OFFSET) as f32,
            0.0
        );
        doorway_count += 1;
    }
    if doorway_mask & (1 << WEST) != 0 {
        doorways[doorway_count] = vec2(
            0.0,
            rng.random_range(DOORWAY_OFFSET..=rectangle.height as usize - DOORWAY_OFFSET) as f32
        );
        doorway_count += 1;
    }
    if doorway_mask & (1 << SOUTH) != 0 {
        doorways[doorway_count] = vec2(
            rng.random_range(DOORWAY_OFFSET..=rectangle.width as usize - DOORWAY_OFFSET) as f32,
            rectangle.height,
        );
        doorway_count += 1;
    }

    (doorways, doorway_count)
}

/// TODO: doc
pub fn generate_rooms<R: Rng>(
    grid_dimensions: Vector2,
    target_room_count: Option<usize>,
    rng: &mut R,
) -> Vec<Room> {
    let mut result: Vec<Room> = vec![];

    let target_room_count =
        target_room_count.unwrap_or((grid_dimensions.x * grid_dimensions.y) as usize);
    let x_range = 0..=(grid_dimensions.x as usize - MIN_ROOM_DIMENSION - MIN_PADDING);
    let y_range = 0..=(grid_dimensions.y as usize - MIN_ROOM_DIMENSION - MIN_PADDING);

    let mut room_count = 0;
    let mut fail_count = 0;

    let mut x: usize;
    let mut y: usize;
    let mut width_range: RangeInclusive<usize>;
    let mut height_range: RangeInclusive<usize>;
    let mut width: usize;
    let mut height: usize;
    let mut rectangle: Rectangle;

    'outer: while room_count < target_room_count {
        if fail_count >= MAX_FAIL_COUNT {
            break;
        }

        x = rng.random_range(x_range.clone());
        y = rng.random_range(y_range.clone());

        width_range = MIN_ROOM_DIMENSION
            ..=(grid_dimensions.x as usize - x - MIN_PADDING).min(MAX_ROOM_DIMENSION);
        height_range = MIN_ROOM_DIMENSION
            ..=(grid_dimensions.y as usize - y - MIN_PADDING).min(MAX_ROOM_DIMENSION);

        width = rng.random_range(width_range);
        height = rng.random_range(height_range);

        rectangle = Rectangle::new(x as f32, y as f32, width as f32, height as f32);

        for previous_room in &result {
            if overlap_with_padding(&previous_room.rectangle, &rectangle) {
                fail_count += 1;
                continue 'outer;
            }
        }

        let (doorways, doorway_count) = generate_doorways(&rectangle, rng);

        fail_count = 0;
        room_count += 1;
        result.push(Room { rectangle, doorways, doorway_count });
    }

    result
}
