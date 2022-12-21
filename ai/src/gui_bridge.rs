use crate::communication::{dump_receiver, run_forever, take_last, Node, NodeReceiver};
use crate::motion::Trajectory;
use crate::perception;
use crate::proto;
use crate::proto::visualization::Visualization;
use prost::Message;
use std::mem::take;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub struct Input {
    pub ssl_vision_proto: NodeReceiver<proto::ssl_vision::SslWrapperPacket>,
    pub perception_world: NodeReceiver<perception::World>,
}
pub struct Output {}

pub struct GuiBridge {
    input: Input,
    output: Output,
    context: zmq::Context,
    ssl_vision_socket: zmq::Socket,
    ssl_gc_socket: zmq::Socket,
    world_socket: zmq::Socket,
}

fn create_endpoint(socket_prefix: String, topic: String) -> String {
    socket_prefix + topic.as_str()
}

impl GuiBridge {
    pub fn new(input: Input, output: Output) -> Self {
        let ctx = zmq::Context::new();
        let ssl_vision_socket = ctx.socket(zmq::PUB).unwrap();
        ssl_vision_socket
            .bind(
                create_endpoint(
                    "ipc:///tmp/underbots_zmq_".to_string(),
                    "ssl_vision".to_string(),
                )
                .as_str(),
            )
            .unwrap();
        let ssl_gc_socket = ctx.socket(zmq::PUB).unwrap();
        ssl_gc_socket
            .bind(
                create_endpoint(
                    "ipc:///tmp/underbots_zmq_".to_string(),
                    "ssl_gc".to_string(),
                )
                .as_str(),
            )
            .unwrap();
        let world_socket = ctx.socket(zmq::PUB).unwrap();
        world_socket
            .bind(
                create_endpoint("ipc:///tmp/underbots_zmq_".to_string(), "world".to_string())
                    .as_str(),
            )
            .unwrap();
        Self {
            input,
            output,
            context: ctx,
            ssl_vision_socket,
            ssl_gc_socket,
            world_socket,
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
            run_forever(Box::new(node), should_stop, "GuiBridge");
        })
    }
}

impl Node for GuiBridge {
    fn run_once(&mut self) -> Result<(), ()> {
        if let Some(ssl_vision_msg) = take_last(&self.input.ssl_vision_proto)? {
            self.ssl_vision_socket
                .send(proto::encode(ssl_vision_msg), 0)
                .unwrap();
        }

        if let Some(world) = take_last(&self.input.perception_world)? {
            // println!("sending world");
            let foo = world_to_proto(&world);
            let vis_msg = Visualization { world: Some(foo) };
            self.world_socket.send(proto::encode(vis_msg), 0).unwrap();
        }

        Ok(())
    }
}

fn world_to_proto(world: &perception::World) -> proto::visualization::PerceptionWorld {
    let mut msg: proto::visualization::PerceptionWorld =
        proto::visualization::PerceptionWorld::default();
    if let Some(ball) = &world.ball {
        let mut ball_proto: proto::visualization::perception_world::Ball =
            proto::visualization::perception_world::Ball::default();
        ball_proto.x = ball.position.x;
        ball_proto.y = ball.position.y;
        ball_proto.vx = ball.velocity.x;
        ball_proto.vy = ball.velocity.y;
        msg.ball = Some(ball_proto);
    }
    if let Some(field) = &world.field {
        let mut field_proto: proto::visualization::perception_world::Field =
            proto::visualization::perception_world::Field::default();
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
    let robot_to_proto = |r: &perception::Robot| -> proto::visualization::perception_world::Robot {
        proto::visualization::perception_world::Robot {
            id: r.id as u32,
            x: r.state.position.x,
            y: r.state.position.y,
            vx: r.state.velocity.x,
            vy: r.state.velocity.y,
            orientation: r.state.orientation.radians(),
        }
    };
    for r in &world.blue_team {
        msg.blue_robots.push(robot_to_proto(r));
    }
    for r in &world.yellow_team {
        msg.yellow_robots.push(robot_to_proto(r));
    }

    msg
}
