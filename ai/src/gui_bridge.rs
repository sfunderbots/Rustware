use crate::communication::node::Node;
use crate::communication::buffer::{NodeReceiver, NodeSender};
use crate::motion::{bb_time_to_position, Trajectory};
use crate::proto;
use crate::proto::config;
use crate::proto::ssl_vision::SslWrapperPackets;
use crate::proto_conversions::{node_performance_to_proto, trajectories_to_proto, world_to_proto};
use crate::world::{Ball, Field, Robot, World};
use prost::Message;
use proto::metrics::NodePerformance;
use std::collections::HashMap;
use std::error::Error;
use std::mem::take;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use crate::proto::ssl_simulation::{SimulatorCommand, SimulatorControl};
use std::time::{Duration, Instant};
use crate::proto::config::Config;

pub struct Input {
    pub ssl_vision: NodeReceiver<proto::ssl_vision::SslWrapperPacket>,
    pub world: NodeReceiver<World>,
    pub trajectories: NodeReceiver<HashMap<usize, Trajectory>>,
    pub metrics: NodeReceiver<(String, f64)>,
}
pub struct Output {
    pub sim_control: NodeSender<SimulatorControl>
}

pub struct GuiBridge {
    input: Input,
    output: Output,
    context: zmq::Context,
    pub_socket: zmq::Socket,
    sub_socket: zmq::Socket,
    config: Arc<Mutex<Config>>,
}

impl GuiBridge {
    fn publish_msg<T>(&self, msg: T, topic: String)
    where
        T: Message,
        T: Default,
    {
        let mut data = topic.as_bytes().to_vec();
        data.append(&mut proto::encode(msg));
        self.pub_socket.send(data, 0).unwrap();
    }

    fn receive_msg<T>(&self, topic: String) -> Result<T, Box<dyn Error>>
        where
            T: Message,
            T: Default,
    {
        let mut data = self.sub_socket.recv_bytes(zmq::DONTWAIT)?;
        let msg_data = data.split_off(topic.len());
        let topic = String::from_utf8(data).unwrap_or("Error getting topic".to_string());
        let msg = T::decode(msg_data.as_slice())?;
        Ok(msg)
    }

}

impl Node for GuiBridge {
    type Input = Input;
    type Output = Output;
    fn run_once(&mut self) -> Result<(), ()> {
        let mut ssl_wrapper_packets:SslWrapperPackets = SslWrapperPackets::default();
        for msg in self.input.ssl_vision.dump()? {
            ssl_wrapper_packets.packets.push(msg);
        }
        if !ssl_wrapper_packets.packets.is_empty() {
            self.publish_msg(
                ssl_wrapper_packets,
                    self.config.lock().unwrap()
                    .gui_bridge
                    .ssl_vision_topic
                    .to_string(),
            );
        }

        if let Some(world) = self.input.world.take_last()? {
            let msg = world_to_proto(&world);
            self.publish_msg(
                msg,
                    self.config.lock().unwrap()
                    .gui_bridge
                    .world_topic
                    .to_string(),
            );
        }

        if let Some(trajectories) = self.input.trajectories.take_last()? {
            let msg = trajectories_to_proto(&trajectories);
            self.publish_msg(
                msg,
                    self.config.lock().unwrap()
                    .gui_bridge
                    .trajectories_topic
                    .to_string(),
            );
        }

        let mut node_performance = HashMap::<String, f64>::new();
        for (topic, pub_period_ms) in self.input.metrics.dump()? {
            if !node_performance.contains_key(&topic) {
                node_performance.insert(topic, pub_period_ms);
            } else {
                *node_performance.get_mut(&topic).unwrap() = pub_period_ms;
            }
        }
        let performance_msg = node_performance_to_proto(node_performance);
        self.publish_msg(
            performance_msg,
                self.config.lock().unwrap()
                .gui_bridge
                .metrics_topic
                .to_string(),
        );

        // TODO: dump everything in queue
        if let Ok(sim_control_command) = self.receive_msg::<SimulatorControl>(
            self.config.lock().unwrap()
                .gui_bridge
                .sim_control_topic
                .to_string()
        ) {
            self.output.sim_control.try_send(sim_control_command);
        }

        // Sending too fast overwhelms the unix sockets
        std::thread::sleep(Duration::from_millis(1));
        Ok(())
    }

    fn new(input: Self::Input, output: Self::Output, config: Arc<Mutex<Config>>) -> Self {
        let context = zmq::Context::new();
        let pub_socket = context.socket(zmq::PUB).unwrap();
        pub_socket
            .bind(config.lock().unwrap().gui_bridge.ai_to_gui_socket.as_str())
            .unwrap();
        let sub_socket = context.socket(zmq::SUB).unwrap();
        sub_socket.set_subscribe(config.lock().unwrap().gui_bridge.sim_control_topic.as_bytes());
        sub_socket.connect(config.lock().unwrap().gui_bridge.gui_to_ai_socket.as_str());
        Self {
            input,
            output,
            context,
            pub_socket,
            sub_socket,
            config,
        }
    }

    fn name() -> String {
        "Gui Bridge".to_string()
    }
}
