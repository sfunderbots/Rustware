use super::tactic::Tactic;
use crate::gameplay::world::GameState;
use crate::gameplay::world::World;
use crate::gameplay::State;
use strum_macros::Display;
use strum_macros::EnumIter;
use crate::geom::{Angle, Point, Vector};

pub struct RequestedTactics {
    pub greedy: Vec<Tactic>,
    pub optimized: Vec<Tactic>,
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

    pub fn run(&self, world: &World, state: &State) -> RequestedTactics {
        match self {
            Self::Halt => {
                RequestedTactics{
                    greedy: world.friendly_team.all_robots().iter().map(|r| {
                        Tactic::Stop
                    }).collect(),
                    optimized: vec![]
                }
            }
            Self::Stop => {
                let stop_positions: Vec<Point> = world.friendly_team.all_robots().iter().enumerate().map(|(i, _)| {
                    world.ball.position + Vector::from_angle(Angle::full() / world.friendly_team.players().len() * i, 1.0)
                }).collect();
                RequestedTactics{
                    greedy: stop_positions.into_iter().map(|p| {
                        Tactic::Move((p, Angle::zero()))
                    }).collect(),
                    optimized: vec![]
                }
            },
            Self::Defense => RequestedTactics::new(),
        }
    }
}
