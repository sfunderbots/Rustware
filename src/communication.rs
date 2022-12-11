use net2;
use net2::unix::UnixUdpBuilderExt;
use prost::Message;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::error::Error;
use std::fmt::format;
use std::io::Cursor;
use std::mem::MaybeUninit;
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

// pub fn create_multicast_socket(ip: &str, port: u16) -> Socket {
//     let addr = SocketAddrV4::new(ip.parse::<Ipv4Addr>().unwrap(), port);
//     // let sockaddr: SocketAddr = "224.5.23.2:10020".parse().unwrap();
//     // let foo = addr.into();
//     // let foo = sockaddr.into();
//     let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
//         .expect(format!("Couldn't bind socket to address {ip}:{port}").as_str());
//     // let socket = UdpSocket::bind(&addr)
//     //     .expect(format!("Couldn't bind socket to address {ip}:{port}").as_str());
//     socket.set_nonblocking(true);
//     socket.join_multicast_v4(&addr.ip(), &Ipv4Addr::UNSPECIFIED);
//     // let foo = socket.into_udp_socket();
//     socket.bind(&addr.into());
//     socket
// }

pub fn create_multicast_socket(ip: &str, port: u16) -> UdpSocket {
    let addr = SocketAddrV4::new(ip.parse::<Ipv4Addr>().unwrap(), port);
    // let sockaddr: SocketAddr = "224.5.23.2:10020".parse().unwrap();
    // let foo = addr.into();
    // let foo = sockaddr.into();
    // let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
    //     .expect(format!("Couldn't bind socket to address {ip}:{port}").as_str());
    let socket = UdpSocket::bind(&addr)
        .expect(format!("Couldn't bind socket to address {ip}:{port}").as_str());

    // let socket = net2::UdpBuilder::new_v4()
    //     .unwrap()
    //     .reuse_address(true)
    //     .unwrap()
    //     .reuse_port(true)
    //     .unwrap()
    //     .bind(&addr)
    //     .unwrap();

    // socket.set_nonblocking(true);
    socket.join_multicast_v4(&addr.ip(), &Ipv4Addr::UNSPECIFIED);
    // let foo = socket.into_udp_socket();
    // socket.bind(&addr.into());
    socket
}

pub struct UdpMulticastClient {
    socket: UdpSocket,
    buffer: [u8; 65536],
    // buffer: [MaybeUninit<u8>; 65536],
}

impl UdpMulticastClient {
    pub fn new(ip: &str, port: u16) -> UdpMulticastClient {
        UdpMulticastClient {
            socket: create_multicast_socket(ip, port),
            // buffer: [MaybeUninit::new(0); 65536],
            buffer: [0; 65536],
        }
    }

    pub fn get_raw_bytes(&mut self) -> Result<&[u8], Box<dyn Error>> {
        // let bytes_received = self.socket.recv(&mut self.buffer)?;
        let bytes_received = 5;
        let bytes = &mut self.buffer[..bytes_received];
        Ok(bytes)
        // let mut v: Vec<u8> = Vec::new();
        // for b in bytes {
        //     unsafe {
        //         v.push(b.assume_init());
        //     }
        // }
        // Ok(v)
        // let valid_bytes: [u8; 65536] = bytes.iter().map(|x| x.assume_init()).collect();
        // let valid_bytes: Vec<u8> = bytes.iter().map(|x| x.assume_init()).collect();
        // unsafe {
        //     let valid_bytes = std::mem::transmute_copy(&bytes);
        //     return Ok(valid_bytes);
        // }
        // let valid_bytes: [u8; 65536] = bytes.try_into().unwrap();
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
        // if let Err(e) = self.socket.send_to(&buf, addr) {
        //     println!("Failed to send proto over udp socket with error {e}");
        // }
    }
}
