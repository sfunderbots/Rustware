use super::Vector;
use std::ops::{Add, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new() -> Point {
        Point { x: 0.0, y: 0.0 }
    }
}

impl Add<Vector> for Point {
    type Output = Point;
    fn add(self, rhs: Vector) -> Self::Output {
        Point{
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl Sub for Point {
    type Output = Vector;
    fn sub(self, rhs: Self) -> Self::Output {
        Vector{
            x: rhs.x-self.x,
            y: rhs.y-self.y
        }
    }
}
