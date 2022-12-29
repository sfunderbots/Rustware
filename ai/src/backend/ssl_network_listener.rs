use super::Output;
use crate::communication::network::UdpMulticastClient;
use crate::communication::node::Node;
use crate::motion::Trajectory;
use crate::proto;
use crate::proto::config::Config;
use multiqueue2;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::Duration;

pub struct SslNetworkListener {
    pub output: Output,
    ssl_vision_udp_client: UdpMulticastClient,
    ssl_gamecontroller_udp_client: UdpMulticastClient,
}

impl Node for SslNetworkListener {
    type Input = ();
    type Output = Output;
    fn run_once(&mut self) -> Result<(), ()> {
        loop {
            match self
                .ssl_vision_udp_client
                .read_proto::<proto::ssl_vision::SslWrapperPacket>()
            {
                Ok(msg) => {
                    self.output.ssl_vision.try_send(msg);
                }
                Err(_) => break,
            }

            match self
                .ssl_gamecontroller_udp_client
                .read_proto::<proto::ssl_gamecontroller::Referee>()
            {
                Ok(msg) => {
                    self.output.ssl_gc.try_send(msg);
                }
                Err(_) => break,
            }
        }
        Ok(())
    }

    fn new(input: Self::Input, output: Self::Output, config: Arc<Mutex<Config>>) -> Self {
        Self {
            output: output,
            ssl_vision_udp_client: UdpMulticastClient::new("224.5.23.2", 10020),
            ssl_gamecontroller_udp_client: UdpMulticastClient::new("224.5.23.1", 10003),
        }
    }

    fn name() -> String {
        "SSL Network Listener".to_string()
    }
}
