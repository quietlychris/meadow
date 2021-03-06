use meadow::*;
use criterion::{criterion_group, criterion_main};
use rand::prelude::*;

fn meadow_instantiation(c: &mut criterion::Criterion) {
    c.bench_function("create_host", |b| {
        b.iter(|| {
            let mut host = HostConfig::default().build().unwrap();
            host.start().unwrap();
        });
    });

    c.bench_function("create_nodes", |b| {
        let mut host = HostConfig::default().build().unwrap();
        host.start().unwrap();
        b.iter(|| {
            let node = NodeConfig::<usize>::new("SIMPLE_NODE")
                .topic("number")
                .build()
                .unwrap();
            let _node = node.activate().unwrap();
        });
        host.stop().unwrap();
    });
}

fn message_sending(c: &mut criterion::Criterion) {
    c.bench_function("tcp_publish_usize", |b| {
        // Open a Host
        let mut host = HostConfig::default().build().unwrap();
        host.start().unwrap();
        // Create and activate a Node
        let node = NodeConfig::<usize>::new("SIMPLE_NODE")
            .topic("number")
            .build()
            .unwrap();
        let node = node.activate().unwrap();
        let val = 1;
        b.iter(|| {
            node.publish(val).unwrap();
        });
        host.stop().unwrap();
    });

    c.bench_function("udp_publish_usize", |b| {
        // Open a Host
        let mut host = HostConfig::default().build().unwrap();
        host.start().unwrap();
        // Create and activate a Node
        let node = NodeConfig::<usize>::new("SIMPLE_NODE")
            .topic("number")
            .build()
            .unwrap();
        let node = node.activate().unwrap();
        let val = 1;
        b.iter(|| {
            node.publish_udp(val).unwrap();
        });
        host.stop().unwrap();
    });

    c.bench_function("tcp_publish_request_f32", |b| {
        // Open a Host

        let (host, tx, rx) = create_meadow_triple();
        let val = 1.0;
        b.iter(|| {
            tx.publish_udp(val).unwrap();
            let _result = rx.request().unwrap();

        });
        host.stop().unwrap();
    });

    static KB: usize = 1024;

    for size in [1, KB, 2 * KB, 4 * KB, 8 * KB].iter() {
        let bench_name = "msg_".to_owned() + &size.to_string();
        c.bench_function(&bench_name, |b| {
            let mut rng = rand::thread_rng();
            let mut host = HostConfig::default().build().unwrap();
            host.start().unwrap();
            // Create and activate a Node
            let tx = NodeConfig::<Vec<f32>>::new("TX")
                .topic("number")
                .build()
                .unwrap()
                .activate()
                .unwrap();
            let rx = NodeConfig::<Vec<f32>>::new("RX")
                .topic("number")
                .build()
                .unwrap()
                .activate()
                .unwrap();
            let mut nums: Vec<f32> = Vec::with_capacity(*size);
            for _ in 0..nums.len() {
                nums.push(rng.gen());
            }

            
            b.iter(|| {
                tx.publish(nums.clone()).unwrap();
                let _result = rx.request().unwrap();
            });

            let result = rx.request().unwrap();
            assert_eq!(nums, result);
            host.stop().unwrap();
        });
    }
}

criterion_group!(benches, meadow_instantiation, message_sending);
criterion_main!(benches);

/// Helper function for creating a simple network
fn create_meadow_triple() -> (Host, Node<Active, f32>, Node<Active, f32>) {
    let mut host = HostConfig::default().build().unwrap();
    host.start().unwrap();
    // Create and activate a Node
    let tx = NodeConfig::<f32>::new("TX")
        .topic("number")
        .build()
        .unwrap()
        .activate()
        .unwrap();
    let rx = NodeConfig::<f32>::new("RX")
        .topic("number")
        .build()
        .unwrap()
        .activate()
        .unwrap();
    (host, tx, rx)
}