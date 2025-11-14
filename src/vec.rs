use raylib::math::Vector2;

/// Converts a point to an array grid index.
/// Grids will be stored in a single-dimension array
/// for the purposes of less allocation.
pub fn to_index(v: Vector2, grid_width: usize) -> usize {
    debug_assert!(v.x >= 0.0);
    debug_assert!(v.x < grid_width as f32);
    debug_assert!(v.y >= 0.0);
    grid_width * v.y as usize + v.x as usize
}

// Shorthands for creating vectors.
pub fn vec2i(x: i32, y: i32) -> Vector2 {
    Vector2 {
        x: x as f32,
        y: y as f32,
    }
}
pub fn vec2u(x: usize, y: usize) -> Vector2 {
    Vector2 {
        x: x as f32,
        y: y as f32,
    }
}
pub fn vec2(x: f32, y: f32) -> Vector2 {
    Vector2 { x, y }
}

/// Checks whether a point is in the circumcircle of a triangle.
/// Reference for finding the circucenter and the circumradius can be found
/// [here](https://en.wikipedia.org/wiki/Circumcircle#Circumcenter_coordinates).
/// Avoids division.
pub fn point_in_circumcircle(mut p: Vector2, a: Vector2, mut b: Vector2, mut c: Vector2) -> bool {
    // Translate the points such that a is the center.
    p -= a;
    b -= a;
    c -= a;

    // Here I do not divide the circumcenter (u) by the value d.
    let u = vec2(
        c.y * b.length_sqr() - b.y * c.length_sqr(),
        b.x * c.length_sqr() - c.x * b.length_sqr(),
    );
    let d = (b.x * c.y - b.y * c.x) * 2.0;

    // Instead, I scale the point (p) by that value, to avoid division and a check
    // whether d is equal to 0.
    p *= d;
    (p - u).length_sqr() <= u.length_sqr()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn circumcircle() {
        assert!(
            point_in_circumcircle(vec2i(7, 10), vec2i(6, 4), vec2i(-6, 12), vec2i(0, 1)),
            "Point should have been in the circumcircle."
        );

        assert!(
            !point_in_circumcircle(vec2i(8, 10), vec2i(6, 4), vec2i(-6, 12), vec2i(0, 1)),
            "Point should not have been in the circumcircle."
        );

        // Test with very big numbers.
        assert!(
            point_in_circumcircle(
                vec2i(700, 1000),
                vec2i(600, 400),
                vec2i(-600, 1200),
                vec2i(0, 100)
            ),
            "Point should have been in the circumcircle."
        );
    }

    #[test]
    pub fn index() {
        let grid_width = 5;
        let vector = vec2i(2, 3);
        let index_in_grid = 17;
        assert_eq!(
            to_index(vector, grid_width),
            index_in_grid,
            "Index in grid does not match with vector."
        );

        let col = 3;
        let row = 2;
        let index_in_grid = to_index(vec2u(row, col), grid_width);
        assert_eq!(
            index_in_grid % grid_width,
            row,
            "Index in grid does not convert back to row correctly."
        );
        assert_eq!(
            index_in_grid / grid_width,
            col,
            "Index in grid does not convert back to column correctly."
        );
    }
}
