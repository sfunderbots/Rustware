use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use multiqueue2;


fn type_of<T>(_: &T) -> &str{
    std::any::type_name::<T>()
}

pub trait Node {
    fn run_once(&mut self) -> Result<(), ()>;
}


mod perception {
    pub struct Input {
        pub ssl_vision_proto: multiqueue2::MPMCReceiver<i32>,
        // pub ssl_refbox_proto: multiqueue2::MPMCReceiver<i32>,
    }
    pub struct Output {
        pub world: multiqueue2::MPMCSender<i32>
    }

    pub struct Perception {
        pub input: Input,
        pub output: Output
    }

    impl super::Node for Perception {
        fn run_once(&mut self) -> Result<(), ()>{
            let packet = match self.input.ssl_vision_proto.try_recv() {
                Ok(p) => p,
                Err(e) => match e {
                    std::sync::mpsc::TryRecvError::Empty => return Ok(()),
                    std::sync::mpsc::TryRecvError::Disconnected => {
                        println!("Breaking perception loop");
                        return Err(())
                    }
                }
            };
            println!("Perception got packet {}", packet);
            self.output.world.try_send(packet);
            Ok(())
        }
    }
}

mod gameplay {
    pub struct Input {
        pub world: multiqueue2::MPMCReceiver<i32>
    }
    pub struct Output {
        pub trajectories: multiqueue2::MPMCSender<i32>
    }
    pub struct Gameplay {
        pub input: Input,
        pub output: Output
    }

    impl super::Node for Gameplay {
        fn run_once(&mut self) -> Result<(), ()>{
            let packet = match self.input.world.try_recv() {
                Ok(p) => p,
                Err(e) => match e {
                    std::sync::mpsc::TryRecvError::Empty => return Ok(()),
                    std::sync::mpsc::TryRecvError::Disconnected => {
                        println!("Breaking perception loop");
                        return Err(())
                    }
                }
            };
            println!("Gameplay got packet {}", packet);
            self.output.trajectories.try_send(packet);
            Ok(())
        }
    }
}
//
// fn run_in_threads() {
//     println!("Hello");
//
//     let (ssl_w, ssl_r) = multiqueue2::mpmc_queue::<i32>(100);
//     let (world_w, world_r) = multiqueue2::mpmc_queue::<i32>(100);
//     let (traj_w, traj_r) = multiqueue2::mpmc_queue::<i32>(100);
//     let mut p = perception::Perception{
//         input: perception::Input{
//             ssl_vision_proto: ssl_r.clone(),
//         },
//         output: perception::Output{
//             world: world_w.clone()
//         }
//     };
//     let mut g = gameplay::Gameplay{
//         input: gameplay::Input{
//             world: world_r.clone()
//         },
//         output: gameplay::Output{
//             trajectories: traj_w.clone()
//         }
//     };
//
//     let p_handle = thread::spawn(move || {
//         p.run_forever();
//     });
//     let g_handle = thread::spawn(move || {
//         g.run_forever();
//     });
//
//     for i in 0..100 {
//         ssl_w.try_send(i).unwrap();
//         println!("Send ssl vision {}", i);
//         sleep(Duration::from_millis(100));
//     }
//
//     drop(ssl_w);
//
//     println!("About to joiun");
//     p_handle.join();
//     g_handle.join();
// }

fn run_sync() {
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


fn run_in_threads2() {
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

pub fn run() {
    // run_sync();
    // run_in_threads();
    run_in_threads2();
}
