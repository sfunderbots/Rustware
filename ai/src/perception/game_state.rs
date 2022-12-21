use std::cell::Ref;
use crate::geom::Point;
use crate::perception::game_state::PlayState::Halt;
use crate::perception::game_state::RestartReason::Kickoff;
use crate::perception::Team;
use crate::proto::ssl_gamecontroller::{Referee, referee, referee::Command};
use crate::proto::config;
use crate::proto::config::Perception;

fn is_friendly_team_blue(referee: Option<&Referee>, config: &Perception) -> Option<bool> {
    match config::FriendlyColor::from_i32(config.friendly_color) {
        Some(config::FriendlyColor::AutorefColor) => {
            if let Some(msg) = referee {
                if msg.blue.name.to_lowercase() == config.team_name.to_lowercase() {
                    Some(true)
                } else if msg.yellow.name.to_lowercase() == config.team_name.to_lowercase() {
                    Some(false)
                } else {
                    println!("Friendly team name not found in either referee team names");
                    None
                }
            }else {
                None
            }
        },
        Some(config::FriendlyColor::Blue) => Some(true),
        Some(config::FriendlyColor::Yellow) => Some(false),
        None => panic!("Should be impossible to get invalid team color from config")
    }
}

fn is_friendly_team_defending_positive_side(referee: Option<&Referee>, config: &Perception, is_friendly_team_blue: bool) -> Option<bool> {
    match config::DefendingSide::from_i32(config.defending_side) {
        Some(config::DefendingSide::AutorefSide) => {
            if let Some(msg) = referee {
                if let Some(msg_blue_team_on_positive_half) = msg.blue_team_on_positive_half {
                    Some(msg_blue_team_on_positive_half == is_friendly_team_blue)
                }else {
                    println!("Autoref hasn't specified a side to defend");
                    None
                }
            }else {
                None
            }
        },
        Some(config::DefendingSide::Negative) => Some(false),
        Some(config::DefendingSide::Positive) => Some(true),
        None => panic!("Should be impossible to get invalid defending side from config")
    }
}

#[derive(Clone)]
pub struct TeamInfo {
    pub is_blue: bool,
    pub score: usize,
    pub goalie_id: usize,
    pub defending_positive_side: bool,
}

impl TeamInfo {
    pub fn from_referee(referee: Option<&Referee>, config: &Perception, is_friendly_team: bool) -> Option<TeamInfo> {
        let is_friendly_team_blue = is_friendly_team_blue(referee, config)?;
        let is_friendly_team_defending_positive_side = is_friendly_team_defending_positive_side(referee, config, is_friendly_team_blue)?;
        let is_blue = is_friendly_team_blue == is_friendly_team;
        let defending_positive_side = is_friendly_team_defending_positive_side == is_friendly_team;
        if let Some(msg) = referee {
            let info = if is_blue {&msg.blue} else {&msg.yellow};
            Some(TeamInfo{
                is_blue,
                defending_positive_side,
                score: info.score as usize,
                goalie_id: info.goalkeeper as usize,
            })
        }else {
            None
        }
    }
}

#[derive(PartialEq, Clone)]
enum PlayState {
    Halt,
    Stop,
    Setup,
    Ready,
    Playing
}

#[derive(PartialEq, Clone)]
enum RestartReason {
    None,
    Kickoff,
    FreeKick,
    Penalty,
    BallPlacement
}

#[derive(Clone)]
pub struct GameState {
    play_state: PlayState,
    restart_reason: RestartReason,
    our_restart: bool,
    ball_position_at_restart: Option<Point>,
}

impl GameState {
    pub fn new() -> GameState {
        GameState{
            play_state: PlayState::Halt,
            restart_reason: RestartReason::None,
            our_restart: false,
            ball_position_at_restart: None
        }
    }

    pub fn update_command(&mut self, command: Command, is_friendly_team_blue: bool) {
        match command {
            Command::Halt => {
                self.play_state = PlayState::Halt;
                self.restart_reason = RestartReason::None;
            }
            Command::Stop => {
                self.play_state = PlayState::Stop;
                self.restart_reason = RestartReason::None;
                self.our_restart = false;
            }
            Command::NormalStart => {
                self.play_state = PlayState::Ready;
            }
            Command::ForceStart => {
                self.play_state = PlayState::Playing;
                self.restart_reason = RestartReason::None;
            }
            Command::PrepareKickoffYellow => {
                self.play_state = PlayState::Setup;
                self.restart_reason = RestartReason::Kickoff;
                self.our_restart = !is_friendly_team_blue;
            }
            Command::PrepareKickoffBlue => {
                self.play_state = PlayState::Setup;
                self.restart_reason = RestartReason::Kickoff;
                self.our_restart = is_friendly_team_blue;
            }
            Command::PreparePenaltyYellow => {
                self.play_state = PlayState::Setup;
                self.restart_reason = RestartReason::Penalty;
                self.our_restart = !is_friendly_team_blue;
            }
            Command::PreparePenaltyBlue => {
                self.play_state = PlayState::Setup;
                self.restart_reason = RestartReason::Penalty;
                self.our_restart = is_friendly_team_blue;
            }
            Command::DirectFreeYellow | Command::IndirectFreeYellow => {
                // Indirect free kicks have been removed and merged with direct free kicks
                self.play_state = PlayState::Ready;
                self.restart_reason = RestartReason::FreeKick;
                self.our_restart = !is_friendly_team_blue;
            }
            Command::DirectFreeBlue | Command::IndirectFreeBlue => {
                // Indirect free kicks have been removed and merged with direct free kicks
                self.play_state = PlayState::Ready;
                self.restart_reason = RestartReason::FreeKick;
                self.our_restart = is_friendly_team_blue;
            }
            Command::TimeoutYellow | Command::TimeoutBlue => {
                self.play_state = PlayState::Halt;
                self.restart_reason = RestartReason::None;
            }
            Command::GoalBlue | Command::GoalYellow => {
                // Deprecated: For compliance, teams must treat as STOP
                self.play_state = PlayState::Stop;
                self.restart_reason = RestartReason::None;
            }
            Command::BallPlacementBlue => {
                self.play_state = PlayState::Setup;
                self.restart_reason = RestartReason::BallPlacement;
                self.our_restart = is_friendly_team_blue;
            }
            Command::BallPlacementYellow => {
                self.play_state = PlayState::Setup;
                self.restart_reason = RestartReason::BallPlacement;
                self.our_restart = !is_friendly_team_blue;
            }
        }
    }

    fn update_is_restart_completed(&mut self, ball_position: Point, config: config::Rules) {
        match self.play_state {
            PlayState::Setup => {
                self.ball_position_at_restart = Some(ball_position.clone());
            }
            PlayState::Ready => {
                if let Some(pos_at_restart) = self.ball_position_at_restart {
                    if (pos_at_restart - ball_position).length() > config.ball_in_play_after_restart_move_dist {
                        self.set_restart_completed()
                    }
                }
            }
            _ => ()
        }
    }

    fn set_restart_completed(&mut self) {
        self.play_state = PlayState::Playing;
        self.restart_reason = RestartReason::None;
        self.ball_position_at_restart = None;
    }

    pub fn halted(&self) -> bool {
        self.play_state == PlayState::Halt
    }

    pub fn stopped(&self) -> bool {
        self.play_state == PlayState::Stop
    }

    pub fn playing(&self) -> bool {
        self.play_state == PlayState::Playing
    }
    pub fn kickoff(&self) -> bool {
        self.restart_reason == RestartReason::Kickoff
    }
    pub fn penalty(&self) -> bool {
        self.restart_reason == RestartReason::Penalty
    }
    pub fn ball_placement(&self) -> bool {
        self.restart_reason == RestartReason::BallPlacement
    }
    pub fn our_restart(&self) -> bool {
        self.our_restart && self.restart_reason != RestartReason::None
    }
    pub fn our_kickoff(&self) -> bool {
        self.kickoff() && self.our_restart
    }
    pub fn their_kickoff(&self) -> bool {
        self.kickoff() && !self.our_restart
    }
    pub fn our_penalty(&self) -> bool {
        self.penalty() && self.our_restart
    }
    pub fn their_penalty(&self) -> bool {
        self.penalty() && !self.our_restart
    }
    pub fn free_kick(&self) -> bool {
        self.restart_reason == RestartReason::FreeKick
    }
    pub fn our_free_kick(&self) -> bool {
        self.free_kick() && self.our_restart
    }
    pub fn their_free_kick(&self) -> bool {
        self.free_kick() && !self.our_restart
    }
    pub fn our_ball_placement(&self) -> bool {
        self.ball_placement() && self.our_restart
    }
    pub fn their_ball_placement(&self) -> bool {
        self.ball_placement() && !self.our_restart
    }
    pub fn can_manipulate_ball(&self) -> bool {
        (self.play_state == PlayState::Playing) || (self.our_restart && self.play_state == PlayState::Ready)
    }
    pub fn friendly_stay_away_from_ball(&self) -> bool {
        (self.play_state != PlayState::Playing) && !self.our_restart
    }
    pub fn stay_on_side(&self) -> bool {
        // Our robots must stay on our half of the field
        let ps = match self.play_state {
            PlayState::Setup | PlayState::Ready => true,
            _ => false
        };
        ps && (self.restart_reason == RestartReason::Kickoff) && !self.our_restart
    }
    pub fn stay_behind_penalty_line(&self) -> bool {
        self.restart_reason == RestartReason::Penalty
    }
}

#[derive(Clone)]
pub struct Gamecontroller {
    pub game_state: GameState,
    pub friendly_team_info: TeamInfo,
    pub enemy_team_info: TeamInfo
}