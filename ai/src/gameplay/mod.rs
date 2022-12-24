mod evaluation;
mod play;
mod tactic;
pub mod world;

use crate::communication::take_last;
use crate::communication::{run_forever, Node, NodeReceiver, NodeSender};
use crate::gameplay::world::World;
use crate::motion::Trajectory;
use crate::world::World as PartialWorld;
use multiqueue2;
use play::{Play, RequestedTactics};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use strum::IntoEnumIterator;
use tactic::Tactic;

pub struct Input {
    pub world: NodeReceiver<PartialWorld>,
}
pub struct Output {
    pub trajectories: NodeSender<HashMap<usize, Trajectory>>,
}

pub struct Gameplay {
    input: Input,
    output: Output,
    state: State,
}

impl Gameplay {
    pub fn new(input: Input, output: Output) -> Self {
        Self {
            input,
            output,
            state: State::new(),
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

    pub fn tick(&mut self, world: &World) -> HashMap<usize, Trajectory> {
        // Update possession, ball model, etc.

        // Update current play
        self.update_current_play(world);

        // Get tactics
        let requested_tactics = self.state.current_play.run(&world, &self.state);

        // Optimize/assign tactics

        // Run tactics to get trajectories

        // Return trajectories

        HashMap::new()
    }

    fn update_current_play(&mut self, world: &World) {
        if !self.state.current_play.can_continue(&world.game_state) {
            for p in Play::iter() {
                if p.can_start(&world.game_state) {
                    self.state.current_play = p;
                    println!("Starting play: {}", self.state.current_play.to_string());
                    return;
                }
            }
            self.state.current_play = Play::Halt;
            println!(
                "No play can start. Falling back to : {}",
                self.state.current_play.to_string()
            );
        }
    }
}

impl Node for Gameplay {
    fn run_once(&mut self) -> Result<(), ()> {
        let partial_world = match self.input.world.recv() {
            Ok(world) => world,
            Err(_) => return Err(()),
        };
        let world = World::from_partial_world(partial_world)?;
        let trajectories = self.tick(&world);
        self.output.trajectories.try_send(trajectories);

        Ok(())
    }
}

struct State {
    enemy_max_speed: f64,
    current_play: Play,
}

impl State {
    pub fn new() -> Self {
        Self {
            enemy_max_speed: 1.0, // Assume they can move somewhat
            current_play: Play::Halt,
        }
    }
}
