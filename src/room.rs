use crate::{Configuration, rng::Rng, vec::vec2};

use std::ops::RangeInclusive;

use raylib::math::{Rectangle, Vector2};

const EAST: usize = 0;
const NORTH: usize = 1;
const WEST: usize = 2;
const SOUTH: usize = 3;

/// Structure representing a rectangular room in the grid.
#[derive(Clone, Debug)]
pub struct Room {
    pub bounds: Rectangle,
}

#[derive(Clone, Debug)]
pub struct Doorway {
    pub room_index: usize,
    pub position: Vector2,
}

#[derive(Debug)]
pub struct Rooms {
    pub rooms: Vec<Room>,
    pub doorways: Vec<Doorway>,
}

#[derive(Clone, Debug)]
pub struct RoomGraph {
    pub rooms: Vec<Room>,
    pub doorways: Vec<Doorway>,
    pub edges: Vec<(usize, usize)>,
}

/// Guarantees that there is at least min_padding cells distance between the two rectangles. We
/// need only extend the rectangles down and to the right.
pub fn overlap_with_padding(min_padding: usize, a: &Rectangle, b: &Rectangle) -> bool {
    let mut padded_a = *a;
    padded_a.width += min_padding as f32;
    padded_a.height += min_padding as f32;
    let mut padded_b = *b;
    padded_b.width += min_padding as f32;
    padded_b.height += min_padding as f32;
    padded_a.check_collision_recs(&padded_b)
}

/// Generates doorways given a room.
pub fn generate_doorways<R: Rng>(
    doorway_offset: usize,
    room_index: usize,
    room: &Room,
    doorways: &mut Vec<Doorway>,
    rng: &mut R,
) {
    // Generate a random number representing which doorways exist.
    let doorway_mask = rng.random_range(1..=15);
    let corner = vec2(room.bounds.x, room.bounds.y);

    let vertical_range = doorway_offset..=room.bounds.height as usize - doorway_offset - 1;
    let horizontal_range = doorway_offset..=room.bounds.width as usize - doorway_offset - 1;

    if doorway_mask & (1 << EAST) != 0 {
        doorways.push(Doorway {
            room_index,
            position: corner
                + vec2(
                    room.bounds.width,
                    rng.random_range(vertical_range.clone()) as f32,
                ),
        });
    }
    if doorway_mask & (1 << NORTH) != 0 {
        doorways.push(Doorway {
            room_index,
            position: corner + vec2(rng.random_range(horizontal_range.clone()) as f32, -1.0),
        });
    }
    if doorway_mask & (1 << WEST) != 0 {
        doorways.push(Doorway {
            room_index,
            position: corner + vec2(-1.0, rng.random_range(vertical_range.clone()) as f32),
        });
    }
    if doorway_mask & (1 << SOUTH) != 0 {
        doorways.push(Doorway {
            room_index,
            position: corner
                + vec2(
                    rng.random_range(horizontal_range.clone()) as f32,
                    room.bounds.height,
                ),
        });
    }
}

/// Randomly picks a position and then valid dimensions to place a room. Attempts to place a given
/// amount of rooms but aborts the operation if there are a number of failed attempts specified in
/// the configuration.
pub fn generate_rooms<R: Rng>(
    configuration: &Configuration,
    grid_dimensions: Vector2,
    target_room_count: Option<usize>,
    rng: &mut R,
) -> Rooms {
    let mut result: Rooms = Rooms {
        rooms: vec![],
        doorways: vec![],
    };

    let min_padding = configuration.min_padding;
    let min_room_dimension = configuration.min_room_dimension;
    let max_room_dimension = configuration.max_room_dimension;

    let target_room_count =
        target_room_count.unwrap_or((grid_dimensions.x * grid_dimensions.y) as usize);
    let x_range = min_padding..=(grid_dimensions.x as usize - min_room_dimension - min_padding);
    let y_range = min_padding..=(grid_dimensions.y as usize - min_room_dimension - min_padding);

    let mut room_count = 0;
    let mut fail_count = 0;

    // Tarpaulin (code coverage) does not seem to be able to handle variable declarations without
    // initialisation.
    let mut x: usize;
    let mut y: usize;
    let mut width_range: RangeInclusive<usize>;
    let mut height_range: RangeInclusive<usize>;
    let mut width: usize;
    let mut height: usize;
    let mut rectangle: Rectangle;

    'outer: while room_count < target_room_count {
        if fail_count > configuration.max_fail_count {
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

        for previous_room in &result.rooms {
            if overlap_with_padding(min_padding, &previous_room.bounds, &rectangle) {
                fail_count += 1;
                continue 'outer;
            }
        }

        let room_index = result.rooms.len();
        let room = Room { bounds: rectangle };

        generate_doorways(
            configuration.doorway_offset,
            room_index,
            &room,
            &mut result.doorways,
            rng,
        );

        fail_count = 0;
        room_count += 1;
        result.rooms.push(room);
    }

    result
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{mock::*, vec::vec2u};
    use raylib::math::Rectangle;

    #[test]
    fn overlap() {
        let rect_a = Rectangle::new(0.0, 0.0, 5.0, 5.0);
        let rect_b = Rectangle::new(8.0, 3.0, 5.0, 5.0);
        let rect_c = Rectangle::new(7.0, 5.0, 5.0, 5.0);

        assert!(
            !overlap_with_padding(3, &rect_a, &rect_b),
            "Rectangles should not overlap with padding."
        );
        assert!(
            !overlap_with_padding(3, &rect_b, &rect_a),
            "Rectangles should not overlap with padding."
        );
        assert!(
            overlap_with_padding(3, &rect_a, &rect_c),
            "Rectangles should overlap with padding."
        );
        assert!(
            overlap_with_padding(3, &rect_c, &rect_a),
            "Rectangles should overlap with padding."
        );
        assert!(
            !overlap_with_padding(2, &rect_a, &rect_c),
            "Rectangles should not overlap with padding."
        );
    }

    fn doorway_generation_variant(mut rng: impl Rng, doorway_count: usize) {
        let doorway_offset = 2;
        let mut doorways: Vec<Doorway> = vec![];
        let rectangle = Rectangle::new(1.0, 1.0, 5.0, 5.0);
        let outline = Rectangle::new(
            rectangle.x - 1.0,
            rectangle.y - 1.0,
            rectangle.width + 2.0,
            rectangle.height + 2.0,
        );
        let corners = [
            vec2(outline.x, outline.y),
            vec2(outline.x, outline.y + outline.height - 1.0),
            vec2(outline.x + outline.width - 1.0, outline.y),
            vec2(
                outline.x + outline.width - 1.0,
                outline.y + outline.height - 1.0,
            ),
        ];

        generate_doorways(
            doorway_offset,
            0,
            &Room { bounds: rectangle },
            &mut doorways,
            &mut rng,
        );

        assert_eq!(
            doorways.len(),
            doorway_count,
            "There should be {} doorways.",
            doorway_count
        );
        for doorway in doorways {
            assert!(
                outline.check_collision_point_rec(doorway.position),
                "Doorway must be within the outline of the room."
            );
            assert!(
                !rectangle.check_collision_point_rec(doorway.position),
                "Doorway must be outside the room."
            );
            for corner in &corners {
                let manhatan =
                    (corner.x - doorway.position.x).abs() + (corner.y - doorway.position.y).abs();
                assert!(
                    manhatan as usize > doorway_offset,
                    "Doorway should be farther from the corner of the outline of the room."
                )
            }
        }
    }

    #[test]
    fn doorway_generation() {
        doorway_generation_variant(MockMinRng, 1);
        doorway_generation_variant(MockMaxRng, 4);
    }

    #[test]
    fn room_generation_max_rng() {
        let configuration = Configuration::default();
        let map_dimension = configuration.min_padding * 2 + configuration.min_room_dimension;
        let result = generate_rooms(
            &configuration,
            vec2u(map_dimension, map_dimension),
            Some(1),
            &mut MockMaxRng,
        );
        assert_eq!(result.rooms.len(), 1, "There should be exactly 1 room.");
        assert!(
            result.rooms[0].bounds.width as usize == configuration.min_room_dimension
                && result.rooms[0].bounds.height as usize == configuration.min_room_dimension,
            "The room dimensions are not correct."
        );
    }

    // In this and the following test we do not care about generating doorways, so the mock_rng
    // chooses a number outside the range of possible doorway masks.
    #[test]
    fn room_generation_failed_second_room() {
        let configuration = Configuration {
            max_fail_count: 3,
            ..Default::default()
        };
        let map_dimension = configuration.min_padding * 3 + 10;
        let map_dimensions = vec2u(map_dimension, map_dimension);

        // x, y, width, height, doorway mask
        let mut numbers: Vec<usize> = vec![0, 0, 5, 5, 0];
        numbers.append(&mut [6, 6, 5, 5].repeat(configuration.max_fail_count + 1));
        let mut mock_rng = MockRng::new(numbers);

        let result = generate_rooms(&configuration, map_dimensions, None, &mut mock_rng);

        assert_eq!(result.rooms.len(), 1, "There should be exactly 1 room.");
    }

    #[test]
    fn room_generation_two_rooms_one_failure() {
        let configuration = Configuration {
            max_fail_count: 3,
            ..Default::default()
        };
        let map_dimension = configuration.min_padding * 3 + 10;
        let map_dimensions = vec2u(map_dimension, map_dimension);
        // x, y, width, height, doorway mask
        let numbers: Vec<usize> = vec![0, 0, 5, 5, 0, 6, 6, 5, 5, 9, 9, 5, 5, 0];
        let mut mock_rng = MockRng::new(numbers);

        let result = generate_rooms(&configuration, map_dimensions, Some(2), &mut mock_rng);

        assert_eq!(result.rooms.len(), 2, "There should be exactly 2 rooms.");
    }
}
