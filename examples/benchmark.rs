#![allow(unused)]
use bissel::*;

use std::error::Error;
use std::time::Instant;

trait Benchmarkable: 'static + Clone + From<u8> + Message + PartialEq {}
impl<T> Benchmarkable for T where T: 'static + Clone + From<u8> + Message + PartialEq {}

#[derive(Debug)]
struct BenchmarkStats {
    payload_size: usize,
    iterations: usize,
    invalid_operations: usize,
    average_time_us: f64,
    min_time_us: f64,
    max_time_us: f64,
}

// USAGE: $ cargo run --release --example benchmark 100000
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    dbg!(args.len());
    if args.len() != 2 {
        let emsg = format!("USAGE: cargo run --example --release <num_iterations>");
        panic!("{}", emsg);
    }
    let iterations = args[1].parse::<usize>().unwrap();

    // Benchmark a single f32 value
    let f32_stats = benchmark_numeric::<f32>(iterations)?;
    println!("f32 stats: {:#?}", f32_stats);

    let collection_size = 100;
    let vec_f32_stats = benchmark_numeric_collections::<f32>(iterations, collection_size)?;
    println!("f32 collection stats, length: {:#?}", vec_f32_stats);

    Ok(())
}

fn benchmark_numeric<T: Benchmarkable>(
    iterations: usize,
) -> Result<BenchmarkStats, Box<dyn Error>> {
    let mut host: Host = HostConfig::new("lo").build()?;
    host.start()?;
    println!("Host should be running in the background");

    // Get the host up and running
    let node: Node<Idle, T> = NodeConfig::new("BENCH_f32")
        .topic("benchmark_f32")
        .build()
        .unwrap();
    let mut node = node.connect()?;

    let mut payload_size: usize = 0;
    let mut times: Vec<u128> = Vec::with_capacity(iterations);
    let mut invalid_ops: Vec<usize> = Vec::with_capacity(iterations);
    for i in 0..iterations {
        let val: T = rand::random::<u8>().into();
        payload_size = std::mem::size_of_val(&val);
        let clone = val.clone();
        let now = Instant::now();

        // Publish and request a value from the host
        node.publish(clone)?;
        let result = node.request()?;

        let time = now.elapsed().as_micros();

        if result == val {
            times.push(time);
        } else {
            invalid_ops.push(i);
        }
    }

    let stats = BenchmarkStats {
        payload_size,
        iterations,
        invalid_operations: invalid_ops.len(),
        average_time_us: times.iter().sum::<u128>() as f64 / (times.len() as f64),
        min_time_us: *times.iter().min().unwrap() as f64,
        max_time_us: *times.iter().max().unwrap() as f64,
    };

    host.stop()?;
    Ok(stats)
}

fn benchmark_numeric_collections<T: Benchmarkable>(
    iterations: usize,
    collection_size: usize,
) -> Result<BenchmarkStats, Box<dyn Error>> {
    let mut host: Host = HostConfig::new("lo").build()?;
    host.start()?;
    println!("Host should be running in the background");

    // Get the host up and running
    let node: Node<Idle, Vec<T>> = NodeConfig::new("BENCH_f32")
        .topic("benchmark_f32")
        .build()
        .unwrap();
    let mut node = node.connect()?;
    let mut payload_size: usize = 0;

    let mut times: Vec<u128> = Vec::with_capacity(iterations);
    let mut invalid_ops: Vec<usize> = Vec::with_capacity(iterations);
    for i in 0..iterations {
        let mut val: Vec<T> = Vec::with_capacity(collection_size);
        for _ in 0..collection_size {
            val.push(rand::random::<u8>().into());
        }
        payload_size = std::mem::size_of_val(&val[0]) * val.len();
        let clone = val.to_owned().to_vec();
        let now = Instant::now();

        // Publish and request a value from the host
        node.publish(clone)?;
        let result = node.request()?;

        let time = now.elapsed().as_micros();

        if result == val {
            times.push(time);
        } else {
            invalid_ops.push(i);
        }
    }

    let stats = BenchmarkStats {
        payload_size,
        iterations,
        invalid_operations: invalid_ops.len(),
        average_time_us: times.iter().sum::<u128>() as f64 / (times.len() as f64),
        min_time_us: *times.iter().min().unwrap() as f64,
        max_time_us: *times.iter().max().unwrap() as f64,
    };

    host.stop()?;
    Ok(stats)
}
