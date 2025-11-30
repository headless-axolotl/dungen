use std::ops::{Mul, MulAssign};

use derive_more::{Add, AddAssign, Sub, SubAssign};

// Shorthands for creating vectors.
pub fn vec2(x: i32, y: i32) -> Vector2 {
    Vector2 { x, y }
}

pub fn vec2u(x: usize, y: usize) -> Vector2 {
    Vector2 {
        x: x as i32,
        y: y as i32,
    }
}

#[derive(Clone, Copy, Add, AddAssign, Sub, SubAssign, Default, Debug)]
pub struct Vector2 {
    pub x: i32,
    pub y: i32,
}

impl Mul<i32> for Vector2 {
    type Output = Vector2;

    fn mul(self, rhs: i32) -> Self::Output {
        vec2(self.x * rhs, self.y * rhs)
    }
}

impl MulAssign<i32> for Vector2 {
    fn mul_assign(&mut self, rhs: i32) {
        *self = *self * rhs;
    }
}

impl Vector2 {
    pub fn length_sqr(&self) -> i32 {
        self.x * self.x + self.y * self.y
    }

    pub fn length_sqr_f32(&self) -> f32 {
        let x = self.x as f32;
        let y = self.y as f32;
        x * x + y * y
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rectangle {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Rectangle {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }

    pub fn check_collision_recs(&self, other: &Rectangle) -> bool {
        self.x < (other.x + other.width)
            && other.x < (self.x + self.width)
            && self.y < (other.y + other.height)
            && other.y < (self.y + self.height)
    }

    pub fn check_collision_point_rec(&self, point: Vector2) -> bool {
        point.x >= self.x as i32
            && point.x < (self.x + self.width) as i32
            && point.y >= self.y as i32
            && point.y < (self.y + self.height) as i32
    }
}

/// Converts a point to an array grid index.
/// Grids will be stored in a single-dimension array
/// for the purposes of less allocation.
pub fn to_index(v: Vector2, grid_width: usize) -> usize {
    debug_assert!(v.x >= 0);
    debug_assert!(v.x < grid_width as i32);
    debug_assert!(v.y >= 0);
    grid_width * v.y as usize + v.x as usize
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
    let d = (b.x * c.y - b.y * c.x) * 2;

    // The triangle points are on a line.
    if d == 0 {
        return false;
    }

    // Instead, I scale the point (p) by that value.
    // Casting to float helps with bigger numbers.
    p *= d;
    (p - u).length_sqr_f32() <= u.length_sqr_f32()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn circumcircle() {
        assert!(
            point_in_circumcircle(vec2(7, 10), vec2(6, 4), vec2(-6, 12), vec2(0, 1)),
            "Point should have been in the circumcircle."
        );

        assert!(
            !point_in_circumcircle(vec2(8, 10), vec2(6, 4), vec2(-6, 12), vec2(0, 1)),
            "Point should not have been in the circumcircle."
        );

        // Test with very big numbers.
        assert!(
            point_in_circumcircle(
                vec2(700, 1000),
                vec2(600, 400),
                vec2(-600, 1200),
                vec2(0, 100)
            ),
            "Point should have been in the circumcircle."
        );

        assert!(
            !point_in_circumcircle(vec2(5, 5), vec2(1, 1), vec2(2, 2), vec2(3, 3)),
            "When the triangle poitns are collinear, the procedure should return false."
        )
    }

    #[test]
    pub fn index() {
        let grid_width = 5;
        let vector = vec2(2, 3);
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
