use multiqueue2;
use crate::communication::Node;

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

impl Node for Perception {
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
