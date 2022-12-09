use super::tactic::Tactic;

pub trait Play {
    fn name(&self) -> &str;
    fn can_start(&self) -> bool;
    fn can_continue(&self) -> bool;
    fn tick(&self) -> RequestedTactics;
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

    fn tick(&self) -> RequestedTactics {
        todo!()
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

    fn tick(&self) -> RequestedTactics {
        todo!()
    }
}
