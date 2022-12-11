pub mod ssl_vision {
    include!(concat!(env!("OUT_DIR"), "/ssl_vision.rs"));
}

pub mod ssl_gamecontroller {
    include!(concat!(env!("OUT_DIR"), "/ssl_game_controller.rs"));
}

pub mod ssl_simulation {
    include!(concat!(env!("OUT_DIR"), "/ssl_simulation_protocol.rs"));
}
