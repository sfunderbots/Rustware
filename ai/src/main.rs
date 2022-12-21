#![allow(dead_code)]
#![allow(unused_variables)]

extern crate core;

mod backend;
mod communication;
mod config;
mod constants;
mod evaluation;
mod experimental;
mod gameplay;
mod geom;
mod gui;
mod gui_bridge;
mod math;
mod motion;
mod perception;
mod proto;

use crate::communication::Node;
use crate::communication::{node_connection, NodeReceiver, NodeSender};
use crate::config::load_config;
use crate::geom::{Point, Vector};
use crate::math::{rect_sigmoid, sigmoid};
use crate::motion::{bb_time_to_position, Trajectory};
use crate::perception::game_state::Gamecontroller;
use multiqueue2;
use perception::{Field, Robot, World};
use prost::Message;
use protobuf;
use protobuf::reflect::rt::v2::make_oneof_copy_has_get_set_simpler_accessors;
use rand::Rng;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{sleep, JoinHandle};
use std::time::Duration;
use std::time::Instant;
use std::{fs, thread};

struct SynchronousNodes {
    perception: perception::Perception,
    gameplay: gameplay::Gameplay,
    backend: backend::SslSynchronousSimulator,
    gui_bridge: gui_bridge::GuiBridge,
}

struct AllNodeIo {
    perception_input: perception::Input,
    perception_output: perception::Output,
    gameplay_input: gameplay::Input,
    gameplay_output: gameplay::Output,
    backend_input: backend::Input,
    backend_output: backend::Output,
    gui_bridge_input: gui_bridge::Input,
    gui_bridge_output: gui_bridge::Output,
}

fn set_up_node_io() -> AllNodeIo {
    let (metrics_sender, metrics_receiver) = multiqueue2::broadcast_queue::<(String, f32)>(1000);
    let metrics_receiver = NodeReceiver::new(metrics_receiver);
    let (ssl_vision_proto_sender, ssl_vision_proto_receiver) =
        node_connection::<proto::ssl_vision::SslWrapperPacket>(
            10,
            metrics_sender.clone(),
            "ssl_vision".to_string(),
        );
    // let (foo, bar) = node_connection(10, metrics_sender.clone(), "ssl_vision".to_string());
    let (ssl_gc_referee_sender, ssl_gc_referee_receiver) =
        node_connection::<proto::ssl_gamecontroller::Referee>(
            10,
            metrics_sender.clone(),
            "ssl_gamecontroller".to_string(),
        );
    let (world_sender, world_receiver) =
        node_connection::<World>(1, metrics_sender.clone(), "world".to_string());
    let (gc_sender, gc_receiver) =
        node_connection::<Gamecontroller>(1, metrics_sender.clone(), "gamestate".to_string());
    let (trajectories_sender, trajectories_receiver) =
        node_connection::<std::collections::HashMap<usize, Trajectory>>(
            1,
            metrics_sender.clone(),
            "Trajectories".to_string(),
        );

    // All Inputs must call add_stream() before clone() so the data is copied to each receiver.
    // All Outputs should not call clone, since we only expect a single producer per queue
    let result = AllNodeIo {
        perception_input: perception::Input {
            ssl_vision_proto: ssl_vision_proto_receiver.add_stream().clone(),
            ssl_refbox_proto: ssl_gc_referee_receiver.add_stream().clone(),
        },
        perception_output: perception::Output {
            world: world_sender,
            gamecontroller: gc_sender,
        },
        gameplay_input: gameplay::Input {
            world: world_receiver.add_stream().clone(),
        },
        gameplay_output: gameplay::Output {
            trajectories: trajectories_sender,
        },
        backend_input: backend::Input {
            trajectories: trajectories_receiver.add_stream().clone(),
        },
        backend_output: backend::Output {
            ssl_vision_proto: ssl_vision_proto_sender,
            ssl_referee_proto: ssl_gc_referee_sender,
        },
        gui_bridge_input: gui_bridge::Input {
            ssl_vision_proto: ssl_vision_proto_receiver.add_stream().clone(),
            perception_world: world_receiver.add_stream().clone(),
            metrics: metrics_receiver.add_stream().clone(),
        },
        gui_bridge_output: gui_bridge::Output {},
    };

    // Drop the original readers - this removes them from the queues, meaning that the readers
    // in the new threads won't get starved by the lack of progress from recv
    ssl_vision_proto_receiver.unsubscribe();
    ssl_gc_referee_receiver.unsubscribe();
    trajectories_receiver.unsubscribe();
    world_receiver.unsubscribe();

    result
}

fn create_synchronous_nodes(io: AllNodeIo) -> SynchronousNodes {
    let config = Arc::new(Mutex::new(load_config().unwrap()));
    SynchronousNodes {
        perception: perception::Perception::new(
            io.perception_input,
            io.perception_output,
            Arc::clone(&config),
        ),
        gameplay: gameplay::Gameplay::new(io.gameplay_input, io.gameplay_output),
        backend: backend::SslSynchronousSimulator::new(io.backend_input, io.backend_output),
        gui_bridge: gui_bridge::GuiBridge::new(
            io.gui_bridge_input,
            io.gui_bridge_output,
            Arc::clone(&config),
        ),
    }
}

fn create_nodes_in_threads(io: AllNodeIo, should_stop: &Arc<AtomicBool>) -> Vec<JoinHandle<()>> {
    let config = Arc::new(Mutex::new(load_config().unwrap()));
    vec![
        perception::Perception::create_in_thread(
            io.perception_input,
            io.perception_output,
            &config,
            should_stop,
        ),
        gameplay::Gameplay::create_in_thread(io.gameplay_input, io.gameplay_output, should_stop),
        backend::SslNetworkListener::create_in_thread(io.backend_output, should_stop),
        backend::SslNetworkSimulator::create_in_thread(io.backend_input, should_stop),
        gui_bridge::GuiBridge::create_in_thread(
            io.gui_bridge_input,
            io.gui_bridge_output,
            &config,
            should_stop,
        ),
    ]
}

fn run_nodes_synchronously() {
    let mut nodes = create_synchronous_nodes(set_up_node_io());

    for i in 0..10 {
        // nodes.backend.send_dummy_data(i);
        nodes.perception.run_once();
        nodes.gameplay.run_once();
        nodes.backend.run_once();
    }
}

fn run_nodes_in_parallel_threads() {
    let mut should_stop = Arc::new(AtomicBool::new(false));
    let handles = create_nodes_in_threads(set_up_node_io(), &should_stop);

    println!("Sleeping to simulate working time");
    sleep(Duration::from_secs(500000));
    println!("Done sleeping. Sending stop signal");
    should_stop.store(true, Ordering::SeqCst);
    println!("About to join");
    for handle in handles {
        handle.join();
    }
    println!("Done join");
}

fn main() {
    // let cfg = config::load_config().expect("config should be fully initialized");
    // println!("{}", cfg.backend.ssl_vision_ip);
    // experimental::run();
    // run_nodes_synchronously();
    run_nodes_in_parallel_threads();

    // println!("square is {}", r);
}
