use crate::geom::{Angle, Point};
use crate::motion::Trajectory;
use crate::motion::planner::{stopping_trajectory, straight_line};
use crate::world::Robot;

pub trait Tactic {
    fn preprocess_tick(&self) {}
    fn robot_assignment_cost(&self, robot: &Robot) -> f32;
    fn tick(&self, robot: &Robot) -> Trajectory;
}

pub struct Stop {}
impl Tactic for Stop {
    fn robot_assignment_cost(&self, robot: &Robot) -> f32 {
        0.5
    }

    fn tick(&self, robot: &Robot) -> Trajectory {
        stopping_trajectory(&robot.state)
    }
}

pub struct Move {
    pub position: Point,
    pub orientation: Angle
}
impl Tactic for Move {
    fn robot_assignment_cost(&self, robot: &Robot) -> f32 {
        (robot.state.position - self.position).length()
    }

    fn tick(&self, robot: &Robot) -> Trajectory {
        straight_line(&robot.state, &self.position, &self.orientation)
    }
}
