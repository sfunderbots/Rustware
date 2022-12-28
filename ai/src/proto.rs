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

pub mod trajectory {
    include!(concat!(env!("OUT_DIR"), "/trajectory.rs"));
}

pub mod world {
    include!(concat!(env!("OUT_DIR"), "/world.rs"));
}

pub mod metrics {
    include!(concat!(env!("OUT_DIR"), "/metrics.rs"));
}

// pub mod ssl {
//     include!(concat!(env!("OUT_DIR"), "/ssl.rs"));
// }

// Prost
pub mod config {
    include!(concat!(env!("OUT_DIR"), "/config.rs"));
}
// rust-protobuf: only used for text serialization/deserialization
pub mod internal {
    include!(concat!(env!("OUT_DIR"), "/config/mod.rs"));
}

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
