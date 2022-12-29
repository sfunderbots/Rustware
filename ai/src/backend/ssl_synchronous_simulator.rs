use super::{Input, Output};
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

pub struct SslSynchronousSimulator {
    pub input: Input,
    pub output: Output,
}

impl Node for SslSynchronousSimulator {
    fn run_once(&mut self) -> Result<(), ()> {
        todo!()
    }
}

impl SslSynchronousSimulator {
    pub fn new(input: Input, output: Output) -> Self {
        Self { input, output }
    }
}
