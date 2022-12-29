use crate::communication_old::Node;
use crate::perception;
use crate::gameplay;
use crate::backend;
use crate::gui_bridge;
use crate::proto;
use crate::communication_old::{node_connection, NodeReceiver, NodeSender};
use crate::config::load_config;
use crate::geom::{Point, Vector};
use crate::math::{rect_sigmoid, sigmoid};
use crate::motion::{bb_time_to_position, Trajectory};
use crate::world::World;
use multiqueue2;
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
use crate::proto::ssl_simulation::SimulatorControl;
use crate::simulation::simulated_test_runner;

pub struct SynchronousNodes {
    pub perception: perception::Perception,
    pub gameplay: gameplay::Gameplay,
    pub backend: backend::SslSynchronousSimulator,
    pub gui_bridge: gui_bridge::GuiBridge,
}

pub struct AllNodeIo {
    pub perception_input: perception::Input,
    pub perception_output: perception::Output,
    pub gameplay_input: gameplay::Input,
    pub gameplay_output: gameplay::Output,
    pub backend_input: backend::Input,
    pub backend_output: backend::Output,
    pub gui_bridge_input: gui_bridge::Input,
    pub gui_bridge_output: gui_bridge::Output,
    pub sim_test_runner_input: simulated_test_runner::Input,
    pub sim_test_runner_output: simulated_test_runner::Output,
}

pub fn set_up_node_io() -> AllNodeIo {
    let (metrics_sender, metrics_receiver) = multiqueue2::broadcast_queue::<(String, f64)>(1000);
    // Use for channels we don't want metrics for
    let (void_metrics_sender, void_metrics_receiver) = multiqueue2::broadcast_queue::<(String, f64)>(1000);
    let metrics_receiver = NodeReceiver::new(metrics_receiver);
    let (ssl_vision_sender, ssl_vision_receiver) =
        node_connection::<proto::ssl_vision::SslWrapperPacket>(
            20,
            metrics_sender.clone(),
            "ssl_vision".to_string(),
        );
    let (ssl_gc_sender, ssl_gc_receiver) = node_connection::<proto::ssl_gamecontroller::Referee>(
        20,
        metrics_sender.clone(),
        "ssl_gamecontroller".to_string(),
    );
    let (world_sender, world_receiver) =
        node_connection::<World>(1, metrics_sender.clone(), "vision".to_string());
    let (trajectories_sender, trajectories_receiver) =
        node_connection::<std::collections::HashMap<usize, Trajectory>>(
            1,
            metrics_sender.clone(),
            "Trajectories".to_string(),
        );
    let (sim_control_sender, sim_control_receiver) = node_connection::<SimulatorControl>(10, void_metrics_sender.clone(), "Simuator Control".to_string());

    // All Inputs must call add_stream() before clone() so the data is copied to each receiver.
    // All Outputs should not call clone, since we only expect a single producer per queue
    let result = AllNodeIo {
        perception_input: perception::Input {
            ssl_vision: ssl_vision_receiver.add_stream().clone(),
            ssl_gc: ssl_gc_receiver.add_stream().clone(),
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
            world: world_receiver.add_stream().clone(),
            sim_control: sim_control_receiver.add_stream().clone()
        },
        backend_output: backend::Output {
            ssl_vision: ssl_vision_sender,
            ssl_gc: ssl_gc_sender,
        },
        gui_bridge_input: gui_bridge::Input {
            ssl_vision: ssl_vision_receiver.add_stream().clone(),
            world: world_receiver.add_stream().clone(),
            trajectories: trajectories_receiver.add_stream().clone(),
            metrics: metrics_receiver.add_stream().clone(),
        },
        gui_bridge_output: gui_bridge::Output {
            sim_control: sim_control_sender.clone(),
        },
        sim_test_runner_input: simulated_test_runner::Input{
            world: world_receiver.add_stream().clone()
        },
        sim_test_runner_output: simulated_test_runner::Output{
            sim_control: sim_control_sender
        }
    };

    // Drop the original readers - this removes them from the queues, meaning that the readers
    // in the new threads won't get starved by the lack of progress from recv
    ssl_vision_receiver.unsubscribe();
    ssl_gc_receiver.unsubscribe();
    trajectories_receiver.unsubscribe();
    world_receiver.unsubscribe();

    result
}

pub fn create_synchronous_nodes(io: AllNodeIo) -> SynchronousNodes {
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

pub fn create_nodes_in_threads(io: AllNodeIo, should_stop: &Arc<AtomicBool>) -> Vec<JoinHandle<()>> {
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
        backend::SslNetworkSimulator::create_in_thread(io.backend_input, &config, should_stop),
        gui_bridge::GuiBridge::create_in_thread(
            io.gui_bridge_input,
            io.gui_bridge_output,
            &config,
            should_stop,
        ),
    ]
}

pub fn create_test_nodes_in_threads(io: AllNodeIo, should_stop: &Arc<AtomicBool>) -> (Vec<JoinHandle<()>>, simulated_test_runner::SimulatedTestRunner) {
    let config = Arc::new(Mutex::new(load_config().unwrap()));
    (vec![
        perception::Perception::create_in_thread(
            io.perception_input,
            io.perception_output,
            &config,
            should_stop,
        ),
        gameplay::Gameplay::create_in_thread(io.gameplay_input, io.gameplay_output, should_stop),
        backend::SslNetworkListener::create_in_thread(io.backend_output, should_stop),
        backend::SslNetworkSimulator::create_in_thread(io.backend_input, &config, should_stop),
        gui_bridge::GuiBridge::create_in_thread(
            io.gui_bridge_input,
            io.gui_bridge_output,
            &config,
            should_stop,
        ),
    ], simulated_test_runner::SimulatedTestRunner::new(io.sim_test_runner_input, io.sim_test_runner_output, Arc::clone(&config)))
}
