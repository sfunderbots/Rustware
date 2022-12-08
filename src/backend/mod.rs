use crate::communication::Node;
use multiqueue2;
use std::thread::sleep;
use std::time::Duration;

pub struct Input {
    pub trajectories: multiqueue2::MPMCReceiver<i32>,
}
pub struct Output {
    pub ssl_vision_proto: multiqueue2::MPMCSender<i32>,
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
        println!("Backend got packet {}", packet.unwrap_or(-1));

        self.output.ssl_vision_proto.try_send(8);
        sleep(Duration::from_millis(100));
        Ok(())
    }
}

impl Backend {
    pub fn send_dummy_data(&self, data: i32) {
        self.output.ssl_vision_proto.try_send(data).unwrap();
        println!("Send ssl vision {}", data);
    }
}
