
use prost::Message;

pub mod ssl_vision {
    include!(concat!(env!("OUT_DIR"), "/ssl_vision.rs"));
}

pub mod ssl_gamecontroller {
    include!(concat!(env!("OUT_DIR"), "/ssl_game_controller.rs"));
}

pub mod ssl_simulation {
    include!(concat!(env!("OUT_DIR"), "/ssl_simulation_protocol.rs"));
}

// This is the config. Not sure why the filename is an underscore
include!(concat!(env!("OUT_DIR"), "/_.rs"));

// pub mod protos {
//     // This is the config. Not sure why the filename is an underscore
//     include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
// }
// pub mod test_out {
    // This is the config. Not sure why the filename is an underscore
    include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
// include!(concat!(env!("OUT_DIR"), "/ssl_vision/mod.rs"));
// }


pub fn encode<T>(msg: T) -> Vec<u8>
where
    T: Message,
    T: Default,
{
    let mut buf = Vec::new();
    buf.reserve(msg.encoded_len());
    msg.encode(&mut buf).unwrap();
    buf
}
