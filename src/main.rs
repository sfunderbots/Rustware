#![allow(dead_code)]
#![allow(unused_variables)]

mod geom;
mod math;
mod motion;
mod world;
mod evaluation;
mod experimental;
mod perception;
mod communication;
mod gameplay;

use crate::communication::Node;
use crate::geom::{Point, Vector};
use crate::math::{rect_sigmoid, sigmoid};
use crate::motion::bb_time_to_position;
use crate::world::{Field, Robot};
use rand::Rng;
use std::time::Instant;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use multiqueue2;


fn run_sync() {
    let (ssl_w, ssl_r) = multiqueue2::mpmc_queue::<i32>(100);
    let (world_w, world_r) = multiqueue2::mpmc_queue::<i32>(100);
    let (traj_w, traj_r) = multiqueue2::mpmc_queue::<i32>(100);
    let mut p = perception::Perception{
        input: perception::Input{
            ssl_vision_proto: ssl_r.clone(),
        },
        output: perception::Output{
            world: world_w.clone()
        }
    };
    let mut g = gameplay::Gameplay{
        input: gameplay::Input{
            world: world_r.clone()
        },
        output: gameplay::Output{
            trajectories: traj_w.clone()
        }
    };

    for i in 0..100 {
        ssl_w.try_send(i).unwrap();
        println!("Send ssl vision {}", i);
        p.run_once();
        g.run_once();
        sleep(Duration::from_millis(100));
    }
}

fn run_forever(mut n: Box<dyn Node>, stop: Arc<AtomicBool>, name: &str) {
    loop {
        match n.run_once() {
            Err(_) => {
                println!("Terminating node {}", name);
                break;
            }
            _ => ()
        }
        if stop.load(Ordering::SeqCst) {
            println!("Terminating node {}", name);
            break;
        }
    }
}


fn run_in_threads() {
    println!("Hello");

    let (ssl_w, ssl_r) = multiqueue2::mpmc_queue::<i32>(100);
    let (world_w, world_r) = multiqueue2::mpmc_queue::<i32>(100);
    let (traj_w, traj_r) = multiqueue2::mpmc_queue::<i32>(100);
    let mut p = perception::Perception{
        input: perception::Input{
            ssl_vision_proto: ssl_r.clone(),
        },
        output: perception::Output{
            world: world_w.clone()
        }
    };
    let mut g = gameplay::Gameplay{
        input: gameplay::Input{
            world: world_r.clone()
        },
        output: gameplay::Output{
            trajectories: traj_w.clone()
        }
    };


    let mut should_stop = Arc::new(AtomicBool::new(false));

    let p_stop = Arc::clone(&should_stop);
    let p_handle = thread::spawn(move || {
        run_forever(Box::new(p), p_stop, "Perception");
    });
    let g_stop = Arc::clone(&should_stop);
    let g_handle = thread::spawn(move || {
        run_forever(Box::new(g), g_stop, "Gameplay");
    });

    for i in 0..10 {
        ssl_w.try_send(i).unwrap();
        println!("Send ssl vision {}", i);
        sleep(Duration::from_millis(10));
    }


    should_stop.store(true, Ordering::SeqCst);

    println!("About to joiun");
    p_handle.join();
    g_handle.join();
}

fn main() {
    // experimental::run();
    run_sync();
    // run_in_threads();
}
