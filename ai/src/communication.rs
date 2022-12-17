use net2;
use net2::unix::UnixUdpBuilderExt;
use prost::Message;
use std::error::Error;
use std::f32::consts::E;
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub trait Node {
    fn run_once(&mut self) -> Result<(), ()>;
}

pub fn run_forever(mut node: Box<dyn Node>, should_stop: Arc<AtomicBool>, name: &str) {
    loop {
        match node.run_once() {
            Err(_) => {
                println!("Terminating node {}", name);
                break;
            }
            _ => (),
        }
        if should_stop.load(Ordering::SeqCst) {
            println!("Terminating node {}", name);
            break;
        }
    }
}

pub fn dump_receiver<T>(mut receiver: &multiqueue2::BroadcastReceiver<T>) -> Result<Vec<T>, ()>
where
    T: Clone,
{
    let mut data: Vec<T> = vec![];
    loop {
        match receiver.try_recv() {
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

pub fn take_last<T>(mut receiver: &multiqueue2::BroadcastReceiver<T>) -> Result<Option<T>, ()>
where
    T: Clone,
{
    let mut data: Option<T> = None;
    loop {
        match receiver.try_recv() {
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

fn create_multicast_socket(ip: &str, port: u16) -> UdpSocket {
    let addr = SocketAddrV4::new(ip.parse::<Ipv4Addr>().unwrap(), port);
    let socket = net2::UdpBuilder::new_v4()
        .unwrap()
        .reuse_address(true)
        .unwrap()
        .reuse_port(true)
        .unwrap()
        .bind(&addr)
        .expect(format!("Couldn't bind socket to address {ip}:{port}").as_str());
    socket.join_multicast_v4(&addr.ip(), &Ipv4Addr::UNSPECIFIED);
    socket.set_nonblocking(true);
    socket
}

pub struct UdpMulticastClient {
    socket: UdpSocket,
    buffer: [u8; 65536],
}

impl UdpMulticastClient {
    pub fn new(ip: &str, port: u16) -> UdpMulticastClient {
        UdpMulticastClient {
            socket: create_multicast_socket(ip, port),
            buffer: [0; 65536],
        }
    }

    pub fn get_raw_bytes(&mut self) -> Result<&[u8], Box<dyn Error>> {
        let bytes_received = self.socket.recv(&mut self.buffer)?;
        let bytes = &mut self.buffer[..bytes_received];
        Ok(bytes)
    }

    pub fn read_proto<T>(&mut self) -> Result<T, Box<dyn Error>>
    where
        T: Message,
        T: Default,
    {
        let bytes = self.get_raw_bytes()?;
        let msg = T::decode(bytes)?;
        Ok(msg)
    }

    pub fn send_proto<T, A>(&mut self, msg: T, addr: A)
    where
        T: Message,
        T: Default,
        A: ToSocketAddrs,
    {
        // TODO: Try use pre-allocated struct buffer to avoid additional allocations
        let mut buf = Vec::new();
        buf.reserve(msg.encoded_len());
        // Unwrap is safe, since we have reserved sufficient capacity in the vector.
        msg.encode(&mut buf).unwrap();
        if let Err(e) = self.socket.send_to(&buf, addr) {
            println!("Failed to send proto over udp socket with error {e}");
        }
    }
}
