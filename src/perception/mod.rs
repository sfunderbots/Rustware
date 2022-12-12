mod ball_filter;

use crate::communication::{run_forever, Node};
use crate::proto;
use crate::world::{Ball, World};
use multiqueue2;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use egui::style::default_text_styles;
use ball_filter::{BallFilter, BallDetection};
use crate::geom::Point;

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
    ball_filter: BallFilter
}

impl Node for Perception {
    fn run_once(&mut self) -> Result<(), ()> {
        let mut ssl_wrapper_packets: Vec<proto::ssl_vision::SslWrapperPacket> = vec![];
        loop {
            match self.input.ssl_vision_proto.try_recv() {
                Ok(msg) => ssl_wrapper_packets.push(msg),
                Err(e) => match e {
                    std::sync::mpsc::TryRecvError::Empty => break,
                    std::sync::mpsc::TryRecvError::Disconnected => {
                        println!("Breaking perception loop");
                        return Err(());
                    }
                },
            };
        }

        if !ssl_wrapper_packets.is_empty() {
            for packet in ssl_wrapper_packets {
                if let Some(detection) = packet.detection {
                    for b in detection.balls {
                        let d = BallDetection{
                            position: Point{
                                x: b.x,
                                y: b.y
                            },
                            timestamp: detection.t_capture as f32
                        };
                        self.ball_filter.add_detection(d);
                    }
                }
            }

            let filtered_ball = self.ball_filter.get_ball();
        }
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
            ball_filter: BallFilter::new()
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
