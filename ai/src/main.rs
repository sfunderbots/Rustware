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
use crate::geom::{Point, Vector};
use crate::math::{rect_sigmoid, sigmoid};
use crate::motion::{bb_time_to_position, Trajectory};
use multiqueue2;
use perception::{Field, Robot, World};
use prost::Message;
use protobuf;
use protobuf::reflect::rt::v2::make_oneof_copy_has_get_set_simpler_accessors;
use rand::Rng;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::{sleep, JoinHandle};
use std::time::Duration;
use std::time::Instant;
use std::{fs, thread};
use zmq;
use cxx::UniquePtr;
use std::pin::Pin;

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
    let (ssl_vision_proto_sender, ssl_vision_proto_receiver) =
        multiqueue2::broadcast_queue::<proto::ssl_vision::SslWrapperPacket>(10);
    let (ssl_gc_referee_sender, ssl_gc_referee_receiver) =
        multiqueue2::broadcast_queue::<proto::ssl_gamecontroller::Referee>(10);
    let (world_sender, world_receiver) = multiqueue2::broadcast_queue::<World>(10);
    let (trajectories_sender, trajectories_receiver) =
        multiqueue2::broadcast_queue::<std::collections::HashMap<usize, Trajectory>>(10);

    // All Inputs must call add_stream() before clone() so the data is copied to each receiver.
    // All Outputs should not call clone, since we only expect a single producer per queue
    let result = AllNodeIo {
        perception_input: perception::Input {
            ssl_vision_proto: ssl_vision_proto_receiver.add_stream().clone(),
            ssl_refbox_proto: ssl_gc_referee_receiver.add_stream().clone(),
        },
        perception_output: perception::Output {
            world: world_sender,
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
    SynchronousNodes {
        perception: perception::Perception::new(io.perception_input, io.perception_output),
        gameplay: gameplay::Gameplay::new(io.gameplay_input, io.gameplay_output),
        backend: backend::SslSynchronousSimulator::new(io.backend_input, io.backend_output),
        gui_bridge: gui_bridge::GuiBridge::new(io.gui_bridge_input, io.gui_bridge_output),
    }
}

fn create_nodes_in_threads(io: AllNodeIo, should_stop: &Arc<AtomicBool>) -> Vec<JoinHandle<()>> {
    vec![
        perception::Perception::create_in_thread(
            io.perception_input,
            io.perception_output,
            should_stop,
        ),
        gameplay::Gameplay::create_in_thread(io.gameplay_input, io.gameplay_output, should_stop),
        backend::SslNetworkListener::create_in_thread(io.backend_output, should_stop),
        backend::SslNetworkSimulator::create_in_thread(io.backend_input, should_stop),
        gui_bridge::GuiBridge::create_in_thread(
            io.gui_bridge_input,
            io.gui_bridge_output,
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

// use libc::size_t;
//
// #[link(name = "mathewtest")]
// extern "C" {
//     fn mysquarefunc(val: size_t) -> size_t;
// }
//
// fn main() {
//     // let cfg = config::load_config().expect("Config should be fully initialized");
//     // println!("{}", cfg.backend.ssl_vision_ip);
//     // experimental::run();
//     // run_nodes_synchronously();
//     // run_nodes_in_parallel_threads();
//
//     println!("Hellow world");
//     let r = unsafe { mysquarefunc(4)};
//     // println!("square is {}", r);
// }

// use libc::size_t;

// #[link(name = "snappy")]
// extern {
//     fn snappy_max_compressed_length(source_length: size_t) -> size_t;
// }

// #[link(name = "mathewtest")]
// extern "C" {
//     fn mysquare(val: i32) -> i32;
// }
//
// // #[link(name = "mathewtestcpp")]
// // extern "C" {
// //     fn mysquarecpp(val: i32) -> i32;
// //     // type Simulator;
// //     // fn new_simulator() -> UniquePtr<Simulator>;
// // }
//
// #[link(name = "hello")]
// extern "C" {
//     fn square(val: i32) -> i32;
// }
//

// #[link(name = "snappy")]
// extern "C" {
//     // fn square(val: i32) -> i32;
//
// }

#[cxx::bridge()]
mod ffi {
    unsafe extern "C++" {
        include!("rustware/src/ersim_wrapper/ersim.h");
        fn ersquare(x: i32) -> i32;
        type SimulatorWrapper;
        fn new_simulator_wrapper() -> UniquePtr<SimulatorWrapper>;
        fn set(&self, x: i32);
        fn get(&self) -> i32;
    }
}

fn main() {
    // let x = unsafe { snappy_max_compressed_length(100) };
    // println!("max compressed length of a 100 byte buffer: {}", x);
    // let y = unsafe { mysquare(100) };
    // println!("square: {}", y);
    // let y = unsafe { mysquarecpp(100) };
    // println!("square: {}", y);
    // let z = unsafe { square(100) };
    // println!("square: {}", z);
    let foo = ffi::ersquare(40);
    println!("square: {}", foo);
    let wrapper = ffi::new_simulator_wrapper();
    println!("{}", wrapper.get());
    wrapper.set(5);
    println!("{}", wrapper.get());
}