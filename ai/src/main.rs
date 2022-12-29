#![allow(dead_code)]
#![allow(unused_variables)]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

mod backend;
mod config;
mod constants;
mod experimental;
mod gameplay;
mod geom;
mod gui_bridge;
mod macros;
mod math;
mod motion;
mod perception;
mod proto;
mod proto_conversions;
mod world;
mod simulation;
mod setup;
mod communication;

use geom::Point;

// fn run_nodes_synchronously() {
//     let mut nodes = setup::create_synchronous_nodes(setup::set_up_node_io());
//
//     for i in 0..1000000 {
//         // nodes.backend.send_dummy_data(i);
//         // nodes.perception.run_once();
//         // nodes.gameplay.run_once();
//         // nodes.backend.run_once();
//         nodes.gui_bridge.run_once();
//         sleep(Duration::from_millis(100));
//     }
// }

fn run_nodes_in_parallel_threads() {
    let mut should_stop = Arc::new(AtomicBool::new(false));
    let nodes = setup::create_threaded_nodes(setup::set_up_node_io(), &should_stop);

    println!("Sleeping to simulate working time");
    sleep(Duration::from_secs(500000));
    println!("Done sleeping. Sending stop signal");
    should_stop.store(true, Ordering::SeqCst);
    println!("About to join");
    nodes.join();
    println!("Done join");
}

// fn run_test_in_threads() {
//     let mut should_stop = Arc::new(AtomicBool::new(false));
//     let (handles, mut test_runner) = setup::create_test_nodes_in_threads(setup::set_up_node_io(), &should_stop);
//     // sleep(Duration::from_secs(1));
//     // test_runner.remove_all_robots();
//     // println!("removed robots");
//     // sleep(Duration::from_secs(1));
//     // test_runner.set_robot(0, Point{x: 0.0, y: 0.0}, true);
//     // println!("add robot 0");
//     // sleep(Duration::from_secs(1));
//     // test_runner.set_robot(1, Point{x: 1.0, y: 0.0}, true);
//     // println!("add robot 1");
//
//     println!("Sleeping to simulate working time");
//     sleep(Duration::from_secs(500000));
//     println!("Done sleeping. Sending stop signal");
//     should_stop.store(true, Ordering::SeqCst);
//     println!("About to join");
//     for handle in handles {
//         handle.join();
//     }
//     println!("Done join");
// }

// TODO: Note to self - timestamps should always be f64, since the erforce sim uses unix time
// timestamps, which are too big for f64. The t_capture given in proto is f64, so we should respect that
fn main() {
    // let cfg = config::load_config().expect("config should be fully initialized");
    // println!("{}", cfg.backend.ssl_vision_ip);
    // experimental::run();
    // run_nodes_synchronously();
    run_nodes_in_parallel_threads();
    // run_test_in_threads();

    // println!("square is {}", r);
}
