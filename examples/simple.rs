use rhiza::host::Host;
use rhiza::msg::{Pose, RhizaMsg};
use rhiza::node::Node;
use std::thread;
use std::time::Duration;

use simple_signal::{self, Signal};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {

    let mut host = Host::default("store").unwrap();
    host.start().unwrap();

    // Signal handling
    let running = Arc::new(AtomicBool::new(true));
    let r1 = Arc::clone(&running);
    let r2 = Arc::clone(&running);
    simple_signal::set_handler(&[Signal::Int, Signal::Term], move |_signals| {
        r1.store(false, Ordering::SeqCst);
    });

    let thread_pose = thread::spawn(move || {
        let pose_node = Node::default("my_pose");
        while r2.load(Ordering::SeqCst) {
            let pose = Pose { x: 1.0, y: 20.0 };
            pose_node.publish(pose).unwrap();
        }
    });
    thread::sleep(Duration::from_millis(1_000));

    while running.load(Ordering::SeqCst) {
        let result = host.get::<RhizaMsg<Pose>>("my_pose"); //.unwrap();
        println!("taken result: {:?}", result.data.unwrap());
        thread::sleep(Duration::from_millis(1000));
    }

    thread_pose.join().unwrap();
}
