use std::sync::{Arc, Mutex};
use crate::proto::ssl_simulation::{SimulatorControl, TeleportRobot, TeleportBall, RobotId, Team};
use crate::communication::{NodeSender, NodeReceiver, Node};
use crate::geom::Point;
use crate::proto::config;
use crate::world::World as PartialWorld;

pub struct Input {
    pub world: NodeReceiver<PartialWorld>,
}

pub struct Output {
    pub sim_control: NodeSender<SimulatorControl>
}

struct SimulatedTestRunner {
    input: Input,
    output: Output,
    config: Arc<Mutex<config::Config>>,
}

impl SimulatedTestRunner {
    pub fn new(input: Input, output: Output, config: Arc<Mutex<config::Config>>) -> Self {
        SimulatedTestRunner{
            input, output, config
        }
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
        }
        self.output.sim_control.try_send(msg);
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
        self.output.sim_control.try_send(msg);
    }
}