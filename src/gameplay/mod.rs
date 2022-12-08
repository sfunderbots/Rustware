use multiqueue2;
use crate::communication::Node;

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

impl Node for Gameplay {
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
