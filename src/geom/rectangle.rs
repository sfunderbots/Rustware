use super::point::Point;

#[derive(Clone, Copy, Debug)]
pub struct Rectangle {
    bottom_left: Point,
    top_right: Point,
}

impl Rectangle {
    pub fn new(p1: Point, p2: Point) -> Rectangle {
        Rectangle {
            bottom_left: Point {
                x: p1.x.min(p2.x),
                y: p1.y.min(p2.y),
            },
            top_right: Point {
                x: p1.x.max(p2.x),
                y: p1.y.max(p2.y),
            },
        }
    }

    pub fn len_x(&self) -> f32 {
        self.top_right.x - self.bottom_left.x
    }

    pub fn len_y(&self) -> f32 {
        self.top_right.y - self.bottom_left.y
    }

    pub fn centre(&self) -> Point {
        Point {
            x: self.bottom_left.x + self.len_x(),
            y: self.bottom_left.y + self.len_y(),
        }
    }
}
