mod ball_filter;
mod robot_filter;

use crate::communication::{run_forever, Node};
use crate::constants::{METERS_PER_MILLIMETER, MILLIMETERS_PER_METER};
use crate::geom::{Angle, Point};
use crate::proto;
use crate::world::{Ball, Field, Team, World};
use ball_filter::{BallDetection, BallFilter};
use multiqueue2;
use robot_filter::{RobotDetection, TeamFilter};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

pub struct Input {
    pub ssl_vision_proto: multiqueue2::BroadcastReceiver<proto::ssl_vision::SslWrapperPacket>,
    pub ssl_refbox_proto: multiqueue2::BroadcastReceiver<proto::ssl_gamecontroller::Referee>,
}
pub struct Output {
    pub world: multiqueue2::BroadcastSender<World>,
}

pub struct Perception {
    pub input: Input,
    pub output: Output,
    ball_filter: BallFilter,
    friendly_team_filter: TeamFilter,
    enemy_team_filter: TeamFilter,
    most_recent_world: World,
}

impl Node for Perception {
    fn run_once(&mut self) -> Result<(), ()> {
        let mut ssl_wrapper_packets: Vec<proto::ssl_vision::SslWrapperPacket> = vec![];
        loop {
            match self.input.ssl_vision_proto.try_recv() {
                Ok(msg) => ssl_wrapper_packets.push(msg),
                Err(e) => match e {
                    std::sync::mpsc::TryRecvError::Empty => break,
                    std::sync::mpsc::TryRecvError::Disconnected => {
                        println!("Breaking perception loop");
                        return Err(());
                    }
                },
            };
        }

        if !ssl_wrapper_packets.is_empty() {
            for packet in ssl_wrapper_packets {
                if let Some(detection) = packet.detection {
                    for b in detection.balls {
                        let ball_detection = BallDetection {
                            position: Point { x: b.x, y: b.y },
                            timestamp: detection.t_capture as f32,
                        };
                        self.ball_filter.add_detection(ball_detection);

                        // TODO: Use actual friendly/enemy team colors here
                        for r in &detection.robots_yellow {
                            let detection = RobotDetection {
                                id: r.robot_id.expect("Should always have robot id in proto")
                                    as usize,
                                position: Point { x: r.x, y: r.y },
                                orientation: Angle::from_radians(
                                    r.orientation
                                        .expect("Should always have robot orientation in proto"),
                                ),
                                timestamp: detection.t_capture as f32,
                            };
                            self.friendly_team_filter.add_detection(detection);
                        }

                        for r in &detection.robots_blue {
                            let detection = RobotDetection {
                                id: r.robot_id.expect("Should always have robot id in proto")
                                    as usize,
                                position: Point { x: r.x, y: r.y },
                                orientation: Angle::from_radians(
                                    r.orientation
                                        .expect("Should always have robot orientation in proto"),
                                ),
                                timestamp: detection.t_capture as f32,
                            };
                            self.enemy_team_filter.add_detection(detection);
                        }
                    }
                }

                if let Some(geometry) = packet.geometry {
                    self.most_recent_world.field = Some(field_from_proto(&geometry.field));
                }
            }

            let filtered_ball = self.ball_filter.get_ball();
            let filtered_friendly_team = self.friendly_team_filter.get_team();
            let filtered_enemy_team = self.enemy_team_filter.get_team();

            self.most_recent_world.ball = filtered_ball;
            self.most_recent_world.enemy_team = filtered_enemy_team;
            self.most_recent_world.friendly_team = filtered_friendly_team;
        }
        // println!("Perception got packet {}", packet);
        // self.output.world.try_send();

        Ok(())
    }
}

impl Perception {
    pub fn new(input: Input, output: Output) -> Self {
        Self {
            input: input,
            output: output,
            ball_filter: BallFilter::new(),
            friendly_team_filter: TeamFilter::new(),
            enemy_team_filter: TeamFilter::new(),
            most_recent_world: World {
                ball: None,
                friendly_team: Team::new(),
                enemy_team: Team::new(),
                field: None,
            },
        }
    }
    pub fn create_in_thread(
        input: Input,
        output: Output,
        should_stop: &Arc<AtomicBool>,
    ) -> JoinHandle<()> {
        let should_stop = Arc::clone(should_stop);
        thread::spawn(move || {
            let node = Self::new(input, output);
            run_forever(Box::new(node), should_stop, "Perception");
        })
    }
}

fn field_from_proto(field_pb: &proto::ssl_vision::SslGeometryFieldSize) -> Field {
    let line_length_from_name = |name: &str| -> Option<f32> {
        for line in &field_pb.field_lines {
            if line.name == name {
                return Some(
                    (line.p1.x - line.p2.x).hypot(line.p1.y - line.p2.y) * METERS_PER_MILLIMETER,
                );
            }
        }
        return None;
    };
    let arc_radius_from_name = |name: &str| -> Option<f32> {
        for arc in &field_pb.field_arcs {
            if arc.name == name {
                return Some(arc.radius * METERS_PER_MILLIMETER);
            }
        }
        return None;
    };
    // For some reason the simulators don't publish penalty area width/depth
    // so we have to find the lines using their names. We assume the lines
    // of the same time will be the same length
    // (eg. right and left penalty stretch lines are the same)
    let penalty_area_depth = if let Some(depth) = field_pb.penalty_area_depth {
        depth as f32 * METERS_PER_MILLIMETER
    } else {
        line_length_from_name("LeftFieldLeftPenaltyStretch").unwrap_or_else(|| {
            println!("Unable to find value for penalty area depth in proto");
            // TODO: fallback to reasonable value based on division
            1.5
        })
    };
    let penalty_area_width = if let Some(width) = field_pb.penalty_area_width {
        width as f32 * METERS_PER_MILLIMETER
    } else {
        line_length_from_name("LeftPenaltyStretch").unwrap_or_else(|| {
            println!("Unable to find value for penalty area width in proto");
            // TODO: fallback to reasonable value based on division
            2.5
        })
    };
    let center_circle_radius = if let Some(radius) = field_pb.center_circle_radius {
        radius as f32 * METERS_PER_MILLIMETER
    } else {
        arc_radius_from_name("CenterCircle").unwrap_or_else(|| {
            println!("Unable to find value for center circle radius in proto");
            // TODO: fallback to reasonable value based on division
            0.5
        })
    };

    Field {
        x_length: field_pb.field_length as f32 * METERS_PER_MILLIMETER,
        y_length: field_pb.field_width as f32 * METERS_PER_MILLIMETER,
        defense_x_length: penalty_area_depth,
        defense_y_length: penalty_area_width,
        goal_x_length: field_pb.goal_depth as f32 * METERS_PER_MILLIMETER,
        goal_y_length: field_pb.goal_width as f32 * METERS_PER_MILLIMETER,
        boundary_buffer_size: field_pb.boundary_width as f32 * METERS_PER_MILLIMETER,
        center_circle_radius: center_circle_radius,
    }
}
