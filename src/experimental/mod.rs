use prost::Message;
use std::error::Error;
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

// pub mod ssl_vision {
//     include!(concat!(env!("OUT_DIR"), "/ssl_vision.rs"));
// }
pub fn run() {
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
