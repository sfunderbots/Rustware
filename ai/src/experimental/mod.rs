use crate::communication::network::UdpMulticastClient;
use crate::proto;
use net2;
use net2::unix::UnixUdpBuilderExt;
use prost::Message;
// use socket2::{Domain, Protocol, Socket, Type};
use std::error::Error;
use std::io::Cursor;
use std::mem::MaybeUninit;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket};
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::Duration;

// pub mod ssl_vision {
//     include!(concat!(env!("OUT_DIR"), "/ssl_vision.rs"));
// }

// fn cast_mut<T>(val: &mut T) -> &mut MaybeUninit<T> {
//     // This is our assumption
//     unsafe { core::mem::transmute(val) }
// }

pub fn run() {
    // let config = Arc::new(Mutex::new(load_config().unwrap()));
    let context = zmq::Context::new();
    let sub_socket = context.socket(zmq::SUB).unwrap();
    sub_socket.set_subscribe("".as_bytes());
    sub_socket.connect("ipc:///tmp/underbots_zmq_gui_bridge");

    loop {
        println!("running");
        match sub_socket.recv_multipart(0) {
            Ok(msg) => {
                println!("got data");
            }
            Err(e) => {
                println!("error");
            }
        }
        sleep(Duration::from_millis(100));
    }

    // let ip = "224.5.23.2";
    // let port = 10020;
    // let addr = SocketAddrV4::new(ip.parse::<Ipv4Addr>().unwrap(), port);
    // let foo = net2::UdpBuilder::new_v4()
    //     .unwrap()
    //     .reuse_address(true)
    //     .unwrap()
    //     .reuse_port(true)
    //     .unwrap()
    //     .bind(&addr)
    //     .unwrap();
    // foo.set_nonblocking(true);
    // foo.join_multicast_v4(&addr.ip(), &Ipv4Addr::UNSPECIFIED);
    //
    // let s = communication::create_multicast_socket("224.5.23.2", 10020);
    // let mut buffer = [0; 65536];
    // loop {
    //     if let Ok(bytes_received) = s.recv(&mut buffer) {
    //         println!("Got data");
    //     }
    // }

    // let s = communication::create_multicast_socket("224.5.23.2", 10020);
    // let mut buffer: [MaybeUninit<u8>; 65536] = [MaybeUninit::new(0); 65536];
    // loop {
    //     if let Ok(bytes_received) = s.recv(&mut buffer) {
    //         println!("Got data");
    //     }
    // }

    let mut client = UdpMulticastClient::new("224.5.23.2", 10020);
    loop {
        // if let Ok(bytes) = client.get_raw_bytes() {
        //     println!("Got data");
        // } else {
        //     println!("no");
        // }
        match client.read_proto::<proto::ssl_vision::SslWrapperPacket>() {
            Ok(msg) => {
                println!("got msg");
            }
            Err(e) => {
                println!("Error {e}");
            }
        }
        // if let Ok(msg) =  {
        //     println!("Got proto");
        // } else {
        //     println!("no protro");
        // }

        sleep(Duration::from_millis(100));
    }

    // let s = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
    // s.set_nonblocking(true);
    // s.set_reuse_address(true);
    // s.join
    // s.set_reuse_port(true);
    // let mut listener = UdpMulticastClient::new("224.5.23.2", 10020);
    // loop {
    //     if let Ok(msg) = listener.read_proto::<ssl_vision::SslWrapperPacket>() {
    //         println!("Got data");
    //         listener.send_proto(msg, "0.0.0.0:10030");
    //     } else {
    //         println!("no data");
    //     }
    // }
}
