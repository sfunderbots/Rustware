mod play;
mod tactic;

use crate::communication::{run_forever, Node};
use crate::motion::Trajectory;
use crate::world::World;
use multiqueue2;
use play::{Halt, Play, RequestedTactics, Stop};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
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
    play_builders: Vec<Box<dyn Fn() -> Box<dyn Play>>>,
}

impl Gameplay {
    pub fn new(input: Input, output: Output) -> Self {
        Self {
            input,
            output,
            state: State::new(),
            play_builders: vec![
                Box::new(|| {Box::new(Halt{})}),
                Box::new(|| {Box::new(Stop{})}),
            ]
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
            // TODO: inefficient because the play has to be allocated
            // just to check if it can start
            for builder in &self.play_builders {
                let play = builder();
                if play.can_start() {
                    self.state.current_play = play;
                    println!("Starting play: {}", self.state.current_play.name());
                    return;
                }
            }
            self.state.current_play = Box::new(Halt{});
            println!("No play can start. Falling back to : {}", self.state.current_play.name());
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
        println!("Gameplay got packet {}", packet);
        self.output.trajectories.try_send(packet);
        Ok(())
    }
}

struct State {
    enemy_max_speed: f32,
    current_play: Box<dyn Play>
}

impl State {
    pub fn new() -> Self {
        Self {
            enemy_max_speed: 1.0, // Assume they can move somewhat
            current_play: Box::new(Halt{})
        }
    }
}
