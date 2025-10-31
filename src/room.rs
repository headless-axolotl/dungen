use crate::{Configuration, vec::vec2};

use std::ops::RangeInclusive;

use rand::Rng;
use raylib::math::{Rectangle, Vector2};

const EAST: usize = 0;
const NORTH: usize = 1;
const WEST: usize = 2;
const SOUTH: usize = 3;

/// TODO: doc
#[derive(Debug)]
pub struct Room {
    pub rectangle: Rectangle,
    pub doorways: [Vector2; 4],
    pub doorway_count: usize,
}

/// TODO: doc
pub fn overlap_with_padding(min_padding: usize, a: &Rectangle, b: &Rectangle) -> bool {
    let mut padded_a = *a;
    padded_a.width += min_padding as f32;
    padded_a.height += min_padding as f32;
    let mut padded_b = *b;
    padded_b.width += min_padding as f32;
    padded_b.height += min_padding as f32;
    padded_a.check_collision_recs(&padded_b)
}

/// TODO: doc
pub fn generate_doorways<R: Rng>(
    doorway_offset: usize,
    rectangle: &Rectangle,
    rng: &mut R,
) -> ([Vector2; 4], usize) {
    let mut doorways: [Vector2; 4] = [Vector2::zero(); 4];
    // Generate a random number representing which doorways exist.
    let doorway_mask = rng.random_range(1..=15);
    let corner = vec2(rectangle.x, rectangle.y);
    let mut doorway_count = 0;

    let vertical_range = doorway_offset..=rectangle.height as usize - doorway_offset - 1;
    let horizontal_range = doorway_offset..=rectangle.width as usize - doorway_offset - 1;

    if doorway_mask & (1 << EAST) != 0 {
        doorways[doorway_count] = corner
            + vec2(
                rectangle.width,
                rng.random_range(vertical_range.clone()) as f32,
            );
        doorway_count += 1;
    }
    if doorway_mask & (1 << NORTH) != 0 {
        doorways[doorway_count] =
            corner + vec2(rng.random_range(horizontal_range.clone()) as f32, -1.0);
        doorway_count += 1;
    }
    if doorway_mask & (1 << WEST) != 0 {
        doorways[doorway_count] =
            corner + vec2(-1.0, rng.random_range(vertical_range.clone()) as f32);
        doorway_count += 1;
    }
    if doorway_mask & (1 << SOUTH) != 0 {
        doorways[doorway_count] = corner
            + vec2(
                rng.random_range(horizontal_range.clone()) as f32,
                rectangle.height,
            );
        doorway_count += 1;
    }

    (doorways, doorway_count)
}

/// TODO: doc
pub fn generate_rooms<R: Rng>(
    configuration: &Configuration,
    grid_dimensions: Vector2,
    target_room_count: Option<usize>,
    rng: &mut R,
) -> Vec<Room> {
    let mut result: Vec<Room> = vec![];

    let min_padding = configuration.min_padding;
    let min_room_dimension = configuration.min_room_dimension;
    let max_room_dimension = configuration.max_room_dimension;

    let target_room_count =
        target_room_count.unwrap_or((grid_dimensions.x * grid_dimensions.y) as usize);
    let x_range = min_padding..=(grid_dimensions.x as usize - min_room_dimension - min_padding);
    let y_range = min_padding..=(grid_dimensions.y as usize - min_room_dimension - min_padding);

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
        if fail_count >= configuration.max_fail_count {
            break;
        }

        x = rng.random_range(x_range.clone());
        y = rng.random_range(y_range.clone());

        width_range = min_room_dimension
            ..=(grid_dimensions.x as usize - x - min_padding).min(max_room_dimension);
        height_range = min_room_dimension
            ..=(grid_dimensions.y as usize - y - min_padding).min(max_room_dimension);

        width = rng.random_range(width_range);
        height = rng.random_range(height_range);

        rectangle = Rectangle::new(x as f32, y as f32, width as f32, height as f32);

        for previous_room in &result {
            if overlap_with_padding(min_padding, &previous_room.rectangle, &rectangle) {
                fail_count += 1;
                continue 'outer;
            }
        }

        let (doorways, doorway_count) =
            generate_doorways(configuration.doorway_offset, &rectangle, rng);

        fail_count = 0;
        room_count += 1;
        result.push(Room {
            rectangle,
            doorways,
            doorway_count,
        });
    }

    result
}
