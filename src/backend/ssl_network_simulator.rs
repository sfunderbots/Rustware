use super::Input;
use crate::communication;
use crate::communication::{run_forever, Node, UdpMulticastClient};
use crate::motion::Trajectory;
use crate::proto;
use multiqueue2;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::Duration;

pub struct SslNetworkSimulator {
    pub input: Input,
    ssl_vision_udp_client: UdpMulticastClient,
}

impl Node for SslNetworkSimulator {
    fn run_once(&mut self) -> Result<(), ()> {
        if let Ok(trajectories) = self.input.trajectories.try_recv() {
            return Ok(());
        }
        Err(())
    }
}

impl SslNetworkSimulator {
    pub fn new(input: Input) -> Self {
        Self {
            input,
            ssl_vision_udp_client: communication::UdpMulticastClient::new("0.0.0.0", 10020),
        }
    }

    pub fn create_in_thread(input: Input, should_stop: &Arc<AtomicBool>) -> JoinHandle<()> {
        let should_stop = Arc::clone(should_stop);
        thread::spawn(move || {
            let node = Self::new(input);
            run_forever(Box::new(node), should_stop, "SslNetworkSimulator");
        })
    }
}
