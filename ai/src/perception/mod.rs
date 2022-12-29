mod ball_filter;
mod robot_filter;

use crate::communication::buffer::{NodeReceiver, NodeSender};
use crate::communication::node::Node;
use crate::constants::{METERS_PER_MILLIMETER, MILLIMETERS_PER_METER};
use crate::geom::{Angle, Point};
use crate::proto;
use crate::proto::config;
use crate::proto::config::Config;
use crate::proto::ssl_gamecontroller;
use crate::proto::ssl_gamecontroller::referee;
use crate::proto::ssl_gamecontroller::referee::Command;
use crate::proto::ssl_vision::SslDetectionRobot;
use crate::world::{Ball, Field, GameState, Robot, Team, TeamInfo, World};
use ball_filter::{BallDetection, BallFilter};
use multiqueue2;
use robot_filter::{RobotDetection, TeamFilter};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

pub struct Input {
    pub ssl_vision: NodeReceiver<proto::ssl_vision::SslWrapperPacket>,
    pub ssl_gc: NodeReceiver<proto::ssl_gamecontroller::Referee>,
}
pub struct Output {
    pub world: NodeSender<World>,
}

pub struct Perception {
    input: Input,
    output: Output,
    ball_filter: BallFilter,
    friendly_team_filter: TeamFilter,
    enemy_team_filter: TeamFilter,
    world: World,
    config: Arc<Mutex<Config>>,
}

impl Node for Perception {
    type Input = Input;
    type Output = Output;
    fn run_once(&mut self) -> Result<(), ()> {
        self.world.friendly_team_info =
            TeamInfo::from_referee(None, &self.config.lock().unwrap().perception, true);
        self.world.enemy_team_info =
            TeamInfo::from_referee(None, &self.config.lock().unwrap().perception, false);
        // TODO: This is dumb, innefficient code that published GameState every single tick,
        // which is unnecessarily fast. Ideally we should only publish when we get a packet, or
        // every N seconds otherwise
        let ssl_referee_packets = self.input.ssl_gc.dump()?;
        if !ssl_referee_packets.is_empty() {
            for packet in &ssl_referee_packets {
                self.world.friendly_team_info = TeamInfo::from_referee(
                    Some(packet),
                    &self.config.lock().unwrap().perception,
                    true,
                );
                self.world.enemy_team_info = TeamInfo::from_referee(
                    Some(packet),
                    &self.config.lock().unwrap().perception,
                    false,
                );
                if let Some(info) = &self.world.friendly_team_info {
                    self.world.game_state.update_command(
                        referee::Command::from_i32(packet.command).unwrap(),
                        info.is_blue,
                    )
                }
            }
        }

        let ssl_wrapper_packets = self.input.ssl_vision.dump()?;
        if !ssl_wrapper_packets.is_empty() {
            for packet in ssl_wrapper_packets {
                if let Some(detection) = packet.detection {
                    for b in detection.balls {
                        let ball_detection = BallDetection {
                            position: Point {
                                x: b.x as f64 * METERS_PER_MILLIMETER,
                                y: b.y as f64 * METERS_PER_MILLIMETER,
                            },
                            timestamp: detection.t_capture,
                        };
                        self.ball_filter.add_detection(ball_detection);
                    }

                    let create_robot_detection =
                        |ssl_robot: &SslDetectionRobot, t_capture: f64| -> RobotDetection {
                            RobotDetection {
                                id: ssl_robot
                                    .robot_id
                                    .expect("Should always have robot id in proto")
                                    as usize,
                                position: Point {
                                    x: ssl_robot.x as f64 * METERS_PER_MILLIMETER,
                                    y: ssl_robot.y as f64 * METERS_PER_MILLIMETER,
                                },
                                orientation: Angle::from_radians(
                                    ssl_robot
                                        .orientation
                                        .expect("Should always have robot orientation in proto")
                                        as f64,
                                ),
                                timestamp: t_capture,
                            }
                        };

                    if let Some(info) = &self.world.friendly_team_info {
                        let friendly_robots = if info.is_blue {
                            &detection.robots_blue
                        } else {
                            &detection.robots_yellow
                        };
                        let enemy_robots = if info.is_blue {
                            &detection.robots_yellow
                        } else {
                            &detection.robots_blue
                        };

                        for r in friendly_robots {
                            self.friendly_team_filter
                                .add_detection(create_robot_detection(r, detection.t_capture));
                        }
                        for r in enemy_robots {
                            self.enemy_team_filter
                                .add_detection(create_robot_detection(r, detection.t_capture));
                        }
                    }
                }

                if let Some(geometry) = packet.geometry {
                    self.world.field = Some(field_from_proto(&geometry.field));
                }
            }

            self.world.ball = self.ball_filter.get_ball();
            self.world.friendly_team = self.friendly_team_filter.get_team();
            if let Some(info) = &self.world.friendly_team_info {
                self.world.friendly_team.set_goalie(Some(info.goalie_id));
            }
            self.world.enemy_team = self.enemy_team_filter.get_team();
            if let Some(info) = &self.world.enemy_team_info {
                self.world.enemy_team.set_goalie(Some(info.goalie_id));
            }
            self.output.world.try_send(self.world.clone());
        }

        Ok(())
    }

    fn new(input: Self::Input, output: Self::Output, config: Arc<Mutex<Config>>) -> Self {
        Self {
            input,
            output,
            ball_filter: BallFilter::new(),
            friendly_team_filter: TeamFilter::new(),
            enemy_team_filter: TeamFilter::new(),
            world: World {
                ball: None,
                friendly_team: Team::new(),
                enemy_team: Team::new(),
                field: None,
                game_state: GameState::new(),
                friendly_team_info: None,
                enemy_team_info: None,
            },
            config,
        }
    }

    fn name() -> String {
        "Perception".to_string()
    }
}

fn field_from_proto(field_pb: &proto::ssl_vision::SslGeometryFieldSize) -> Field {
    let line_length_from_name = |name: &str| -> Option<f64> {
        for line in &field_pb.field_lines {
            if line.name == name {
                return Some(
                    (line.p1.x - line.p2.x).hypot(line.p1.y - line.p2.y) as f64
                        * METERS_PER_MILLIMETER,
                );
            }
        }
        return None;
    };
    let arc_radius_from_name = |name: &str| -> Option<f64> {
        for arc in &field_pb.field_arcs {
            if arc.name == name {
                return Some(arc.radius as f64 * METERS_PER_MILLIMETER);
            }
        }
        return None;
    };
    // For some reason the simulators don't publish penalty area width/depth
    // so we have to find the lines using their names. We assume the lines
    // of the same time will be the same length
    // (eg. right and left penalty stretch lines are the same)
    let penalty_area_depth = if let Some(depth) = field_pb.penalty_area_depth {
        depth as f64 * METERS_PER_MILLIMETER
    } else {
        line_length_from_name("LeftFieldLeftPenaltyStretch").unwrap_or_else(|| {
            println!("Unable to find value for penalty area depth in proto");
            // TODO: fallback to reasonable value based on division
            1.5
        })
    };
    let penalty_area_width = if let Some(width) = field_pb.penalty_area_width {
        width as f64 * METERS_PER_MILLIMETER
    } else {
        line_length_from_name("LeftPenaltyStretch").unwrap_or_else(|| {
            println!("Unable to find value for penalty area width in proto");
            // TODO: fallback to reasonable value based on division
            2.5
        })
    };
    let center_circle_radius = if let Some(radius) = field_pb.center_circle_radius {
        radius as f64 * METERS_PER_MILLIMETER
    } else {
        arc_radius_from_name("CenterCircle").unwrap_or_else(|| {
            println!("Unable to find value for center circle radius in proto");
            // TODO: fallback to reasonable value based on division
            0.5
        })
    };

    Field {
        x_length: field_pb.field_length as f64 * METERS_PER_MILLIMETER,
        y_length: field_pb.field_width as f64 * METERS_PER_MILLIMETER,
        defense_x_length: penalty_area_depth,
        defense_y_length: penalty_area_width,
        goal_x_length: field_pb.goal_depth as f64 * METERS_PER_MILLIMETER,
        goal_y_length: field_pb.goal_width as f64 * METERS_PER_MILLIMETER,
        boundary_size: field_pb.boundary_width as f64 * METERS_PER_MILLIMETER,
        center_circle_radius: center_circle_radius,
    }
}
