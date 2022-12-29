use crate::communication_old::{NodeReceiver, NodeSender};
use crate::motion::Trajectory;
use crate::proto;
use std::collections::HashMap;

mod ssl_network_listener;
mod ssl_network_simulator;
mod ssl_synchronous_simulator;

use crate::world::World;
pub use ssl_network_listener::SslNetworkListener;
pub use ssl_network_simulator::SslNetworkSimulator;
pub use ssl_synchronous_simulator::SslSynchronousSimulator;
use crate::proto::ssl_simulation::SimulatorControl;

pub struct Output {
    pub ssl_vision: NodeSender<proto::ssl_vision::SslWrapperPacket>,
    pub ssl_gc: NodeSender<proto::ssl_gamecontroller::Referee>,
}

pub struct Input {
    pub world: NodeReceiver<World>,
    pub trajectories: NodeReceiver<HashMap<usize, Trajectory>>,
    pub sim_control: NodeReceiver<SimulatorControl>,
}
