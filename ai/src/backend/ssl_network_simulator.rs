use super::Input;
use crate::communication::node::Node;
use crate::communication::network::UdpMulticastClient;
use crate::motion::tracker::SslSimulatorTrajectoryTracker;
use crate::motion::Trajectory;
use crate::proto;
use crate::proto::config;
use crate::proto::ssl_simulation::{RobotCommand, RobotControl, SimulatorCommand, SimulatorControl};
use multiqueue2;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::Duration;
use crate::proto::config::Config;

pub struct SslNetworkSimulator {
    pub input: Input,
    ssl_simulator_udp_client: UdpMulticastClient,
    trajectory_trackers: HashMap<usize, SslSimulatorTrajectoryTracker>,
}

impl Node for SslNetworkSimulator {
    type Input = Input;
    type Output = ();
    fn run_once(&mut self) -> Result<(), ()> {
        if let Some(trajectories) = self.input.trajectories.take_last()? {
            for (id, t) in trajectories {
                self.trajectory_trackers
                    .get_mut(&id)
                    .unwrap()
                    .update_trajectory(t);
            }
        }

        if let Some(world) = self.input.world.take_last()? {
            for (id, t) in self.trajectory_trackers.iter_mut() {
                if let Some(r) = world.friendly_team.robot(id) {
                    t.update_most_recently_observe_state(r.state.clone());
                }
            }
        }

        let mut sim_control_command: RobotControl = RobotControl::default();
        for (id, t) in self.trajectory_trackers.iter_mut() {
            if let Some(command) = t.run() {
                sim_control_command.robot_commands.push(command);
            }
        }
        self.ssl_simulator_udp_client
            .send_proto(sim_control_command, "0.0.0.0:10301");

        if let Some(command) = self.input.sim_control.take_last()? {
            let mut msg: SimulatorCommand = SimulatorCommand::default();
            msg.control = Some(command);
            self.ssl_simulator_udp_client
                .send_proto(msg, "0.0.0.0:10300");
        }

        sleep(Duration::from_millis(2));
        Ok(())
    }

    fn new(input: Self::Input, output: Self::Output, config: Arc<Mutex<Config>>) -> Self {
        let mut trackers: HashMap<usize, SslSimulatorTrajectoryTracker> = HashMap::new();
        for i in 0..=config.lock().unwrap().rules.max_robot_id {
            trackers.insert(i as usize, SslSimulatorTrajectoryTracker::new(i as usize));
        }

        Self {
            input,
            ssl_simulator_udp_client: UdpMulticastClient::new("0.0.0.0", 10020),
            trajectory_trackers: trackers,
        }
    }

    fn name() -> String {
        "SSL Network Simulator".to_string()
    }
}