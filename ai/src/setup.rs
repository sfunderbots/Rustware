use crate::perception;
use crate::gameplay;
use crate::backend;
use crate::gui_bridge;
use crate::proto;
use crate::communication::buffer::{node_connection, NodeReceiver, NodeSender};
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
use crate::communication::node::{Node, ThreadedRunner, SynchronousRunner};

pub struct SynchronousNodes {
    pub perception: SynchronousRunner<perception::Perception>,
    pub gameplay: SynchronousRunner<gameplay::Gameplay>,
    pub ssl_listener: SynchronousRunner<backend::SslNetworkListener>,
    pub ssl_simulator: SynchronousRunner<backend::SslNetworkSimulator>,
    pub gui_bridge: SynchronousRunner<gui_bridge::GuiBridge>,
}

pub struct ThreadedNodes {
    pub perception: ThreadedRunner<perception::Perception>,
    pub gameplay: ThreadedRunner<gameplay::Gameplay>,
    pub ssl_listener: ThreadedRunner<backend::SslNetworkListener>,
    pub ssl_simulator: ThreadedRunner<backend::SslNetworkSimulator>,
    pub gui_bridge: ThreadedRunner<gui_bridge::GuiBridge>,
}

impl ThreadedNodes {
    pub fn join(self) {
        self.perception.join();
        self.gameplay.join();
        self.ssl_listener.join();
        self.ssl_simulator.join();
        self.gui_bridge.join();
    }
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
        perception: SynchronousRunner::<perception::Perception>::new(
            io.perception_input,
            io.perception_output,
            &config,
        ),
        gameplay: SynchronousRunner::<gameplay::Gameplay>::new(io.gameplay_input, io.gameplay_output, &config),
        ssl_listener: SynchronousRunner::<backend::SslNetworkListener>::new((), io.backend_output, &config),
        ssl_simulator: SynchronousRunner::<backend::SslNetworkSimulator>::new(io.backend_input, (), &config),
        gui_bridge: SynchronousRunner::<gui_bridge::GuiBridge>::new(
            io.gui_bridge_input,
            io.gui_bridge_output,
            &config,
        ),
    }
}

pub fn create_threaded_nodes(io: AllNodeIo, should_stop: &Arc<AtomicBool>) -> ThreadedNodes {
    let config = Arc::new(Mutex::new(load_config().unwrap()));
    ThreadedNodes{
        perception: ThreadedRunner::<perception::Perception>::new(
            io.perception_input,
            io.perception_output,
            &config,
            &should_stop,
        ),
        gameplay: ThreadedRunner::<gameplay::Gameplay>::new(io.gameplay_input, io.gameplay_output, &config, &should_stop),
        ssl_listener: ThreadedRunner::<backend::SslNetworkListener>::new((), io.backend_output, &config, &should_stop),
        ssl_simulator: ThreadedRunner::<backend::SslNetworkSimulator>::new(io.backend_input, (), &config, &should_stop),
        gui_bridge: ThreadedRunner::<gui_bridge::GuiBridge>::new(
        io.gui_bridge_input,
        io.gui_bridge_output,
        &config,
        &should_stop,
        ),
    }
}