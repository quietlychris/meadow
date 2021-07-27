use std::net::UdpSocket;

use postcard::*;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::mpsc::channel;
use std::thread::{self, JoinHandle};

use std::sync::{Arc, Mutex};

use std::error::Error;
use std::result::Result;

use crate::msg::*;

#[derive(Debug)]
pub struct Host {
    socket: UdpSocket,
    store: Arc<Mutex<sled::Db>>,
    thread_recv: Option<JoinHandle<()>>,
    thread_store: Option<JoinHandle<()>>,
    thread_send: Option<JoinHandle<()>>,
    running: bool,
}

#[derive(Debug)]
pub struct HostConfig {
    ip: String,
    socket_num: usize,
    store_name: String,
}

impl HostConfig {
    pub fn new(ip: String) -> HostConfig {
        HostConfig {
            ip,
            socket_num: 25_000,
            store_name: "rhiza_store".into(),
        }
    }

    pub fn socket_num(mut self, socket_num: usize) -> HostConfig {
        self.socket_num = socket_num;
        self
    }

    pub fn build(&self) -> Result<Host, Box<dyn Error>> {
        // self.validate();
        let socket = UdpSocket::bind(self.ip.clone() + &self.socket_num.to_string())?;
        let store: sled::Db = sled::open(&self.store_name)?;

        let host = Host {
            socket,
            store: Arc::new(Mutex::new(store)),
            thread_recv: None,
            thread_store: None,
            thread_send: None,
            running: false,
        };

        Ok(host)
    }
}

impl Host {
    pub fn default(store_name: &str) -> Result<Host, Box<dyn Error>> {
        let ip = "127.0.0.1:"; // Defaults to localhost

        let socket = UdpSocket::bind(ip.to_owned() + "25000")?;

        let config = sled::Config::default().path(store_name).temporary(true);
        let store: sled::Db = config.open()?;

        let host = Host {
            socket,
            store: Arc::new(Mutex::new(store)),
            thread_recv: None,
            thread_store: None,
            thread_send: None,
            running: false,
        };

        Ok(host)
    }

    pub fn get<T: Serialize + DeserializeOwned + std::fmt::Debug>(&mut self, query: &str) -> T {
        let store = self.store.lock().unwrap();
        println!("Asking for '{}' from the store", &query);
        let bytes = store.get(query).unwrap().unwrap();
        /*println!(
            "From the kv store using query '{}', we got {:?}",
            query, bytes
        );*/
        let v: T = from_bytes(&bytes[..]).unwrap();
        v
    }

    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        let (tx, rx) = channel();
        let cloned_socket = self.socket.try_clone()?;

        // I don't like how this is done
        let thread_recv = thread::spawn(move || {
            let mut buf = [0u8; 1024];
            loop {
                match cloned_socket.recv(&mut buf) {
                    Ok(num_bytes) => {
                        let recvd = buf[..num_bytes].to_vec();
                        tx.send(recvd).unwrap();
                    }
                    Err(e) => println!("recv function failed: {:?}", e),
                }
            }
        });

        self.thread_recv = Some(thread_recv);

        let store = Arc::clone(&self.store);
        let thread_store = thread::spawn(move || loop {
            if let Ok(bytes) = rx.try_recv() {
                let msg: GenericRhizaMsg = from_bytes(&bytes).unwrap();

                let store = store.lock().unwrap();
                store.insert(msg.name.as_bytes(), bytes).unwrap();

                drop(store);
            }
        });

        self.thread_store = Some(thread_store);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        //self.thread_recv = None;
        //self.thread_store.unwrap().join().unwrap();

        panic!("unimplemented!");
        Ok(())
    }
}

#[test]
fn test_default() {
    let host = Host::default("store");
}
