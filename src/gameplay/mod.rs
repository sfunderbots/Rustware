mod play;
mod tactic;

use crate::communication::{Node, run_forever};
use multiqueue2;
use crate::motion::Trajectory;
use crate::world::World;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use play::{Play, RequestedTactics, Halt, Stop};
use tactic::Tactic;

pub struct Input {
    pub world: multiqueue2::MPMCReceiver<i32>,
}
pub struct Output {
    pub trajectories: multiqueue2::MPMCSender<i32>,
}

pub struct Gameplay {
    input: Input,
    output: Output,
    state: State,
    available_plays: Vec<Box<dyn Play>>
}



impl Gameplay {
    pub fn new(input: Input, output: Output) -> Self {
        Self{
            input: input,
            output: output,
            state: State::new(),
            available_plays: vec![Box::new(Halt{}), Box::new(Stop{})]
        }
    }

    pub fn create_in_thread(input: Input, output: Output, should_stop: &Arc<AtomicBool>) -> JoinHandle<()> {
        let should_stop = Arc::clone(should_stop);
        thread::spawn(move || {
            let node = Self::new(input, output);
            run_forever(Box::new(node), should_stop, "Gameplay");
        })
    }

    pub fn tick(world: World) -> HashMap<i32, Trajectory> {
        // Update possession, ball model, etc.

        // Update current play

        // Get tactics

        // Optimize/assign tactics

        // Run tactics to get trajectories

        // Return trajectories

        HashMap::new()
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
        println!("Gameplay got packet {}", packet);
        self.output.trajectories.try_send(packet);
        Ok(())
    }
}

struct State {
    enemy_max_speed: f32,
}

impl State {
    pub fn new() -> Self{
        Self{
            enemy_max_speed: 1.0 // Assume they can move somewhat
        }
    }
}

