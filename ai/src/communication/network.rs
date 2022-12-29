use net2;
use net2::unix::UnixUdpBuilderExt;
use prost::Message;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket};

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

// TODO: split this into sender and receiver, since pure senders
// don't need to bind on creation
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
