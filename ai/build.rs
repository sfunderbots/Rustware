use protobuf_codegen::Customize;
use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(
        &[
            "../third_party/ssl_vision/messages_robocup_ssl_geometry.proto",
            "../third_party/ssl_vision/messages_robocup_ssl_detection.proto",
            "../third_party/ssl_vision/messages_robocup_ssl_wrapper.proto",
        ],
        &["../third_party/"],
    )
    .unwrap();
    prost_build::compile_protos(
        &[
            "../third_party/ssl_simulation_protocol/ssl_gc_common.proto",
            "../third_party/ssl_simulation_protocol/ssl_simulation_config.proto",
            "../third_party/ssl_simulation_protocol/ssl_simulation_control.proto",
            "../third_party/ssl_simulation_protocol/ssl_simulation_error.proto",
            "../third_party/ssl_simulation_protocol/ssl_simulation_robot_control.proto",
            "../third_party/ssl_simulation_protocol/ssl_simulation_robot_feedback.proto",
            "../third_party/ssl_simulation_protocol/ssl_simulation_synchronous.proto",
            "../third_party/ssl_simulation_protocol/ssl_vision_geometry.proto",
            "../third_party/ssl_simulation_protocol/ssl_vision_detection.proto",
        ],
        &["../third_party/"],
    )
    .unwrap();
    prost_build::compile_protos(
        &[
            "../third_party/ssl_game_controller/ssl_autoref_ci.proto",
            "../third_party/ssl_game_controller/ssl_gc_api.proto",
            "../third_party/ssl_game_controller/ssl_gc_change.proto",
            "../third_party/ssl_game_controller/ssl_gc_ci.proto",
            "../third_party/ssl_game_controller/ssl_gc_common.proto",
            "../third_party/ssl_game_controller/ssl_gc_engine.proto",
            "../third_party/ssl_game_controller/ssl_gc_engine_config.proto",
            "../third_party/ssl_game_controller/ssl_gc_game_event.proto",
            "../third_party/ssl_game_controller/ssl_gc_geometry.proto",
            "../third_party/ssl_game_controller/ssl_gc_rcon.proto",
            "../third_party/ssl_game_controller/ssl_gc_rcon_autoref.proto",
            "../third_party/ssl_game_controller/ssl_gc_rcon_remotecontrol.proto",
            "../third_party/ssl_game_controller/ssl_gc_rcon_team.proto",
            "../third_party/ssl_game_controller/ssl_gc_referee_message.proto",
            "../third_party/ssl_game_controller/ssl_gc_state.proto",
            "../third_party/ssl_game_controller/ssl_vision_detection.proto",
            "../third_party/ssl_game_controller/ssl_vision_detection_tracked.proto",
            "../third_party/ssl_game_controller/ssl_vision_geometry.proto",
            "../third_party/ssl_game_controller/ssl_vision_wrapper.proto",
            "../third_party/ssl_game_controller/ssl_vision_wrapper_tracked.proto",
        ],
        &["../third_party/"],
    )
    .unwrap();
    prost_build::compile_protos(
        &["../config/config.proto", "../proto/visualization.proto"],
        &["../config/", "../proto/"],
    )
    .unwrap();
    protobuf_codegen::Codegen::new()
        .includes(&["../config"])
        .input("../config/config.proto")
        .cargo_out_dir("config")
        .run_from_script();

    Ok(())
}
