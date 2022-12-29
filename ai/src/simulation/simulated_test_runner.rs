use crate::geom::Point;
use crate::proto::config;
use crate::proto::ssl_simulation::{RobotId, SimulatorControl, Team, TeleportBall, TeleportRobot};
use crate::setup::{create_threaded_nodes, set_up_node_io, ThreadedNodes};
use crate::world::World as PartialWorld;
use std::sync::{Arc, Mutex};

pub struct SimulatedTestRunner {
    nodes: ThreadedNodes,
}
