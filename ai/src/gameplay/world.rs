use crate::unwrap_or_err;
use crate::world::World as PartialWorld;
pub use crate::world::{Ball, Field, GameState, Robot, Team, TeamInfo};

// A varient of the base world that has all the data required to run gameplay.
// This is partially for convenience to avoid having to unwrap Option
// constantly in the gameplay code
#[derive(Clone)]
pub struct World {
    pub field: Field,
    pub ball: Ball,
    pub friendly_team: Team,
    pub enemy_team: Team,
    pub game_state: GameState,
    pub friendly_team_info: TeamInfo,
    pub enemy_team_info: TeamInfo,
}

impl World {
    pub fn from_partial_world(world: PartialWorld) -> Result<World, ()> {
        let field = unwrap_or_err!(world.field);
        let ball = unwrap_or_err!(world.ball);
        let friendly_team = world.friendly_team;
        let enemy_team = world.enemy_team;
        let game_state = world.game_state;
        let friendly_team_info = unwrap_or_err!(world.friendly_team_info);
        let enemy_team_info = unwrap_or_err!(world.enemy_team_info);

        Ok(World {
            field,
            ball,
            friendly_team,
            enemy_team,
            game_state,
            friendly_team_info,
            enemy_team_info,
        })
    }
}
