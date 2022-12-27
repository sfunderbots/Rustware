use std::os::macos::raw::stat;
use crate::geom::{Angle, Point, Vector};
use crate::motion::{KinematicState, Trajectory};
use crate::proto::ssl_simulation::{RobotCommand, MoveLocalVelocity, RobotMoveCommand};
use crate::proto::ssl_simulation::robot_move_command;
use std::collections::vec_deque::VecDeque;

pub struct SslSimulatorTrajectoryTracker {
    id: usize,
    state: Option<KinematicState>,
    trajectory: Option<Trajectory>,
    tracking_points: VecDeque<Point>,
    current_tracking_point: Option<Point>
}

impl SslSimulatorTrajectoryTracker {
    pub fn new(id: usize) -> SslSimulatorTrajectoryTracker {
        SslSimulatorTrajectoryTracker {
            id,
            state: None,
            trajectory: None,
            tracking_points: VecDeque::new(),
            current_tracking_point: None
        }
    }

    pub fn update_trajectory(&mut self, trajectory: Trajectory) {
        self.trajectory = Some(trajectory);
        // The first state is always the current state so ignore it for tracking, otherwise
        // the robot stutters each time it gets a new trajectory
        // self.tracking_points = VecDeque::from(&self.trajectory.unwrap().points[self.trajectory.unwrap().points.len()-1..].to_owned().to_vec().clone());
        self.tracking_points.clear();
        for x in &self.trajectory.as_ref().unwrap().points[self.trajectory.as_ref().unwrap().points.len()-1..] {
            self.tracking_points.push_back(x.clone());
        }
        if !self.tracking_points.is_empty() {
            self.current_tracking_point = Some(self.tracking_points.pop_front().unwrap());
        }

    }

    pub fn update_most_recently_observe_state(&mut self, state: KinematicState) {
        self.state = Some(state);
    }

    pub fn run(&mut self) -> Option<RobotCommand> {
        if self.state.is_none() || self.trajectory.is_none() || self.current_tracking_point.is_none() {
            return None
        }

        if (self.state.as_ref().unwrap().position - self.current_tracking_point.as_ref().unwrap()).length() < 0.1 && !self.tracking_points.is_empty() {
            self.current_tracking_point = Some(self.tracking_points.pop_front().unwrap());
        }

        let state = self.state.as_ref().unwrap();
        let current_tracking_point = self.current_tracking_point.as_ref().unwrap();

        let target_position = current_tracking_point;
        let position_error: Vector = target_position - &state.position;
        let desired_velocity = if position_error.length() < 1.0e-3 {Vector::new()} else {position_error.norm((position_error.length() * 2.5).min(3.0))};
        let desired_velocity = desired_velocity.rotate(&-state.orientation);

        let target_orientation = self.trajectory.as_ref().unwrap().final_orientation;
        let orientation_error = state.orientation - target_orientation;
        let desired_angular_velocity = (orientation_error / 4).min(Angle::full() * 4);

        let mut robot_command: RobotCommand = RobotCommand::default();
        robot_command.id = self.id as u32;
        let mut move_local_velocity: MoveLocalVelocity = MoveLocalVelocity::default();
        move_local_velocity.forward = desired_velocity.x as f32;
        move_local_velocity.left = desired_velocity.y as f32;
        move_local_velocity.angular = desired_angular_velocity.radians() as f32;
        // move_local_velocity.angular = 1.0;
        let robot_move_command = robot_move_command::Command::LocalVelocity(move_local_velocity);
        let mut move_command = RobotMoveCommand::default();
        move_command.command = Some(robot_move_command);
        robot_command.move_command = Some(move_command);

        Some(robot_command)
    }
}
