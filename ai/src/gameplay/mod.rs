mod play;
mod tactic;

use crate::communication::{run_forever, Node, NodeReceiver, NodeSender};
use crate::motion::Trajectory;
use crate::perception::World;
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
    pub world: NodeReceiver<World>,
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

    pub fn tick(&mut self, world: World) -> HashMap<i32, Trajectory> {
        // Update possession, ball model, etc.

        // Update current play
        self.update_current_play();

        // Get tactics

        // Optimize/assign tactics

        // Run tactics to get trajectories

        // Return trajectories

        HashMap::new()
    }

    fn update_current_play(&mut self) {
        if !self.state.current_play.can_continue() {
            for p in Play::iter() {
                if p.can_start() {
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
        let packet = match self.input.world.try_recv() {
            Ok(p) => p,
            Err(e) => match e {
                std::sync::mpsc::TryRecvError::Empty => return Ok(()),
                std::sync::mpsc::TryRecvError::Disconnected => {
                    println!("Breaking perception loop");
                    return Err(());
                }
            },
        };
        // println!("Gameplay got packet {}", packet);
        // self.output.trajectories.try_send(packet);
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
