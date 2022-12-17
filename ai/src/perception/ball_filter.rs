use crate::geom::Point;
use crate::perception::world::Ball;
use std::collections::vec_deque::VecDeque;
use float_cmp::ApproxEqUlps;

pub struct BallDetection {
    pub position: Point,
    pub timestamp: f32
}

pub struct BallFilter {
    detections: VecDeque<BallDetection>
}

impl BallFilter {
    pub fn new() -> BallFilter {
        BallFilter{
            detections: VecDeque::new()
        }
    }

    pub fn add_detection(&mut self, detection: BallDetection) {
        // Don't add duplicate timestamps to avoid division by 0
        if self.detections.iter().any(|x| {x.timestamp.approx_eq_ulps(&detection.timestamp, 10)}) {
            return;
        }
        self.detections.push_back(detection);
        if self.detections.len() > 2 {
            self.detections.pop_front();
        }

    }

    pub fn get_ball(&mut self) -> Option<Ball> {
        if self.detections.len() < 2 {
            return None;
        }

        // let sorted: Vec<BallDetection> = self.detections.make_contiguous().iter().sorted_by(|a, b| {a.timestamp.partial_cmp(&b.timestamp).unwrap()}).collect();
        self.detections.make_contiguous().sort_by(|a, b| {a.timestamp.partial_cmp(&b.timestamp).unwrap()});
        let position = self.detections[1].position;
        let time_diff = self.detections[1].timestamp - self.detections[0].timestamp;
        let velocity = (self.detections[1].position - self.detections[0].position) / time_diff;
        Some(Ball{
            position,
            velocity
        })
    }
}