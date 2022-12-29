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
use std::sync::{Arc, Mutex};
// use parking_lot::{Mutex, MutexGuard, MappedMutexGuard, RawMutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant;
use std::marker::Send;
use crate::proto::config::Config;

pub trait Node {
    type Input;
    type Output;
    fn run_once(&mut self, config: &Config) -> Result<(), ()>;
    fn new(input: Self::Input, output: Self::Output) -> Self;
    fn name() -> String;
}

pub struct SynchronousRunner<T>
    where
        T: Node + Clone
{
    node: T,
    config: Arc<Mutex<Config>>
}

impl<T: Node + Clone> SynchronousRunner<T> {
    pub fn new(input: T::Input, output: T::Output, config: &Arc<Mutex<Config>>) -> Self {
        Self {
            node: T::new(input, output),
            config: Arc::clone(config)
        }
    }
    pub fn run_once(&mut self) {
        self.node.run_once(&self.config.lock().unwrap());
    }
    pub fn node(&self) -> &T {
        &self.node
    }
    pub fn mut_node(&mut self) -> &mut T {
        &mut self.node
    }
}

pub struct ThreadedRunner<T>
    where
        T: Node + Clone + Send + 'static
{
    node: Arc<Mutex<T>>,
    config: Arc<Mutex<Config>>,
    join_handle: JoinHandle<()>,
}

impl<T: Node + Clone + Send + 'static> ThreadedRunner<T> {
    pub fn new(input: T::Input, output: T::Output, config: &Arc<Mutex<Config>>, stop: &Arc<AtomicBool>) -> Self {
        let node = Arc::new(Mutex::new(T::new(input, output)));
        let stop = Arc::clone(stop);
        let thread_local_config = Arc::clone(config);
        Self {
            node: Arc::clone(&node),
            config: Arc::clone(config),
            join_handle: thread::spawn(move || {
                loop {
                    // TODO: locking the config like this before passing might cause lots of lock
                    // contention between threads
                    match node.lock().unwrap().run_once(&thread_local_config.lock().unwrap()) {
                        Err(_) => {
                            println!("Terminating node {}", T::name());
                            break;
                        }
                        _ => (),
                    }
                    if stop.load(Ordering::SeqCst) {
                        println!("Terminating node {}", T::name());
                        break;
                    }
                }
            })
        }
    }

    pub fn node(&self) -> Arc<Mutex<T>> {
        Arc::clone(&self.node)
    }
}
