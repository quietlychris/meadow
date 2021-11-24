use rhiza::host::Host;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn run_host() {
    let mut host = Host::default("store").await.unwrap();
    println!("Starting host");
    host.start().await.unwrap();
    /*
    loop {
        let word = match host.get::<String>("hello") {
            Some(word) => word,
            None => "None".to_string(),
        };
        println!("{}", word);
        std::thread::sleep(std::time::Duration::from_millis(1_000));
    }
    */
}

fn main() {
    let handle = thread::spawn(|| {
        run_host();
    });
    println!("Running a host in the background");
    // Do whatever else you'd like while there's a host in the background
    for i in 0..60 {
        thread::sleep(std::time::Duration::from_millis(1_000));
        println!("Still running");
    }
    // Need to kill host w/ Ctrl^C
    handle.join().unwrap();
}
