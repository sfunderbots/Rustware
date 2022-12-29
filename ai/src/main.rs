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

fn run_nodes_in_parallel_threads() {
    let nodes = setup::create_threaded_nodes(setup::set_up_node_io());
    sleep(Duration::from_secs(1));
    nodes.ssl_simulator.node().lock().unwrap().remove_all_robots();
    sleep(Duration::from_secs(3));
    nodes.ssl_simulator.node().lock().unwrap().set_robot(0, Point{x: 0.0, y: 0.0}, true);
    nodes.ssl_simulator.node().lock().unwrap().set_robot(1, Point{x: 1.0, y: 0.0}, true);
    sleep(Duration::from_secs(1));
    nodes.ssl_simulator.node().lock().unwrap().set_robot(1, Point{x: 1.0, y: -1.0}, false);

    println!("Sleeping to simulate working time");
    sleep(Duration::from_secs(500000));
    println!("Done sleeping. Sending stop signal");
    nodes.stop();
    println!("About to join");
    nodes.join();
    println!("Done join");
}


// TODO: Note to self - timestamps should always be f64, since the erforce sim uses unix time
// timestamps, which are too big for f64. The t_capture given in proto is f64, so we should respect that
fn main() {
    // experimental::run();
    run_nodes_in_parallel_threads();
}
