use crate::gameplay::world::{Robot, World};
use crate::gameplay::State;
use crate::geom::{Angle, Point};
use crate::motion::planner::{stopping_trajectory, straight_line};
use crate::motion::Trajectory;

#[derive(Clone, Debug)]
pub enum Tactic {
    Stop,
    Move((Point, Angle)),
    // ShadowEnemy(Robot),
}

impl Tactic {
    pub fn robot_assignment_cost(&self, robot: &Robot) -> f64 {
        match self {
            Self::Stop => 0.5,
            // Self::Move((p, a)) => ((p - &robot.state.position).length()*1000.0).powi(2),
            Self::Move((p, a)) => (p - &robot.state.position).length(),
            // Self::ShadowEnemy(r) => (r.state.position - robot.state.position).length(),
        }
    }

    pub fn run(&self, robot: &Robot, world: &World, state: &State) -> Trajectory {
        match self {
            Self::Stop => stopping_trajectory(&robot.state),
            Self::Move((p, a)) => straight_line(&robot.state, p, a),
            // Self::ShadowEnemy(r) => straight_line(&robot.state, &r.state.position, &Angle::zero()),
        }
    }
}
