use crate::geom::{Angle, Point};
use crate::motion::KinematicState;
use crate::perception::world::{Ball, Robot, Team};
use float_cmp::ApproxEqUlps;
use std::borrow::BorrowMut;
use std::collections::vec_deque::VecDeque;
use std::collections::HashMap;

pub struct RobotDetection {
    pub id: usize,
    pub position: Point,
    pub orientation: Angle,
    pub timestamp: f64,
}

pub struct RobotFilter {
    detections: VecDeque<RobotDetection>,
}

impl RobotFilter {
    pub fn new() -> RobotFilter {
        RobotFilter {
            detections: VecDeque::new(),
        }
    }

    pub fn add_detection(&mut self, detection: RobotDetection) {
        // Don't add duplicate timestamps to avoid division by 0
        if self
            .detections
            .iter()
            .any(|x| x.timestamp.approx_eq_ulps(&detection.timestamp, 10))
        {
            // println!("skipping detection");
            return;
        }
        self.detections.push_back(detection);
        if self.detections.len() > 2 {
            self.detections.pop_front();
        }
        self.detections
            .make_contiguous()
            .sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());
    }

    pub fn get_robot(&self) -> Option<Robot> {
        if self.detections.len() < 2 {
            return None;
        }

        let position = self.detections[1].position;
        let orientation = self.detections[1].orientation;
        let time_diff = self.detections[1].timestamp - self.detections[0].timestamp;
        let velocity = (self.detections[1].position - self.detections[0].position) / time_diff;
        let angular_velocity =
            (self.detections[1].orientation - self.detections[0].orientation) / time_diff;
        Some(Robot {
            id: self.detections[0].id,
            state: KinematicState {
                position,
                orientation,
                velocity,
                angular_velocity,
            },
        })
    }
}

pub struct TeamFilter {
    robot_filters: HashMap<usize, RobotFilter>,
}

impl TeamFilter {
    pub fn new() -> TeamFilter {
        TeamFilter {
            robot_filters: HashMap::new(),
        }
    }

    pub fn add_detection(&mut self, detection: RobotDetection) {
        if !self.robot_filters.contains_key(&detection.id) {
            self.robot_filters.insert(detection.id, RobotFilter::new());
        }
        self.robot_filters
            .get_mut(&detection.id)
            .unwrap()
            .add_detection(detection);
    }

    pub fn get_team(&self) -> Vec<Robot> {
        let mut robots: Vec<Robot> = vec![];
        for (k, v) in &self.robot_filters {
            if let Some(r) = v.get_robot() {
                robots.push(r);
            }
        }
        robots
    }
}
