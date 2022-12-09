use crate::geom::{Angle, Point, Rectangle, Vector};
use crate::motion::KinematicState;

pub struct Field {
    pub x_length: f32,
    pub y_length: f32,
    pub defense_x_length: f32,
    pub defense_y_length: f32,
    pub goal_x_length: f32,
    pub goal_y_length: f32,
    pub boundary_buffer_size: f32,
    pub center_circle_radius: f32,
}

impl Field {
    pub fn ssl_div_b() -> Field {
        Field {
            x_length: 9.0,
            y_length: 6.0,
            defense_x_length: 1.0,
            defense_y_length: 2.0,
            goal_x_length: 0.18,
            goal_y_length: 1.0,
            boundary_buffer_size: 0.3,
            center_circle_radius: 0.5,
        }
    }

    pub fn touch_lines(&self) -> Rectangle {
        Rectangle::new(
            Point {
                x: -self.x_length / 2.0,
                y: -self.y_length / 2.0,
            },
            Point {
                x: self.x_length / 2.0,
                y: self.y_length / 2.0,
            },
        )
    }

    pub fn enemy_defense_area(&self) -> Rectangle {
        Rectangle::new(
            Point {
                x: self.x_length / 2.0 - self.defense_x_length,
                y: -self.defense_y_length / 2.0,
            },
            Point {
                x: self.x_length / 2.0,
                y: self.defense_y_length / 2.0,
            },
        )
    }

    pub fn friendly_defense_area(&self) -> Rectangle {
        Rectangle::new(
            Point {
                x: -self.x_length / 2.0,
                y: -self.defense_y_length / 2.0,
            },
            Point {
                x: -self.x_length / 2.0 + self.defense_x_length,
                y: self.defense_y_length / 2.0,
            },
        )
    }
}

pub struct Robot {
    pub id: usize,
    pub state: KinematicState
}

pub struct Ball {
    pub position: Point,
    pub velocity: Vector,
}

pub struct World {}
