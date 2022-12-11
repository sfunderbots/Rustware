use crate::world::World;
use super::tactic::Tactic;
use strum_macros::EnumIter;
use strum_macros::Display;


pub struct RequestedTactics {
    greedy: Vec<Tactic>,
    optimized: Vec<Tactic>,
}

impl RequestedTactics {
    pub fn new() -> RequestedTactics {
        RequestedTactics{
            greedy: vec![],
            optimized: vec![]
        }
    }
}

#[derive(Debug, Copy, Clone, EnumIter, Display)]
pub enum Play {
    Halt,
    Stop,
    Defense
}

impl Play {
    pub fn can_start(&self) -> bool {
        match self {
            Self::Halt => true,
            Self::Stop => true,
            Self::Defense => true,
        }
    }

    pub fn can_continue(&self) -> bool {
        match self {
            Self::Halt => true,
            Self::Stop => true,
            Self::Defense => true,
        }
    }

    pub fn run(&self) -> RequestedTactics{
        match self {
            Self::Halt => {
                RequestedTactics::new()
            },
            Self::Stop => {
                RequestedTactics::new()
            },
            Self::Defense => {
                RequestedTactics::new()
            },
        }
    }
}
