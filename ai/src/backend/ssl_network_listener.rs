use super::Output;
use crate::communication_old;
use crate::communication_old::{run_forever, Node, UdpMulticastClient};
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
    ssl_vision_udp_client: UdpMulticastClient,
    ssl_gamecontroller_udp_client: UdpMulticastClient,
}

impl Node for SslNetworkListener {
    fn run_once(&mut self) -> Result<(), ()> {
        loop {
            match self
                .ssl_vision_udp_client
                .read_proto::<proto::ssl_vision::SslWrapperPacket>()
            {
                Ok(msg) => {
                    match self.output.ssl_vision.try_send(msg) {
                        Ok(_) => {
                            // println!("Sent data from backend");
                        }
                        Err(e) => {
                            println!("Failed to push to buffer with error {}", e);
                        }
                    };
                    // println!("Send data");
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
}

impl SslNetworkListener {
    pub fn new(output: Output) -> Self {
        Self {
            output: output,
            ssl_vision_udp_client: communication_old::UdpMulticastClient::new("224.5.23.2", 10020),
            ssl_gamecontroller_udp_client: communication_old::UdpMulticastClient::new(
                "224.5.23.1",
                10003,
            ),
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
