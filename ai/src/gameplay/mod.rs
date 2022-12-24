mod play;
mod tactic;

use crate::communication::{run_forever, Node, NodeReceiver, NodeSender};
use crate::motion::Trajectory;
use crate::perception::World;
use multiqueue2;
use crate::communication::take_last;
use play::{Play, RequestedTactics};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use strum::IntoEnumIterator;
use tactic::Tactic;
use crate::perception::game_state::Gamecontroller;

pub struct Input {
    pub world: NodeReceiver<World>,
    pub gamecontroller: NodeReceiver<Gamecontroller>,
}
pub struct Output {
    pub trajectories: NodeSender<HashMap<usize, Trajectory>>,
}

pub struct Gameplay {
    input: Input,
    output: Output,
    state: Option<State>,
}

impl Gameplay {
    pub fn new(input: Input, output: Output) -> Self {
        Self {
            input,
            output,
            state: None,
        }
    }

    pub fn create_in_thread(
        input: Input,
        output: Output,
        should_stop: &Arc<AtomicBool>,
    ) -> JoinHandle<()> {
        let should_stop = Arc::clone(should_stop);
        thread::spawn(move || {
            let node = Self::new(input, output);
            run_forever(Box::new(node), should_stop, "Gameplay");
        })
    }

    pub fn tick(&mut self) -> HashMap<usize, Trajectory> {
        // Update possession, ball model, etc.

        // Update current play
        update_current_play(self.state.as_mut().unwrap());

        // Get tactics

        // Optimize/assign tactics

        // Run tactics to get trajectories

        // Return trajectories

        HashMap::new()
    }


}

fn update_current_play(state: &mut State) {
    if !state.current_play.can_continue(&state.gamecontroller.game_state) {
        for p in Play::iter() {
            if p.can_start(&state.gamecontroller.game_state) {
                state.current_play = p;
                println!("Starting play: {}", state.current_play.to_string());
                return;
            }
        }
        state.current_play = Play::Halt;
        println!(
            "No play can start. Falling back to : {}",
            state.current_play.to_string()
        );
    }
}

impl Node for Gameplay {
    fn run_once(&mut self) -> Result<(), ()> {
        let mut world_updated = false;
        let mut gamecontroller_updated = false;

        if self.state.is_none() {
            let perception_world = match self.input.world.recv() {
                Ok(world) => {
                    world_updated = true;
                    world
                },
                _ => return Err(())
            };
            let gamecontroller = match self.input.gamecontroller.recv() {
                Ok(gc) => {
                    gamecontroller_updated = true;
                    gc
                },
                _ => return Err(())
            };
            self.state = Some(State::new(perception_world, gamecontroller));
        }

        if let Some(world) = take_last(&self.input.world)? {
            self.state.as_mut().unwrap().world = world;
            world_updated = true;
        }

        if let Some(gc) = take_last(&self.input.gamecontroller)? {
            self.state.as_mut().unwrap().gamecontroller = gc;
            gamecontroller_updated = true;
        }

        if gamecontroller_updated || world_updated {
            let trajectories = self.tick();
            self.output.trajectories.try_send(trajectories);
        }

        Ok(())
    }
}

struct State {
    world: World,
    gamecontroller: Gamecontroller,
    enemy_max_speed: f64,
    current_play: Play,
}

impl State {
    pub fn new(world: World, gamecontroller: Gamecontroller) -> Self {
        Self {
            world,
            gamecontroller,
            enemy_max_speed: 1.0, // Assume they can move somewhat
            current_play: Play::Halt,
        }
    }
}
