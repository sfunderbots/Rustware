use crate::motion::Trajectory;
use crate::proto;
use crate::proto::config;
use crate::world::{Ball, Field, Robot, World};
use prost::Message;
use proto::metrics::NodePerformance;
use std::collections::HashMap;
use std::mem::take;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub fn world_to_proto(world: &World) -> proto::world::World {
    let mut msg: proto::world::World = proto::world::World::default();
    if let Some(ball) = &world.ball {
        let mut ball_proto: proto::world::Ball = proto::world::Ball::default();
        ball_proto.x = ball.position.x;
        ball_proto.y = ball.position.y;
        ball_proto.vx = ball.velocity.x;
        ball_proto.vy = ball.velocity.y;
        msg.ball = Some(ball_proto);
    }
    if let Some(field) = &world.field {
        let mut field_proto: proto::world::Field = proto::world::Field::default();
        field_proto.x_length = field.x_length;
        field_proto.y_length = field.y_length;
        field_proto.defense_x_length = field.defense_x_length;
        field_proto.defense_y_length = field.defense_y_length;
        field_proto.goal_x_length = field.goal_x_length;
        field_proto.goal_y_length = field.goal_y_length;
        field_proto.boundary_size = field.boundary_size;
        field_proto.center_circle_radius = field.center_circle_radius;
        msg.field = Some(field_proto);
    }
    let robot_to_proto = |r: &Robot| -> proto::world::Robot {
        proto::world::Robot {
            id: r.id as u32,
            x: r.state.position.x,
            y: r.state.position.y,
            vx: r.state.velocity.x,
            vy: r.state.velocity.y,
            orientation: r.state.orientation.radians(),
        }
    };
    for r in world.friendly_team.all_robots() {
        msg.friendly_robots.push(robot_to_proto(r));
    }
    for r in world.enemy_team.all_robots() {
        msg.enemy_robots.push(robot_to_proto(r));
    }

    msg
}

pub fn node_performance_to_proto(p: HashMap<String, f64>) -> NodePerformance {
    let mut msg: NodePerformance = NodePerformance::default();
    for (k, v) in p {
        msg.mean_publish_period_ms.insert(k, v);
    }
    msg
}

pub fn trajectories_to_proto(
    trajectories: &HashMap<usize, Trajectory>,
) -> proto::trajectory::Trajectories {
    let mut msg: proto::trajectory::Trajectories = proto::trajectory::Trajectories::default();
    for (_, t) in trajectories {
        let mut t_proto = proto::trajectory::Trajectory::default();
        for p in &t.points {
            t_proto
                .points
                .push(proto::trajectory::Vector2 { x: p.x, y: p.y });
        }
        msg.trajectories.push(t_proto);
    }
    msg
}
