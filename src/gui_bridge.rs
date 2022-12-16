use crate::communication::{run_forever, Node};
use crate::motion::Trajectory;
use crate::proto;
use crate::world::World;
use prost::Message;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub struct Input {
    pub ssl_vision_proto: multiqueue2::BroadcastReceiver<proto::ssl_vision::SslWrapperPacket>,
}
pub struct Output {}

pub struct GuiBridge {
    input: Input,
    output: Output,
    context: zmq::Context,
    socket: zmq::Socket,
}

impl GuiBridge {
    pub fn new(input: Input, output: Output) -> Self {
        let ctx = zmq::Context::new();
        let socket = ctx.socket(zmq::PUB).unwrap();
        socket.bind("ipc:///tmp/underbots_zmq_test").unwrap();
        Self {
            input,
            output,
            context: ctx,
            socket: socket,
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
            run_forever(Box::new(node), should_stop, "GuiBridge");
        })
    }
}

impl Node for GuiBridge {
    fn run_once(&mut self) -> Result<(), ()> {
        if let Ok(ssl_wrapper_packet) = self.input.ssl_vision_proto.try_recv() {
            self.socket
                .send(proto::encode(ssl_wrapper_packet), 0)
                .unwrap();
        }
        // thread::sleep(Duration::from_millis(10));
        Ok(())
    }
}
