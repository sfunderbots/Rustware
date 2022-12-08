#![allow(dead_code)]
#![allow(unused_variables)]

mod backend;
mod communication;
mod evaluation;
mod experimental;
mod gameplay;
mod geom;
mod math;
mod motion;
mod perception;
mod world;

use crate::communication::Node;
use crate::geom::{Point, Vector};
use crate::math::{rect_sigmoid, sigmoid};
use crate::motion::bb_time_to_position;
use crate::world::{Field, Robot};
use multiqueue2;
use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

struct AllNodes {
    perception: perception::Perception,
    gameplay: gameplay::Gameplay,
    backend: backend::Backend,
}

fn set_up_nodes() -> AllNodes {
    let (ssl_vision_proto_sender, ssl_vision_proto_receiver) = multiqueue2::mpmc_queue::<i32>(100);
    let (world_sender, world_receiver) = multiqueue2::mpmc_queue::<i32>(100);
    let (trajectories_sender, trajectories_receiver) = multiqueue2::mpmc_queue::<i32>(100);

    let nodes = AllNodes {
        perception: perception::Perception {
            input: perception::Input {
                ssl_vision_proto: ssl_vision_proto_receiver.clone(),
            },
            output: perception::Output {
                world: world_sender.clone(),
            },
        },
        gameplay: gameplay::Gameplay {
            input: gameplay::Input {
                world: world_receiver.clone(),
            },
            output: gameplay::Output {
                trajectories: trajectories_sender.clone(),
            },
        },
        backend: backend::Backend {
            input: backend::Input {
                trajectories: trajectories_receiver.clone(),
            },
            output: backend::Output {
                ssl_vision_proto: ssl_vision_proto_sender.clone(),
            },
        },
    };
    nodes
}

fn run_nodes_synchronously() {
    let mut nodes = set_up_nodes();

    for i in 0..100 {
        nodes.backend.send_dummy_data(i);
        nodes.perception.run_once();
        nodes.gameplay.run_once();
        nodes.backend.run_once();
    }
}

fn create_node_thread(
    mut n: Box<dyn Node + Send>,
    should_stop: &Arc<AtomicBool>,
    name: String,
) -> thread::JoinHandle<()> {
    let should_stop = Arc::clone(should_stop);
    let handles = thread::spawn(move || loop {
        match n.run_once() {
            Err(_) => {
                println!("Terminating node {}", name);
                break;
            }
            _ => (),
        }
        if should_stop.load(Ordering::SeqCst) {
            println!("Terminating node {}", name);
            break;
        }
    });
    handles
}

fn run_nodes_in_parallel_threads() {
    let mut nodes = set_up_nodes();

    let mut should_stop = Arc::new(AtomicBool::new(false));

    let handles = vec![
        create_node_thread(
            Box::new(nodes.perception),
            &should_stop,
            "Perception".to_string(),
        ),
        create_node_thread(
            Box::new(nodes.gameplay),
            &should_stop,
            "Gameplay".to_string(),
        ),
        create_node_thread(Box::new(nodes.backend), &should_stop, "Backend".to_string()),
    ];

    println!("Sleeping to simulate working time");
    sleep(Duration::from_secs(2));
    println!("Done sleeping. Sending stop signal");
    should_stop.store(true, Ordering::SeqCst);
    println!("About to join");
    for handle in handles {
        handle.join();
    }
    println!("Done join");
}

fn main() {
    // experimental::run();
    run_nodes_synchronously();
    // run_nodes_in_parallel_threads();
}
