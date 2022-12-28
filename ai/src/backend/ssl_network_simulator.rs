use super::Input;
use crate::communication;
use crate::communication::{run_forever, take_last, Node, UdpMulticastClient};
use crate::motion::tracker::SslSimulatorTrajectoryTracker;
use crate::motion::Trajectory;
use crate::proto;
use crate::proto::config;
use crate::proto::ssl_simulation::{RobotCommand, RobotControl};
use multiqueue2;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::Duration;

pub struct SslNetworkSimulator {
    pub input: Input,
    ssl_simulator_udp_client: UdpMulticastClient,
    trajectory_trackers: HashMap<usize, SslSimulatorTrajectoryTracker>,
}

impl Node for SslNetworkSimulator {
    fn run_once(&mut self) -> Result<(), ()> {
        if let Some(trajectories) = take_last(&self.input.trajectories)? {
            for (id, t) in trajectories {
                self.trajectory_trackers
                    .get_mut(&id)
                    .unwrap()
                    .update_trajectory(t);
            }
        }

        if let Some(world) = take_last(&self.input.world)? {
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

        sleep(Duration::from_millis(2));
        Ok(())
    }
}

impl SslNetworkSimulator {
    pub fn new(input: Input, config: Arc<Mutex<config::Config>>) -> Self {
        let mut trackers: HashMap<usize, SslSimulatorTrajectoryTracker> = HashMap::new();
        for i in 0..=config.lock().unwrap().rules.max_robot_id {
            trackers.insert(i as usize, SslSimulatorTrajectoryTracker::new(i as usize));
        }

        Self {
            input,
            ssl_simulator_udp_client: communication::UdpMulticastClient::new("0.0.0.0", 10020),
            trajectory_trackers: trackers,
        }
    }

    pub fn create_in_thread(
        input: Input,
        config: &Arc<Mutex<config::Config>>,
        should_stop: &Arc<AtomicBool>,
    ) -> JoinHandle<()> {
        let should_stop = Arc::clone(should_stop);
        let local_config = Arc::clone(config);
        thread::spawn(move || {
            let node = Self::new(input, local_config);
            run_forever(Box::new(node), should_stop, "SslNetworkSimulator");
        })
    }
}
