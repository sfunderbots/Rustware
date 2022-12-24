use super::tactic::Tactic;
use crate::perception::World;
use strum_macros::Display;
use strum_macros::EnumIter;
use crate::perception::game_state::GameState;

pub struct RequestedTactics {
    greedy: Vec<Tactic>,
    optimized: Vec<Tactic>,
}

impl RequestedTactics {
    pub fn new() -> RequestedTactics {
        RequestedTactics {
            greedy: vec![],
            optimized: vec![],
        }
    }
}

#[derive(Debug, Copy, Clone, EnumIter, Display)]
pub enum Play {
    Halt,
    Stop,
    Defense,
}

impl Play {
    pub fn can_start(&self, state: &GameState) -> bool {
        match self {
            Self::Halt => state.halted(),
            Self::Stop => state.stopped(),
            Self::Defense => state.playing(),
        }
    }

    pub fn can_continue(&self, state: &GameState) -> bool {
        match self {
            Self::Halt => state.halted(),
            Self::Stop => state.stopped(),
            Self::Defense => state.playing(),
        }
    }

    pub fn run(&self) -> RequestedTactics {
        match self {
            Self::Halt => RequestedTactics::new(),
            Self::Stop => RequestedTactics::new(),
            Self::Defense => RequestedTactics::new(),
        }
    }
}
