mod game_state;
mod vision;

pub use vision::{Ball, Team, Field, Robot};
pub use game_state::{GameState, TeamInfo};

#[derive(Clone)]
pub struct World {
    pub field: Option<Field>,
    pub ball: Option<Ball>,
    pub friendly_team: Team,
    pub enemy_team: Team,
    pub game_state: GameState,
    pub friendly_team_info: Option<TeamInfo>,
    pub enemy_team_info: Option<TeamInfo>
}

