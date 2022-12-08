use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

pub trait Node {
    fn run_once(&mut self) -> Result<(), ()>;
}

pub fn run_forever(mut node: Box<dyn Node>, should_stop: Arc<AtomicBool>, name: &str) {
    loop {
        match node.run_once() {
            Err(_) => {
                println!("Terminating node {}", name);
                break;
            }
            _ => (),
        }
        if should_stop.load(Ordering::SeqCst) {
            println!("Terminating node {}", name);
            break;
        }
    }
}
