use crate::communication::{dump_receiver, run_forever, take_last, Node, NodeReceiver};
use crate::motion::Trajectory;
use crate::proto;
use crate::proto::config;
use crate::proto::visualization::Visualization;
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

pub struct Input {
    pub ssl_vision: NodeReceiver<proto::ssl_vision::SslWrapperPacket>,
    pub world: NodeReceiver<World>,
    pub trajectories: NodeReceiver<HashMap<usize, Trajectory>>,
    pub metrics: NodeReceiver<(String, f64)>,
}
pub struct Output {}

pub struct GuiBridge {
    input: Input,
    output: Output,
    context: zmq::Context,
    ssl_vision_socket: zmq::Socket,
    ssl_gc_socket: zmq::Socket,
    world_socket: zmq::Socket,
    metrics_socket: zmq::Socket,
    config: Arc<Mutex<config::Config>>,
}

fn create_endpoint(socket_prefix: String, topic: String) -> String {
    socket_prefix + topic.as_str()
}
fn create_endpoint2(socket_prefix: &str, topic: &str) -> String {
    socket_prefix.to_owned() + topic
}

impl GuiBridge {
    pub fn new(input: Input, output: Output, config: Arc<Mutex<config::Config>>) -> Self {
        let context = zmq::Context::new();
        let ssl_vision_socket = context.socket(zmq::PUB).unwrap();
        // BEWARE: The config mutex is only unlocked when the retrieved value goes out of scope.
        // For function calls, this isn't until after the function, so if multiple function
        // parameters try access the config mutex, this will cause a deadlock
        let socket_prefix = config
            .lock()
            .unwrap()
            .gui_bridge
            .unix_socket_prefix
            .to_string();
        ssl_vision_socket
            .bind(
                create_endpoint(
                    socket_prefix.clone(),
                    config
                        .lock()
                        .unwrap()
                        .gui_bridge
                        .ssl_vision_topic
                        .to_string()
                        .clone(),
                )
                .as_str(),
            )
            .unwrap();
        let ssl_gc_socket = context.socket(zmq::PUB).unwrap();
        ssl_gc_socket
            .bind(
                create_endpoint(
                    socket_prefix.clone(),
                    config.lock().unwrap().gui_bridge.ssl_gc_topic.to_string(),
                )
                .as_str(),
            )
            .unwrap();
        let world_socket = context.socket(zmq::PUB).unwrap();
        world_socket
            .bind(
                create_endpoint(
                    socket_prefix.clone(),
                    config.lock().unwrap().gui_bridge.world_topic.to_string(),
                )
                .as_str(),
            )
            .unwrap();
        let metrics_socket = context.socket(zmq::PUB).unwrap();
        metrics_socket
            .bind(
                create_endpoint(
                    socket_prefix.clone(),
                    config.lock().unwrap().gui_bridge.metrics_topic.to_string(),
                )
                .as_str(),
            )
            .unwrap();
        Self {
            input,
            output,
            context,
            ssl_vision_socket,
            ssl_gc_socket,
            world_socket,
            metrics_socket,
            config,
        }
    }

    pub fn create_in_thread(
        input: Input,
        output: Output,
        config: &Arc<Mutex<config::Config>>,
        should_stop: &Arc<AtomicBool>,
    ) -> JoinHandle<()> {
        let should_stop = Arc::clone(should_stop);
        let local_config = Arc::clone(config);
        thread::spawn(move || {
            let node = Self::new(input, output, local_config);
            run_forever(Box::new(node), should_stop, "GuiBridge");
        })
    }
}

impl Node for GuiBridge {
    fn run_once(&mut self) -> Result<(), ()> {
        // println!("sending ssl vision on bridge");
        for msg in dump_receiver(&self.input.ssl_vision)? {
            // TODO: faster to batch send?
            self.ssl_vision_socket.send(proto::encode(msg), 0).unwrap();
        }

        if let Some(world) = take_last(&self.input.world)? {
            let foo = world_to_proto(&world);
            let mut msg = Visualization::default();
            msg.world = Some(foo);
            self.world_socket.send(proto::encode(msg), 0).unwrap();
        }

        if let Some(trajectories) = take_last(&self.input.trajectories)? {
            let mut msg = Visualization::default();
            trajectories_to_proto(&trajectories, &mut msg);
            // TODO: since the world and trajectories are both in the same message, could probably
            // publish them together
            self.world_socket.send(proto::encode(msg), 0).unwrap();
        }

        let mut node_performance = HashMap::<String, f64>::new();
        for (topic, pub_period_ms) in dump_receiver(&self.input.metrics)? {
            if !node_performance.contains_key(&topic) {
                node_performance.insert(topic, pub_period_ms);
            } else {
                *node_performance.get_mut(&topic).unwrap() = pub_period_ms;
            }
        }
        let performance_msg = node_performance_to_proto(node_performance);
        self.metrics_socket
            .send(proto::encode(performance_msg), 0)
            .unwrap();

        // Sending too fast overwhelms the unix sockets
        std::thread::sleep(Duration::from_millis(5));
        Ok(())
    }
}

fn world_to_proto(world: &World) -> proto::visualization::PerceptionWorld {
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
    let robot_to_proto = |r: &Robot| -> proto::visualization::perception_world::Robot {
        proto::visualization::perception_world::Robot {
            id: r.id as u32,
            x: r.state.position.x,
            y: r.state.position.y,
            vx: r.state.velocity.x,
            vy: r.state.velocity.y,
            orientation: r.state.orientation.radians(),
        }
    };
    // TODO: use actual team colors. Perhaps the GUI should ignore
    // blue/yellow for the filtered vision
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

pub fn trajectories_to_proto(trajectories: &HashMap<usize, Trajectory>, mut msg: &mut Visualization) {
    msg.trajectories.clear();
    for (_, t) in trajectories {
        let mut t_proto = proto::visualization::Trajectory::default();
        for p in &t.points {
            t_proto.points.push(proto::visualization::Vector2{x: p.x, y: p.y});
        }
        msg.trajectories.push(t_proto);
    }
}
