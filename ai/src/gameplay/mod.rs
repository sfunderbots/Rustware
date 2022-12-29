mod evaluation;
mod play;
mod tactic;
pub mod world;

use crate::communication::buffer::{NodeReceiver, NodeSender};
use crate::communication::node::Node;
use crate::gameplay::world::{Robot, World};
use crate::motion::Trajectory;
use crate::proto::config::Config;
use crate::run_nodes_in_parallel_threads;
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

fn optimized_tactic_assignment(
    mut tactics: Vec<Tactic>,
    robots: Vec<&Robot>,
) -> HashMap<usize, Tactic> {
    let mut assignments: HashMap<usize, Tactic> = HashMap::new();
    if robots.len() > tactics.len() {
        println!("More tactics requested to optimize than robots available");
        tactics.truncate(robots.len());
    }
    if !robots.is_empty() {
        let mut tactic_assignment_weights: Vec<f64> = vec![];
        // Each row holds the cost of assigning a robot to each tactic
        for t in &tactics {
            for r in &robots {
                // println!("{:?} : {} : {}", t, r.id, t.robot_assignment_cost(r));
                tactic_assignment_weights.push(t.robot_assignment_cost(r));
            }
        }
        let mut tactic_assignment_weights =
            WeightMatrix::from_row_vec(tactics.len(), tactic_assignment_weights);
        // println!("weights: {:?}", tactic_assignment_weights.as_slice());
        let munkres_assignments = munkres::solve_assignment(&mut tactic_assignment_weights)
            .unwrap_or_else(|e| {
                println!("Hungarian tactic optimization failed");
                vec![]
            });
        for a in munkres_assignments {
            let id = robots[a.column].id;
            let t = tactics[a.row].clone();
            // println!("Assigned {:?} to {}", t, id);
            assignments.insert(id, t);
        }
    }
    assignments
}

fn greedy_tactic_assignment(
    tactics: Vec<Tactic>,
    mut robots: &mut HashMap<usize, &Robot>,
) -> HashMap<usize, Tactic> {
    let mut assignments: HashMap<usize, Tactic> = HashMap::new();
    for t in tactics {
        if !robots.is_empty() {
            let (id, cost) = robots
                .iter()
                .map(|(_, r)| (r.id, t.robot_assignment_cost(r)))
                .min_by(|(id1, c1), (id2, c2)| c1.total_cmp(c2))
                .unwrap();
            assignments.insert(id, t);
            robots.remove(&id);
        } else {
            println!("Warning: More greedy tactics requested than robots available");
            break;
        }
    }
    assignments
}

fn assign_robots_to_tactics(
    tactics: RequestedTactics,
    mut robots: HashMap<usize, &Robot>,
) -> HashMap<usize, Tactic> {
    let mut assignments = greedy_tactic_assignment(tactics.greedy, &mut robots);
    let robots: Vec<&Robot> = robots.into_values().collect();
    assignments.extend(optimized_tactic_assignment(tactics.optimized, robots));
    assignments
}

impl Gameplay {
    pub fn tick(&mut self, world: &World) -> HashMap<usize, Trajectory> {
        // Update possession, ball model, etc.

        // Update current play
        self.update_current_play(world);

        // Get tactics
        let requested_tactics = self.state.current_play.run(&world, &self.state);

        // Optimize/assign tactics
        let unassigned_robots: HashMap<usize, &Robot> = world
            .friendly_team
            .all_robots()
            .iter()
            .map(|r| (r.id, *r))
            .collect();
        let robot_tactic_assignment =
            assign_robots_to_tactics(requested_tactics, unassigned_robots);

        // Run tactics to get trajectories
        let trajectories: HashMap<usize, Trajectory> = robot_tactic_assignment
            .iter()
            .map(|(id, t)| {
                (
                    *id,
                    t.run(world.friendly_team.robot(&id).unwrap(), &world, &self.state),
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
