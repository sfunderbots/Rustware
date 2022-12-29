use super::Input;
use crate::communication::node::Node;
use crate::communication::network::UdpMulticastClient;
use crate::motion::tracker::SslSimulatorTrajectoryTracker;
use crate::proto::ssl_simulation::{TeleportRobot, TeleportBall, RobotId, Team};
use crate::geom::Point;
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
    config: Arc<Mutex<Config>>
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
            self.send_sim_control_command(command);
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
            config
        }
    }

    fn name() -> String {
        "SSL Network Simulator".to_string()
    }
}

impl SslNetworkSimulator {
    pub fn send_sim_control_command(&mut self, command: SimulatorControl) {
        let mut msg: SimulatorCommand = SimulatorCommand::default();
        msg.control = Some(command);
        self.ssl_simulator_udp_client
            .send_proto(msg, "0.0.0.0:10300");
    }

    pub fn remove_all_robots(&mut self) {
        let mut msg: SimulatorControl = SimulatorControl::default();
        for id in 0..=self.config.lock().unwrap().rules.max_robot_id {
            let mut tr_yellow: TeleportRobot = TeleportRobot::default();
            let mut robot_id: RobotId = RobotId{
                id: Some(id),
                team: Some(Team::Yellow as i32)
            };
            tr_yellow.id = robot_id;
            tr_yellow.present = Some(false);
            msg.teleport_robot.push(tr_yellow);
            let mut tr_blue: TeleportRobot = TeleportRobot::default();
            let mut robot_id: RobotId = RobotId{
                id: Some(id),
                team: Some(Team::Blue as i32)
            };
            tr_blue.id = robot_id;
            tr_blue.present = Some(false);
            msg.teleport_robot.push(tr_blue);
        }
        self.send_sim_control_command(msg);
    }

    pub fn set_robot(&mut self, id: usize, position: Point, blue: bool) {
        let mut msg:SimulatorControl = SimulatorControl::default();
        let mut teleport_robot: TeleportRobot = TeleportRobot::default();
        let mut robot_id: RobotId = RobotId{
            id: Some(id as u32),
            team: Some(if blue {Team::Blue} else {Team::Yellow} as i32)
        };
        teleport_robot.id = robot_id;
        teleport_robot.present = Some(true);
        teleport_robot.x = Some(position.x as f32);
        teleport_robot.y = Some(position.y as f32);
        teleport_robot.orientation = Some(0.0);
        teleport_robot.v_x = Some(0.0);
        teleport_robot.v_y = Some(0.0);
        teleport_robot.v_angular = Some(0.0);
        msg.teleport_robot.push(teleport_robot);
        self.send_sim_control_command(msg);
    }
}