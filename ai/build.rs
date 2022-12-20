use protobuf_codegen::Customize;
use std::io::Result;
use std::path::Path;
use std::env;
use cmake;

fn main() {
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
    // println!("cargo:rustc-link-search=/home/mathew/Projects/rustware/third_party/erforce_simulator/build/src/amun/simulator/");
    // println!("cargo:rustc-link-search=/usr/local/lib");
    // println!("cargo:rustc-link-search=.");
    // let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    // println!("cargo:rustc-link-search=native={}", Path::new(&dir).display());
    // println!("cargo:rustc-link-lib=static=mathewtest");
    // println!("cargo:rustc-link-lib=/home/mathew/Projects/rustware/third_party/erforce_simulator/build/src/amun/simulator/libsimulator.a");
    // println!("cargo:rustc-link-lib=static=simulator");
    // println!("cargo:rustc-link-search=.");
    // println!("cargo:rustc-link-search=native=.");
    // println!("cargo:rustc-link-lib=simulator");

    // let dst = cmake::build("/home/mathew/Projects/rustware/third_party/erforce_simulator/src/amun/simulator/");


    cxx_build::bridge("src/main.rs")
        .file("src/ersim_wrapper/ersim.cpp")
        .flag("-I/home/mathew/Projects/rustware/third_party/erforce_simulator/src/amun/simulator/include/")
        .flag("-I/home/mathew/Projects/rustware/third_party/erforce_simulator/src/amun/simulator/")
        .flag("-I/home/mathew/Projects/rustware/third_party/erforce_simulator/src/protobuf/include/")
        .flag("-I/home/mathew/Projects/rustware/third_party/erforce_simulator/build/src/")
        .flag("-I/usr/include/x86_64-linux-gnu/qt5/QtCore/")
        .flag("-I/usr/include/x86_64-linux-gnu/qt5/")
        // .flag("-lsnappy")
        // .flag("-lmathewtestcpp")
        // .flag("-lsimulator")
        // .flag("-lsimulator_wrapper")
        // .file("src/ersim_wrapper/ersim.h")
        // .flag_if_supported("-std=c++14")
        // .flag("-lmathewtestcpp")
        // .flag("-lmathewtest")
        // .flag("-lmtest")
        .compile("cxxbridge-demo");
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/ersim_wrapper/ersim.cpp");
    println!("cargo:rerun-if-changed=src/ersim_wrapper/ersim.h");

    // println!("cargo:rustc-link-search=/home/mathew/Projects/rustware/third_party/erforce_simulator/build/");
    println!("cargo:rustc-link-search=/home/mathew/Projects/rustware/third_party/erforce_simulator/build/src/amun/simulator/");
    // println!("cargo:rustc-link-search=/home/mathew/Projects/rustware/third_party/erforce_simulator/build/project_bullet-prefix/lib/");
    // println!("cargo:rustc-link-search=/home/mathew/Projects/rustware/third_party/erforce_simulator/build/src/amun/simulator/");
    // THESE MUST GO AT THE END SO LINKER FLAGS ARE IN THE RIGHT ORDER
    // println!("cargo:rustc-link-lib=simulator_wrapper");
    println!("cargo:rustc-link-lib=simulator");
    // println!("cargo:rustc-link-lib=BulletDynamics");
    // println!("cargo:rustc-link-lib=BulletCollision");
    // println!("cargo:rustc-link-lib=BulletSoftBody");
    // println!("cargo:rustc-link-lib=LinearMath");
    // println!("cargo:rustc-link-lib=static=mathewtestcpp");
    // println!("cargo:rustc-link-lib=static=simulator_wrapper");
    // println!("cargo:rustc-link-lib=static=simulator");
    // println!("cargo:rustc-link-lib=dylib=snappy");

}
