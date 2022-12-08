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
use std::thread::{JoinHandle, sleep};
use std::time::Duration;
use std::time::Instant;

struct AllNodes {
    perception: perception::Perception,
    gameplay: gameplay::Gameplay,
    backend: backend::Backend,
}

struct AllNodeIo {
    perception_input: perception::Input,
    perception_output: perception::Output,
    gameplay_input: gameplay::Input,
    gameplay_output: gameplay::Output,
    backend_input: backend::Input,
    backend_output: backend::Output,
}

fn set_up_node_io() -> AllNodeIo {
    let (ssl_vision_proto_sender, ssl_vision_proto_receiver) = multiqueue2::mpmc_queue::<i32>(100);
    let (world_sender, world_receiver) = multiqueue2::mpmc_queue::<i32>(100);
    let (trajectories_sender, trajectories_receiver) = multiqueue2::mpmc_queue::<i32>(100);

    AllNodeIo{
        perception_input: perception::Input {
            ssl_vision_proto: ssl_vision_proto_receiver.clone(),
        },
        perception_output: perception::Output {
            world: world_sender.clone(),
        },
        gameplay_input: gameplay::Input {
            world: world_receiver.clone(),
        },
        gameplay_output: gameplay::Output {
            trajectories: trajectories_sender.clone(),
        },
        backend_input: backend::Input {
            trajectories: trajectories_receiver.clone(),
        },
        backend_output: backend::Output {
            ssl_vision_proto: ssl_vision_proto_sender.clone(),
        },
    }
}

fn create_synchronous_nodes(io: AllNodeIo) -> AllNodes {
    AllNodes {
        perception: perception::Perception::new(
            io.perception_input, io.perception_output
        ),
        gameplay: gameplay::Gameplay::new(
            io.gameplay_input, io.gameplay_output
        ),
        backend: backend::Backend::new(
            io.backend_input, io.backend_output
        )
    }
}

fn create_nodes_in_threads(io: AllNodeIo, should_stop: &Arc<AtomicBool>) -> Vec<JoinHandle<()>> {
    vec![
        perception::Perception::create_in_thread(io.perception_input, io.perception_output, should_stop),
        gameplay::Gameplay::create_in_thread(io.gameplay_input, io.gameplay_output, should_stop),
        backend::Backend::create_in_thread(io.backend_input, io.backend_output, should_stop),
    ]
}

fn run_nodes_synchronously() {
    let mut nodes = create_synchronous_nodes(set_up_node_io());

    for i in 0..10 {
        nodes.backend.send_dummy_data(i);
        nodes.perception.run_once();
        nodes.gameplay.run_once();
        nodes.backend.run_once();
    }
}

fn run_nodes_in_parallel_threads() {
    let mut should_stop = Arc::new(AtomicBool::new(false));
    let handles = create_nodes_in_threads(set_up_node_io(), &should_stop);

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
