use crate::communication::{dump_receiver, run_forever, take_last, Node, NodeReceiver};
use crate::motion::Trajectory;
use crate::proto;
use crate::proto::config;
use crate::proto_conversions::{node_performance_to_proto, trajectories_to_proto, world_to_proto};
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
    socket: zmq::Socket,
    config: Arc<Mutex<config::Config>>,
}

impl GuiBridge {
    pub fn new(input: Input, output: Output, config: Arc<Mutex<config::Config>>) -> Self {
        let context = zmq::Context::new();
        let socket = context.socket(zmq::PUB).unwrap();
        socket
            .bind(config.lock().unwrap().gui_bridge.unix_socket.as_str())
            .unwrap();
        Self {
            input,
            output,
            context,
            socket,
            config,
        }
    }

    fn publish_msg<T>(&self, msg: T, topic: String)
    where
        T: Message,
        T: Default,
    {
        let mut data = topic.as_bytes().to_vec();
        data.append(&mut proto::encode(msg));
        self.socket.send(data, 0).unwrap();
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
        for msg in dump_receiver(&self.input.ssl_vision)? {
            // TODO: faster to batch send?
            self.publish_msg(
                msg,
                self.config
                    .lock()
                    .unwrap()
                    .gui_bridge
                    .ssl_vision_topic
                    .to_string(),
            );
        }

        if let Some(world) = take_last(&self.input.world)? {
            let msg = world_to_proto(&world);
            self.publish_msg(
                msg,
                self.config
                    .lock()
                    .unwrap()
                    .gui_bridge
                    .world_topic
                    .to_string(),
            );
        }

        if let Some(trajectories) = take_last(&self.input.trajectories)? {
            let msg = trajectories_to_proto(&trajectories);
            self.publish_msg(
                msg,
                self.config
                    .lock()
                    .unwrap()
                    .gui_bridge
                    .trajectories_topic
                    .to_string(),
            );
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
        self.publish_msg(
            performance_msg,
            self.config
                .lock()
                .unwrap()
                .gui_bridge
                .metrics_topic
                .to_string(),
        );

        // Sending too fast overwhelms the unix sockets
        std::thread::sleep(Duration::from_millis(5));
        Ok(())
    }
}
