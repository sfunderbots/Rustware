use super::Output;
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

pub struct SslNetworkListener {
    pub output: Output,
    ssl_vision_udp_client: communication::UdpMulticastClient,
}

impl Node for SslNetworkListener {
    fn run_once(&mut self) -> Result<(), ()> {
        todo!()
    }
}

impl SslNetworkListener {
    pub fn new(output: Output) -> Self {
        Self {
            output: output,
            ssl_vision_udp_client: communication::UdpMulticastClient::new("224.5.23.2", 10020),
        }
    }

    pub fn create_in_thread(output: Output, should_stop: &Arc<AtomicBool>) -> JoinHandle<()> {
        let should_stop = Arc::clone(should_stop);
        thread::spawn(move || {
            let node = Self::new(output);
            run_forever(Box::new(node), should_stop, "SslNetworkListener");
        })
    }
}
