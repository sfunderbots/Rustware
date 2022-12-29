use std::sync::{Arc, Mutex};
use crate::proto::ssl_simulation::{SimulatorControl, TeleportRobot, TeleportBall, RobotId, Team};
use crate::geom::Point;
use crate::proto::config;
use crate::world::World as PartialWorld;
use crate::setup::{ThreadedNodes, create_threaded_nodes, set_up_node_io};

pub struct SimulatedTestRunner {
    nodes: ThreadedNodes,
}
