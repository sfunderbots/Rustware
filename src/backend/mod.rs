use crate::communication::{run_forever, Node};
use crate::motion::Trajectory;
use crate::proto;
use multiqueue2;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::Duration;

pub struct Input {
    pub trajectories: multiqueue2::MPMCReceiver<HashMap<usize, Trajectory>>,
}
pub struct Output {
    pub ssl_vision_proto: multiqueue2::MPMCSender<proto::ssl_vision::SslWrapperPacket>,
    pub ssl_referee_proto: multiqueue2::MPMCSender<proto::ssl_gamecontroller::Referee>,
}
pub struct Backend {
    pub input: Input,
    pub output: Output,
}

impl Node for Backend {
    fn run_once(&mut self) -> Result<(), ()> {
        let packet = match self.input.trajectories.try_recv() {
            Ok(p) => Some(p),
            Err(e) => match e {
                std::sync::mpsc::TryRecvError::Empty => None,
                std::sync::mpsc::TryRecvError::Disconnected => {
                    println!("Breaking backend loop");
                    return Err(());
                }
            },
        };
        // println!("Backend got packet {}", packet.unwrap_or(-1));
        //
        // self.output.ssl_vision_proto.try_send(8);
        // sleep(Duration::from_millis(100));
        Ok(())
    }
}

impl Backend {
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
            run_forever(Box::new(node), should_stop, "Backend");
        })
    }
}
