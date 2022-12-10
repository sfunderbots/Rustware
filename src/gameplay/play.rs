use crate::world::World;
use super::tactic::Tactic;

pub trait Play {
    fn name(&self) -> &str;
    fn can_start(&self) -> bool;
    fn can_continue(&self) -> bool;
    fn tick(&self, world: &World) -> RequestedTactics;
}

pub struct RequestedTactics {
    greedy: Vec<Box<dyn Tactic>>,
    optimized: Vec<Box<dyn Tactic>>,
}

pub struct Halt {}
impl Play for Halt {
    fn name(&self) -> &str {
        "Halt"
    }

    fn can_start(&self) -> bool {
        todo!()
    }

    fn can_continue(&self) -> bool {
        todo!()
    }

    fn tick(&self, world: &World) -> RequestedTactics {
        // todo!()
        RequestedTactics{
            // greedy: vec![Box::new(super::tactic::Stop{}); world.friendly_team.all_robots().len()],
            // greedy: vec![Box::new(super::tactic::Stop{}); 5],
            greedy: (0..world.friendly_team.all_robots().len()).map(|_| {Box::new(super::tactic::Stop{})}).collect(),
            optimized: vec![]
        }
    }
}

pub struct Stop {}
impl Play for Stop {
    fn name(&self) -> &str {
        "Stop"
    }

    fn can_start(&self) -> bool {
        todo!()
    }

    fn can_continue(&self) -> bool {
        todo!()
    }

    fn tick(&self, world: &World) -> RequestedTactics {
        todo!()
    }
}
