mod evaluation;
mod play;
mod tactic;
pub mod world;

use crate::communication::node::Node;
use crate::communication::buffer::{NodeSender, NodeReceiver};
use crate::gameplay::world::{Robot, World};
use crate::motion::Trajectory;
use crate::world::World as PartialWorld;
use multiqueue2;
use munkres::WeightMatrix;
use play::{Play, RequestedTactics};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use strum::IntoEnumIterator;
use tactic::Tactic;
use crate::proto::config::Config;

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
    pub fn tick(&mut self, world: &World) -> HashMap<usize, Trajectory> {
        // Update possession, ball model, etc.

        // Update current play
        self.update_current_play(world);

        // Get tactics
        let requested_tactics = self.state.current_play.run(&world, &self.state);

        // Optimize/assign tactics
        let mut robot_tactic_assigment: HashMap<usize, Tactic> = HashMap::new();
        // let mut unnasigned_robot_ids: HashSet<usize> = HashSet::from_iter(world.friendly_team.all_robots().iter().map(|r| {r.id}));
        let mut unassigned_robots: HashMap<usize, Robot> = world
            .friendly_team
            .all_robots()
            .iter()
            .map(|r| (r.id, (*r).clone()))
            .collect();

        // Greedy assignment
        for t in requested_tactics.greedy {
            if !unassigned_robots.is_empty() {
                let (id, cost) = unassigned_robots
                    .iter()
                    .map(|(_, r)| (r.id, t.robot_assignment_cost(r)))
                    .min_by(|(id1, c1), (id2, c2)| c1.total_cmp(c2))
                    .unwrap();
                robot_tactic_assigment.insert(id, t);
                unassigned_robots.remove(&id);
            } else {
                println!("Warning: More greedy tactics requested than robots available");
                break;
            }
        }

        // Optimized assignment
        let mut tactics_to_optimize = requested_tactics.optimized;
        if unassigned_robots.len() > tactics_to_optimize.len() {
            println!("More tactics requested to optimize than robots available");
            tactics_to_optimize.truncate(unassigned_robots.len());
        }
        if !unassigned_robots.is_empty() {
            // let mut tactic_assignment_weights = Matrix::new_square(tactics_to_optimize.len(), float_ord::FloatOrd(0.0));
            // let mut tactic_assignment_weights = WeightMatrix::from_row_vec(2, vec![1, 2, 3, 4]);
            let mut tactic_assignment_weights: Vec<f64> = vec![];
            println!("{:?}", tactic_assignment_weights);
            // Each row holds the cost of assigning a robot to each tactic
            for t in &tactics_to_optimize {
                for (id, r) in &unassigned_robots {
                    tactic_assignment_weights.push(t.robot_assignment_cost(r));
                }
            }
            let mut tactic_assignment_weights = WeightMatrix::from_row_vec(
                tactics_to_optimize.len(),
                tactic_assignment_weights,
            );
            let assignments = munkres::solve_assignment(&mut tactic_assignment_weights)
                .unwrap_or_else(|e| {
                    println!("Hungarian tactic optimization failed");
                    vec![]
                });
            for a in assignments {
                let r = unassigned_robots.get(&a.column).unwrap().id;
                let t = tactics_to_optimize[a.row].clone();
                robot_tactic_assigment.insert(r, t);
            }
        }

        // Run tactics to get trajectories
        let trajectories: HashMap<usize, Trajectory> = robot_tactic_assigment
            .iter()
            .map(|(id, t)| {
                (
                    *id,
                    t.run(world.friendly_team.robot(id).unwrap(), &world, &self.state),
                )
            })
            .collect();

        // Return trajectories
        trajectories
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
    type Input = Input;
    type Output = Output;
    fn run_once(&mut self) -> Result<(), ()> {
        let partial_world = match self.input.world.recv() {
            Ok(world) => world,
            Err(_) => return Err(()),
        };
        let world = match World::from_partial_world(partial_world) {
            Ok(w) => w,
            Err(_) => return Ok(()),
        };
        let trajectories = self.tick(&world);
        self.output.trajectories.try_send(trajectories);

        Ok(())
    }

    fn new(input: Self::Input, output: Self::Output, config: Arc<Mutex<Config>>) -> Self {
        Self {
            input,
            output,
            state: State::new(),
        }
    }

    fn name() -> String {
        "Gameplay".to_string()
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
