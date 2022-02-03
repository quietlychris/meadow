use bissel::*;
use tracing::*;

use std::error::Error;
use std::thread;
use std::time::{Duration, Instant};

// $ cargo run --release --example benchmark 100000
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    dbg!(args.len());
    if args.len() != 2 {
        let emsg = format!("Args: cargo run --example --release <num_iterations>");
        panic!(emsg);
    }
    let iterations = args[1].parse::<usize>().unwrap();

    // Benchmark a single f32 value
    let (avg_f32, invalid_ops_f32) = benchmark_f32(iterations)?;
    dbg!(avg_f32, invalid_ops_f32);

    // Benchmark a vector of f32 values
    let (avg_vec_f32, invalid_ops_vec_f32) = benchmark_vec_f32(iterations)?;
    dbg!(avg_vec_f32, invalid_ops_vec_f32);

    Ok(())
}

fn benchmark_f32(iterations: usize) -> Result<(f64, usize), Box<dyn Error>> {
    let mut host: Host = HostConfig::new("lo").build()?;
    host.start()?;
    println!("Host should be running in the background");

    // Get the host up and running
    let mut node: Node<f32> = NodeConfig::new("BENCH_f32")
        .topic("benchmark_f32")
        .build()
        .unwrap();
    node.connect()?;

    let mut times = Vec::with_capacity(iterations);
    let mut invalid_ops = Vec::with_capacity(iterations);
    for i in 0..iterations {
        let val = rand::random::<f32>();
        let clone = val.clone();
        let now = Instant::now();

        // Publish and request a value from the host
        node.publish(clone)?;
        let result = node.request()?;

        let time = now.elapsed().as_micros() as f64;

        if result == val {
            times.push(time);
        } else {
            invalid_ops.push(i);
        }
    }

    let average: f64 = times.iter().sum::<f64>() / (times.len() as f64);
    let total_invalid_ops = invalid_ops.len();

    host.stop()?;
    Ok((average, total_invalid_ops))
}

fn benchmark_vec_f32(iterations: usize) -> Result<(f64, usize), Box<dyn Error>> {
    let mut host: Host = HostConfig::new("lo").build()?;
    host.start()?;
    println!("Host should be running in the background");

    // Get the host up and running
    let mut node: Node<Vec<f32>> = NodeConfig::new("BENCH_vec_f32")
        .topic("benchmark_vec_f32")
        .build()
        .unwrap();
    node.connect()?;

    let mut times = Vec::with_capacity(iterations);
    let mut invalid_ops = Vec::with_capacity(iterations);
    for i in 0..iterations {
        // let val = rand::random::<f32>();
        let length = 100;
        let mut val = Vec::with_capacity(length);
        for _ in 0..length {
            val.push(rand::random::<f32>());
        }
        let clone = val.to_owned().to_vec();

        let now = Instant::now();

        // Publish and request a value from the host
        node.publish(clone)?;
        let result = node.request()?;

        let time = now.elapsed().as_micros() as f64;

        if result == val {
            times.push(time);
        } else {
            invalid_ops.push(i);
        }
    }

    let average: f64 = times.iter().sum::<f64>() / (times.len() as f64);
    let total_invalid_ops = invalid_ops.len();

    host.stop()?;
    Ok((average, total_invalid_ops))
}
