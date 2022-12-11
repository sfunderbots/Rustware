use prost::Message;
use std::error::Error;
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub fn create_multicast_socket(ip: &str, port: u16) -> UdpSocket {
    let addr = SocketAddrV4::new(ip.parse::<Ipv4Addr>().unwrap(), port);
    let socket = UdpSocket::bind(&addr).unwrap();
    socket.join_multicast_v4(&addr.ip(), &Ipv4Addr::UNSPECIFIED);
    socket
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    socket
}

struct UdpMulticastListener {
    socket: UdpSocket,
    buffer: [u8; 65536],
}

impl UdpMulticastListener {
    pub fn new(ip: &str, port: u16) -> UdpMulticastListener {
        UdpMulticastListener {
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
}

pub mod ssl_vision {
    include!(concat!(env!("OUT_DIR"), "/ssl_vision.rs"));
}
pub fn run() {
    let mut listener = UdpMulticastListener::new("224.5.23.2", 10020);
    loop {
        if let Ok(msg) = listener.read_proto::<ssl_vision::SslWrapperPacket>() {
            println!("Got data");
        } else {
            println!("no data");
        }
    }
}
