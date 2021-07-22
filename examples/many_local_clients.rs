// This is an example of having many client Nodes publishing a topic 
// to a local Host, which is then printing the updated values.

use rhiza::*;
use std::thread;
use std::time::Duration;

use simple_signal::{self, Signal};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    let mut host = Host::default("rhiza_store").unwrap();
    host.start().unwrap();

    let running = Arc::new(AtomicBool::new(true));
    let r = Arc::clone(&running);
    simple_signal::set_handler(&[Signal::Int, Signal::Term], move |_signals| {
        r.store(false, Ordering::SeqCst);
    });

    let num_clients = 10;
    let mut threads = Vec::with_capacity(num_clients.clone());
    for i in 0..num_clients {
        let r = running.clone();
        let thread_pose = thread::spawn(move || {
            let pose_node = Node::new("my_pose");
            let num = i;
            while r.load(Ordering::SeqCst) {
                let pose = Pose {
                    x: i as f32,
                    y: 20.0,
                };
                // println!("Publishing from thread {}",i);
                pose_node.publish(pose).unwrap();
            }
        });
        thread::sleep(Duration::from_millis(i as u64));
        threads.push(thread_pose);
    }

    while running.load(Ordering::SeqCst) {
        let result = host.get::<RhizaMsg<Pose>>("my_pose"); //.unwrap();
        println!("taken result: {:?}", result.data.unwrap());
        thread::sleep(Duration::from_millis(20));
    }

    for thread in threads {
        thread.join().unwrap();
    }
}
