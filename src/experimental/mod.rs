use crossbeam::queue::ArrayQueue;
use std::sync::mpsc;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
// extern crate multiqueue2 as multiqueue;
use multiqueue2 as multiqueue;



pub trait Node {
    type Input;
    type Output;
    fn tick(&self, input: Self::Input) -> Self::Output;
    fn run(&mut self);
}

pub struct Topic<T: Clone>
{
    name: String,
    subscriber_queues: Vec<ArrayQueue<T>>,
    // data: Array
}

impl<T> Topic<T>
where
    T: Clone
{
    pub fn publish(&self, data: T) {
        for q in &self.subscriber_queues {
            q.force_push(data.clone());
        }
    }

    pub fn register_subscriber(&mut self, max_size: usize) -> &ArrayQueue<T> {
        self.subscriber_queues.push(ArrayQueue::<T>::new(max_size));
        self.subscriber_queues.iter().last().unwrap()
    }

    pub fn new() -> Topic<T> {
        Topic{name: "foo".to_string(), subscriber_queues: vec![]}
    }
}

// pub enum Topic {
//     PerceptionOutput,
//     Trajectories,
//     Config
// }

pub trait Publisher {
    // type Data;
    // pub fn subscribe(&self, publisher: impl)
}

pub trait Subscriber {
    type Data;
    fn subscribe(&self, publisher: impl Publisher);
}

pub struct Runner {
    node: perception::Perception,
    // topic_a: &Topic<i32>,
    // topic_b: &Topic<i32>
    // send_queues: Vec<ArrayQueue>
    // send_queues
}

// impl Runner {
//     pub fn run(&self) {
//         let data = 32;
//         self.topic_a.publish(data);
//
//     }
//
//     // pub fn publish(topic: Topic) {
//     //
//     // }
// }

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
enum TopicEnum {
    FOO,
    BAR
}

pub trait TopicMessage {}

struct TopicManager {
    data: std::collections::HashMap<TopicEnum, Vec<ArrayQueue<Box<dyn TopicMessage>>>>
}

// impl TopicManager {
//     pub fn publish<T: Clone + TopicMessage>(&mut self, topic: TopicEnum, data: T)
//     {
//         if !self.data.contains_key(&topic) {
//             self.data.insert(topic, vec![]);
//         }
//         let v = self.data.get_mut(&topic).unwrap();
//         for q in v {
//             // let new_data = ;
//             q.push(Box::new(data.clone()));
//         }
//         // let foo: &Vec<ArrayQueue<Box<dyn TopicMessage>>> = self.data.entry(topic).or_default();
//         // for q in foo {
//         //     q.force_push()
//         // }
//         // let foo = self.data.get(&topic).unwrap_or_default();
//         // let foo = self.data.get_mut(&topic).unwrap_or(vec![]);
//         // let foo = match self.data.get_mut(&topic) {
//         //     Some(_) => return,
//         //     None => return
//         // };
//     }
// }
// struct TopicManager {
//
// }

mod perception {
    // pub struct PerceptionInput
    pub struct Input {
        // raw protos
    }
    pub struct Output {
        // Filtered world, gamestate
    }

    pub struct Io {
        pub raw_ssl_vision_proto: multiqueue2::MPMCReceiver<i32>,
        pub world: multiqueue2::MPMCSender<i32>
    }

    pub struct Perception {
        pub io: Io
    }

    impl super::Node for Perception {
        type Input = Input;
        type Output = Output;
        fn tick(&self, input: Self::Input) -> Self::Output {
            // println!("{}", self.x);
            Output{}
        }

        fn run(&mut self) {
            loop {
                let packet = match self.io.raw_ssl_vision_proto.try_recv() {
                    Ok(p) => p,
                    Err(e) => match e {
                        std::sync::mpsc::TryRecvError::Empty => continue,
                        std::sync::mpsc::TryRecvError::Disconnected => {
                            println!("Breaking perception loop");
                            break
                        }
                    }
                };
                println!("Perception got packet {}", packet);
                self.io.world.try_send(packet);
            }
        }
    }
}

mod gameplay {
    pub struct Output {
        // Filtered world, gamestate
    }

    pub struct Gameplay {
        pub io: Io
    }

    pub struct Io {
        pub world_input: multiqueue2::MPMCReceiver<i32>,
        pub trajectory_output: multiqueue2::MPMCSender<i32>
    }

    impl super::Node for Gameplay {
        type Input = super::perception::Output;
        type Output = Output;
        fn tick(&self, input: Self::Input) -> Self::Output {
            // println!("{}", self.x);
            Output{}
        }

        fn run(&mut self) {
            loop {
                let packet = match self.io.world_input.try_recv() {
                    Ok(p) => p,
                    Err(e) => match e {
                        std::sync::mpsc::TryRecvError::Empty => continue,
                        std::sync::mpsc::TryRecvError::Disconnected => {
                            println!("Breaking Gameplay loop");
                            break
                        }
                    }
                };
                println!("Gameplay got packet {}", packet);
                self.io.trajectory_output.try_send(packet);
            }
        }
    }
}


// enum Topics {
//     FOO(Topic<perception::Output>)
// }

fn mq2_test() {
    let (send, recv) = multiqueue::mpmc_queue::<i32>(10);
    let handle = thread::spawn(move || {
        for val in recv {
            println!("Got {}", val);
        }
    });

    for i in 0..100 {
        send.try_send(i).unwrap();
    }

    drop(send);

    handle.join();
}

pub fn run() {
    println!("Hello");
    // let t: Topic<i32> = Topic::new();
    // t.publish(32);

    let (ssl_w, ssl_r) = multiqueue2::mpmc_queue::<i32>(100);
    let (world_w, world_r) = multiqueue2::mpmc_queue::<i32>(100);
    let (traj_w, traj_r) = multiqueue2::mpmc_queue::<i32>(100);
    let mut p = perception::Perception{
        io: perception::Io{
            raw_ssl_vision_proto: ssl_r.clone(),
            world: world_w.clone()
        },
    };
    let mut g = gameplay::Gameplay{
        io: gameplay::Io{
            world_input: world_r.clone(),
            trajectory_output: traj_w.clone()
        }
    };

    let p_handle = thread::spawn(move || {
        p.run();
    });
    let g_handle = thread::spawn(move || {
        g.run();
    });

    for i in 0..100 {
        ssl_w.try_send(i).unwrap();
        println!("Send ssl vision {}", i);
        sleep(Duration::from_millis(100));
    }

    drop(ssl_w);

    println!("About to joiun");
    p_handle.join();
    g_handle.join();


}
