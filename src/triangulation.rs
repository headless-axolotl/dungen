use crate::room::{Doorway, Rooms, RoomGraph};
use crate::vec::{point_in_circumcircle, vec2};

use std::collections::HashSet;

use raylib::math::Vector2;

/// Makes edges with consistent point ordering.
fn make_edge(point_a: usize, point_b: usize) -> (usize, usize) {
    (point_a.min(point_b), point_a.max(point_b))
}

/// Employs the Bowyer-Watson algorithm to create a Delaynay triangulation
/// between the doorways of the rooms.
pub fn triangulate(grid_dimensions: Vector2, rooms: Rooms) -> RoomGraph {
    let Rooms {
        rooms,
        mut doorways,
    } = rooms;

    // The last three doorways belong to the super_triangle.
    let none_room_index = rooms.len();
    // Create a right triangle, which covers the whole grid.
    doorways.push(Doorway {
        room_index: none_room_index,
        position: vec2(-1.0, -1.0),
    });
    doorways.push(Doorway {
        room_index: none_room_index,
        position: vec2(-1.0, 2.0 * grid_dimensions.y + 1.0),
    });
    doorways.push(Doorway {
        room_index: none_room_index,
        position: vec2(2.0 * grid_dimensions.x + 1.0, -1.0),
    });

    let super_triangle_first_point_index = doorways.len() - 3;

    let mut triangles: Vec<(usize, usize, usize)> = vec![(
        super_triangle_first_point_index,
        super_triangle_first_point_index + 1,
        super_triangle_first_point_index + 2,
    )];

    let mut bad_triangles: Vec<usize> = vec![];
    let mut polygon: HashSet<(usize, usize)> = HashSet::new();

    // Skip the last three doorways since those are part of the super triangle.
    for (point_index, point) in doorways[..doorways.len() - 3].iter().enumerate() {
        bad_triangles.clear();
        for (triangle_index, triangle) in triangles.iter().enumerate() {
            let point_is_in_circumcircle = point_in_circumcircle(
                point.position,
                doorways[triangle.0].position,
                doorways[triangle.1].position,
                doorways[triangle.2].position,
            );

            if point_is_in_circumcircle {
                bad_triangles.push(triangle_index);
            }
        }

        // Polygonal hole created from the bad triangles.
        polygon.clear();
        // Since each edge is shared by at most 2 triangles,
        // shared edges will be added once and then removed.
        // Non-shared edges will be added just once and not removed.
        // I.e. polygon will contain edges not shared by other triangles.
        let mut add_if_not_shared = |edge: (usize, usize)| {
            if polygon.contains(&edge) {
                polygon.remove(&edge);
            } else {
                polygon.insert(edge);
            }
        };
        for triangle_index in &bad_triangles {
            let triangle = &triangles[*triangle_index];
            add_if_not_shared(make_edge(triangle.0, triangle.1));
            add_if_not_shared(make_edge(triangle.0, triangle.2));
            add_if_not_shared(make_edge(triangle.1, triangle.2));
        }

        // I use swap remove because it works in O(1), therefore,
        // the triangles must be removed in reverse index order.
        // The bad_triangles array is guaranteed to be sorted
        // since that is how we iterated the triangles initially.
        for bad_triangle_index in bad_triangles.iter().rev() {
            triangles.swap_remove(*bad_triangle_index);
        }

        // Make new triangles.
        for edge in &polygon {
            triangles.push((edge.0, edge.1, point_index));
        }
    }

    // Remove triangles which have doorways from
    // the original super triangle.
    let mut triangle_index: usize = 0;
    while triangle_index < triangles.len() {
        let triangle = &triangles[triangle_index];
        let triangle_contains_super_point = triangle.0 >= super_triangle_first_point_index
            || triangle.1 >= super_triangle_first_point_index
            || triangle.2 >= super_triangle_first_point_index;
        if triangle_contains_super_point {
            triangles.swap_remove(triangle_index);
            continue;
        }
        triangle_index += 1;
    }

    // Remove the doorways of the super-triangle.
    doorways.pop();
    doorways.pop();
    doorways.pop();

    // In subsequent steps it is better to have an array
    // of edges instead of the triangles.
    polygon.clear();
    for triangle in triangles {
        polygon.insert(make_edge(triangle.0, triangle.1));
        polygon.insert(make_edge(triangle.0, triangle.2));
        polygon.insert(make_edge(triangle.1, triangle.2));
    }

    RoomGraph {
        rooms,
        doorways,
        edges: polygon.drain().collect(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::vec::vec2u;

    /// Shorthand for creating a doorway.
    fn point(x: usize, y: usize) -> Doorway {
        Doorway {
            room_index: 0,
            position: vec2u(x, y),
        }
    }

    #[test]
    fn triangulation() {
        // For this test we do not need the room boundaries.
        let grid_dimensions = vec2u(100, 100);
        let rooms = Rooms {
            rooms: vec![],
            doorways: vec![point(1, 1), point(2, 1), point(1, 2)],
        };

        let result = triangulate(grid_dimensions, rooms);
        assert_eq!(
            result.edges.len(),
            3,
            "There should be exactly 1 triangle and exactly 3 edges."
        );
        assert!(
            result.edges.contains(&(0, 1))
                && result.edges.contains(&(1, 2))
                && result.edges.contains(&(0, 2)),
            "The result does not contain all the correct edges."
        );
    }
}
