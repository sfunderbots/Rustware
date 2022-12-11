use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

pub fn run() {
    let basic_addr = Ipv4Addr::new(224, 5, 23, 2);
    let addr = IpAddr::V4(basic_addr);
    let port = 10020;
    let sock_addr = SocketAddr::new(addr, port);
    // let mut socket = UdpSocket::bind("224.5.23.2:10020").unwrap();
    let mut socket = UdpSocket::bind(sock_addr).unwrap();
    let interface = Ipv4Addr::new(127, 0, 0, 1);
    socket
        .join_multicast_v4(&basic_addr, &Ipv4Addr::UNSPECIFIED)
        .unwrap();

    let mut buf = vec![0; 65536];
    loop {
        let (amt, src) = socket.recv_from(&mut buf).unwrap();
        let buf = &mut buf[..amt];
        println!("Got packet");
    }
}
