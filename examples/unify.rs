use meadow::prelude::*;
use std::marker::PhantomData;
use std::thread;
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Pose {
    x: f32,
    y: f32,
}

fn main() {
    let t = Thing::<One>::new();
    let s = Thing::<Two>::new();

    let one = create::<One>();
}

fn create<S: Seuss>() -> Thing<S> {
    Thing::<S>::new()
}

trait Seuss {}

struct One;
impl Seuss for One {}

struct Two;
impl Seuss for Two {}

struct Thing<S: Seuss> {
    __tag: PhantomData<S>,
    data: usize,
}

impl<S: Seuss> Thing<S> {
    fn new() -> Self {
        Thing::<S> {
            __tag: PhantomData::<S>,
            data: 0,
        }
    }
}

// ----

use meadow::node::network_config::Block;
use meadow::Message;
trait ToActive<B: Block, I: Interface, T: Message> {
    fn activate(self) -> Node<B, I, Active, T>;
}

impl<B: Block, I: Interface, T: Message> ToActive<B, I, T> for Node<B, I, Idle, T> {
    fn activate2(self) -> Node<B, I, Active, T> {
        self.activate()
    }
}

//-------

use meadow::node::network_config::Interface;

fn run<I: Interface>() {
    let mut host: Host = HostConfig::default().build().unwrap();
    host.start().unwrap();
    println!("Host should be running in the background");

    // Get the host up and running
    let node: Node<Blocking, I, Idle, Pose> = NodeConfig::new("pose").build().unwrap();
    let node = node.activate().unwrap();

    for i in 0..5 {
        // Could get this by reading a GPS, for example
        let pose = Pose {
            x: i as f32,
            y: i as f32,
        };

        node.publish(pose.clone()).unwrap();
        thread::sleep(Duration::from_millis(10));
        let result = node.request().unwrap();
        println!("Got position: {:?}", result);

        assert_eq!(pose, result.data);
    }
}
