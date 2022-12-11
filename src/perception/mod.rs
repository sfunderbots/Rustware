use crate::communication::{run_forever, Node};
use crate::proto;
use crate::world::World;
use multiqueue2;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

pub struct Input {
    pub ssl_vision_proto: multiqueue2::MPMCReceiver<proto::ssl_vision::SslWrapperPacket>,
    pub ssl_refbox_proto: multiqueue2::MPMCReceiver<proto::ssl_gamecontroller::Referee>,
}
pub struct Output {
    pub world: multiqueue2::MPMCSender<World>,
}

pub struct Perception {
    pub input: Input,
    pub output: Output,
}

impl Node for Perception {
    fn run_once(&mut self) -> Result<(), ()> {
        let packet = match self.input.ssl_vision_proto.try_recv() {
            Ok(p) => p,
            Err(e) => match e {
                std::sync::mpsc::TryRecvError::Empty => return Ok(()),
                std::sync::mpsc::TryRecvError::Disconnected => {
                    println!("Breaking perception loop");
                    return Err(());
                }
            },
        };
        // println!("Perception got packet {}", packet);
        // self.output.world.try_send();
        Ok(())
    }
}

impl Perception {
    pub fn new(input: Input, output: Output) -> Self {
        Self {
            input: input,
            output: output,
        }
    }
    pub fn create_in_thread(
        input: Input,
        output: Output,
        should_stop: &Arc<AtomicBool>,
    ) -> JoinHandle<()> {
        let should_stop = Arc::clone(should_stop);
        thread::spawn(move || {
            let node = Self::new(input, output);
            run_forever(Box::new(node), should_stop, "Perception");
        })
    }
}
