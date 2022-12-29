use multiqueue2;
use multiqueue2::{BroadcastReceiver, BroadcastSender};
use net2;
use net2::unix::UnixUdpBuilderExt;
use prost::Message;
use std::collections::vec_deque::VecDeque;
use std::error::Error;
use std::f64::consts::E;
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket};
use std::ptr::addr_of_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{RecvError, TryRecvError, TrySendError};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant;

#[derive(Clone)]
struct CircularBuffer<T: Copy> {
    buffer: VecDeque<T>,
    capacity: usize,
}

impl<T: Copy> CircularBuffer<T> {
    pub fn new(capacity: usize) -> CircularBuffer<T> {
        CircularBuffer {
            buffer: VecDeque::<T>::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, item: T) {
        if self.buffer.len() == self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(item);
    }

    pub fn as_slice(&mut self) -> &[T] {
        self.buffer.make_contiguous()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

#[derive(Clone)]
pub struct NodeSender<T: Clone> {
    sender: BroadcastSender<T>,
    metrics_sender: BroadcastSender<(String, f64)>,
    pub_times_buffer: CircularBuffer<Instant>,
    topic_name: String,
}

impl<T: Clone> NodeSender<T> {
    pub fn try_send(&mut self, val: T) -> Result<(), TrySendError<T>> {
        self.pub_times_buffer.push(Instant::now());
        if self.pub_times_buffer.len() > 1 {
            let average_duration = self
                .pub_times_buffer
                .as_slice()
                .windows(2)
                .map(|x| x[1] - x[0])
                .sum::<Duration>()
                / (self.pub_times_buffer.len() - 1) as u32;
            let average_pub_period_ms = average_duration.as_secs_f64() * 1000.0;
            self.metrics_sender
                .try_send((self.topic_name.clone(), average_pub_period_ms));
        }
        self.sender.try_send(val)
    }
}

#[derive(Clone)]
pub struct NodeReceiver<T: Clone> {
    receiver: BroadcastReceiver<T>,
}

impl<T: Clone> NodeReceiver<T> {
    pub fn new(receiver: multiqueue2::BroadcastReceiver<T>) -> NodeReceiver<T> {
        NodeReceiver { receiver }
    }
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        self.receiver.try_recv()
    }
    pub fn recv(&self) -> Result<T, RecvError> {
        self.receiver.recv()
    }

    pub fn add_stream(&self) -> NodeReceiver<T> {
        NodeReceiver {
            receiver: self.receiver.add_stream(),
        }
    }

    pub fn unsubscribe(self) -> bool {
        self.receiver.unsubscribe()
    }

    pub fn dump(&self) -> Result<Vec<T>, ()> {
        let mut data: Vec<T> = vec![];
        loop {
            match self.try_recv() {
                Ok(msg) => data.push(msg),
                Err(e) => match e {
                    std::sync::mpsc::TryRecvError::Empty => break,
                    std::sync::mpsc::TryRecvError::Disconnected => {
                        return Err(());
                    }
                },
            };
        }
        Ok(data)
    }

    pub fn take_last(&self) -> Result<Option<T>, ()> {
        let mut data: Option<T> = None;
        loop {
            match self.try_recv() {
                Ok(msg) => data = Some(msg),
                Err(e) => match e {
                    std::sync::mpsc::TryRecvError::Empty => break,
                    std::sync::mpsc::TryRecvError::Disconnected => {
                        return Err(());
                    }
                },
            };
        }
        Ok(data)
    }
}

pub fn node_connection<T: Clone>(
    capacity: usize,
    metrics_sender: BroadcastSender<(String, f64)>,
    topic_name: String,
) -> (NodeSender<T>, NodeReceiver<T>) {
    // The broadcast_queue capacity is an internal "Index" type which is really just u64
    let (sender, receiver) = multiqueue2::broadcast_queue::<T>(capacity as u64);
    let node_sender = NodeSender {
        sender,
        metrics_sender,
        pub_times_buffer: CircularBuffer::new(50),
        topic_name,
    };
    let node_receiver = NodeReceiver { receiver };
    (node_sender, node_receiver)
}
