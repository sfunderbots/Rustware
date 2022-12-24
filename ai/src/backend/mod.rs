use crate::communication::{NodeReceiver, NodeSender};
use crate::motion::Trajectory;
use crate::proto;
use std::collections::HashMap;

mod ssl_network_listener;
mod ssl_network_simulator;
mod ssl_synchronous_simulator;

pub use ssl_network_listener::SslNetworkListener;
pub use ssl_network_simulator::SslNetworkSimulator;
pub use ssl_synchronous_simulator::SslSynchronousSimulator;

pub struct Output {
    pub ssl_vision: NodeSender<proto::ssl_vision::SslWrapperPacket>,
    pub ssl_gc: NodeSender<proto::ssl_gamecontroller::Referee>,
}

pub struct Input {
    pub trajectories: NodeReceiver<HashMap<usize, Trajectory>>,
}
