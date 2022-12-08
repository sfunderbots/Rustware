use crate::motion::Trajectory;

pub trait Tactic {
    fn preprocess_tick(&self);
    fn robot_assignment_cost(&self) -> f32;
    fn tick(&self) -> Trajectory;
}

pub struct Stop {}
impl Tactic for Stop {
    fn preprocess_tick(&self) {
        todo!()
    }

    fn robot_assignment_cost(&self) -> f32 {
        todo!()
    }

    fn tick(&self) -> Trajectory {
        todo!()
    }
}
