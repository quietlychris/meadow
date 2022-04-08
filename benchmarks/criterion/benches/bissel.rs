use bissel::*;
use criterion::{criterion_group, criterion_main};

fn bissel_instantiation(c: &mut criterion::Criterion) {
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
}

criterion_group!(benches, bissel_instantiation, message_sending);
criterion_main!(benches);
