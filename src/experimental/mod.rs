use std::error::Error;
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

    pub fn get(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let bytes_received = self.socket.recv(&mut self.buffer)?;
        let bytes = &mut self.buffer[..bytes_received];
        Ok(bytes.to_vec())
    }
}

pub fn run() {
    let mut listener = UdpMulticastListener::new("224.5.23.2", 10020);
    loop {
        // let result = listener.get().unwrap_or(vec![]);
        if let Ok(result) = listener.get() {
            println!("Got data");
        } else {
            println!("no data");
        }
    }

    // let socket = create_multicast_socket("224.5.23.2", 10020);
    // let mut buf = vec![0; 65536];
    // loop {
    //     match socket.recv(&mut buf) {
    //         Ok(received_size) => {
    //             let buf = &mut buf[..received_size];
    //         }
    //         Err(_) => break,
    //     }
    //     let read_bytes = socket.recv(&mut buf).unwrap();
    //     let buf = &mut buf[..read_bytes];
    //     println!("Got packet");
    // }
}
