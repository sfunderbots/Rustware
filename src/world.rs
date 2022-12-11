use crate::geom::{Angle, Point, Rectangle, Vector};
use crate::motion::KinematicState;
use std::collections::HashMap;

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
    pub state: KinematicState,
}

pub struct Ball {
    pub position: Point,
    pub velocity: Vector,
}

pub struct Team {
    goalie_id: Option<usize>,
    robots: HashMap<usize, Robot>,
}

impl Team {
    pub fn players(&self) -> Vec<&Robot> {
        let mut result = vec![];
        for (id, robot) in self.robots.iter() {
            if self.goalie_id.is_some() && self.goalie_id.unwrap() == *id {
                continue;
            }
            result.push(robot);
        }
        result
    }

    pub fn all_robots(&self) -> Vec<&Robot> {
        let mut result = vec![];
        for (id, robot) in self.robots.iter() {
            result.push(robot);
        }
        result
    }

    pub fn goalie(&self) -> Option<&Robot> {
        match self.goalie_id {
            Some(id) => self.robots.get(&id),
            None => None,
        }
    }

    pub fn new() -> Team {
        Team {
            goalie_id: None,
            robots: HashMap::new(),
        }
    }

    pub fn with_robots(&mut self, robots: Vec<Robot>) -> &mut Self {
        for r in robots {
            self.robots.insert(r.id, r);
        }
        self
    }

    pub fn with_goalie(&mut self, goalie_id: usize) -> &mut Self {
        self.goalie_id = Some(goalie_id);
        self
    }

    pub fn build(self) -> Team {
        self
    }
}

pub struct World {
    pub ball: Ball,
    pub friendly_team: Team,
    pub enemy_team: Team,
    pub field: Field,
}
